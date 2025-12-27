// SPDX-License-Identifier: MPL-2.0
//! Async video frame decoder using `FFmpeg`.
//!
//! This module provides asynchronous video frame decoding via Tokio tasks,
//! delivering frames through channels for non-blocking UI updates.

use crate::error::{Error, Result};
use crate::video_player::frame_cache::{CacheConfig, FrameCache};
use crate::video_player::sync::{calculate_sync_action, SharedSyncClock, SyncAction};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Maximum number of frames to skip during precise seeking.
/// Prevents infinite loops on corrupted files or seeks beyond EOF.
/// At 30fps, 1000 frames = ~33 seconds of video.
const MAX_SEEK_FRAMES: u32 = 1000;

/// Maximum consecutive frames to skip when video is behind audio.
/// After this many skips, we display the next frame anyway to prevent freezing.
const MAX_CONSECUTIVE_SKIPS: u32 = 5;

/// Result of frame pacing calculation.
#[derive(Debug, Clone, Copy, PartialEq)]
enum PacingResult {
    /// Display the frame (possibly after sleeping).
    Display,
    /// Skip this frame, decode next one.
    SkipFrame,
}

/// Applies frame pacing based on A/V sync or wall-clock timing.
///
/// This function handles the timing of when to display video frames:
/// - When a sync clock is available (audio playing), uses A/V sync
/// - Otherwise, uses wall-clock timing based on playback start time
///
/// May sleep to wait for the correct display time.
/// Returns `SkipFrame` if the frame should be skipped (video behind audio).
///
/// # Arguments
/// * `pts_secs` - Frame presentation timestamp in seconds
/// * `first_pts` - Reference PTS for timing calculation (mutated if None)
/// * `playback_speed` - Current playback speed multiplier
/// * `sync_clock` - Optional audio sync clock for A/V synchronization
/// * `playback_start_time` - Wall-clock reference for timing
/// * `consecutive_skips` - Counter for consecutive skipped frames (mutated)
fn apply_frame_pacing(
    pts_secs: f64,
    first_pts: &mut Option<f64>,
    playback_speed: f64,
    sync_clock: &Option<SharedSyncClock>,
    playback_start_time: Option<std::time::Instant>,
    consecutive_skips: &mut u32,
) -> PacingResult {
    // Adjust PTS for playback speed
    let adjusted_pts = if let Some(first) = *first_pts {
        first + (pts_secs - first) / playback_speed
    } else {
        *first_pts = Some(pts_secs);
        pts_secs
    };

    // A/V sync: use audio clock if available
    if let Some(ref clock) = sync_clock {
        if clock.is_playing() && clock.is_sync_enabled() {
            let audio_time = clock.current_time_secs();
            match calculate_sync_action(adjusted_pts, audio_time) {
                SyncAction::Display => {
                    *consecutive_skips = 0;
                }
                SyncAction::Wait(duration) => {
                    *consecutive_skips = 0;
                    std::thread::sleep(duration);
                }
                SyncAction::Skip => {
                    *consecutive_skips += 1;
                    if *consecutive_skips < MAX_CONSECUTIVE_SKIPS {
                        #[cfg(debug_assertions)]
                        eprintln!(
                            "[sync] Skipping frame (video behind by {:.3}s, skip #{})",
                            audio_time - adjusted_pts,
                            *consecutive_skips
                        );
                        return PacingResult::SkipFrame;
                    }
                    // Too many skips, display anyway to prevent freezing
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "[sync] Max skips reached, displaying frame (behind by {:.3}s)",
                        audio_time - adjusted_pts
                    );
                    *consecutive_skips = 0;
                }
                SyncAction::Repeat => {
                    *consecutive_skips = 0;
                }
            }
            return PacingResult::Display;
        }
    }

    // Fallback: wall-clock timing (when no sync clock or not playing)
    if let Some(start_time) = playback_start_time {
        if let Some(first) = *first_pts {
            let frame_delay = (pts_secs - first) / playback_speed;
            let target_time = start_time + std::time::Duration::from_secs_f64(frame_delay);
            let now = std::time::Instant::now();
            if target_time > now {
                std::thread::sleep(target_time - now);
            }
        }
    }

    PacingResult::Display
}

/// Holds mutable state for the decoder loop.
///
/// This struct groups together all state variables that are modified during
/// the decode loop, making it easier to pass them to helper functions.
struct DecoderLoopState {
    /// Whether playback is currently active.
    is_playing: bool,
    /// Wall-clock reference for frame timing.
    playback_start_time: Option<std::time::Instant>,
    /// Reference PTS for timing calculation.
    first_pts: Option<f64>,
    /// Flag to decode a single frame (after seek while paused).
    decode_single_frame: bool,
    /// True when user is stepping through frames.
    in_stepping_mode: bool,
    /// Frame displayed after seek (for stepping history).
    last_paused_frame: Option<DecodedFrame>,
    /// Target PTS for precise seeking.
    seek_target_secs: Option<f64>,
    /// Counter for frames skipped during precise seeking.
    seek_frames_skipped: u32,
    /// Current playback speed multiplier.
    playback_speed: f64,
    /// Counter for consecutive A/V sync frame skips.
    consecutive_skips: u32,
}

impl DecoderLoopState {
    /// Creates a new decoder loop state with default values.
    fn new() -> Self {
        Self {
            is_playing: false,
            playback_start_time: None,
            first_pts: None,
            decode_single_frame: false,
            in_stepping_mode: false,
            last_paused_frame: None,
            seek_target_secs: None,
            seek_frames_skipped: 0,
            playback_speed: 1.0,
            consecutive_skips: 0,
        }
    }

    /// Resets timing state (called after seek or when starting playback).
    fn reset_timing(&mut self) {
        self.playback_start_time = Some(std::time::Instant::now());
        self.first_pts = None;
    }

    /// Clears stepping mode state.
    fn clear_stepping(&mut self, frame_history: &mut FrameHistory) {
        self.in_stepping_mode = false;
        frame_history.clear();
        self.last_paused_frame = None;
    }
}

/// Result of processing a decoder command.
enum CommandResult {
    /// Continue the main loop normally.
    Continue,
    /// Break out of the main loop (stop decoder).
    Break,
    /// A frame was emitted from history, skip decoding this iteration.
    FrameEmitted,
}

/// Result of packet decoding with seek handling.
enum PacketDecodeResult {
    /// Frame was emitted successfully.
    FrameEmitted,
    /// Frame stored for seeking, continue decoding.
    ContinueDecoding,
    /// Seek timeout reached.
    SeekTimeout,
    /// Frame skipped (A/V sync), continue to next packet.
    FrameSkipped,
    /// Channel closed, break from loops.
    ChannelClosed,
    /// Error occurred, continue to next packet.
    Error,
}

/// Handles end-of-stream: emits last decoded frame if seeking beyond EOF.
///
/// Returns true if a frame was emitted, false otherwise.
#[allow(clippy::too_many_arguments)]
fn handle_end_of_stream(
    last_decoded_for_seek: Option<(ffmpeg_next::frame::Video, f64, bool)>,
    state: &mut DecoderLoopState,
    scaler: &mut ffmpeg_next::software::scaling::Context,
    frame_cache: &mut FrameCache,
    frame_history: &mut FrameHistory,
    event_tx: &mpsc::Sender<DecoderEvent>,
    width: u32,
    height: u32,
) -> bool {
    if let Some((last_frame, pts_secs, is_keyframe)) = last_decoded_for_seek {
        state.seek_target_secs = None;
        let mut rgb_frame = ffmpeg_next::frame::Video::empty();
        if scaler.run(&last_frame, &mut rgb_frame).is_ok()
            && emit_frame(
                &rgb_frame,
                pts_secs,
                is_keyframe,
                state,
                frame_cache,
                frame_history,
                event_tx,
                width,
                height,
            )
        {
            return true;
        }
    }
    false
}

/// Processes a decoded packet frame with seek timeout handling.
///
/// This function handles the packet decoding path which includes seek timeout
/// detection that isn't needed for buffered frames.
#[allow(clippy::too_many_arguments)]
fn process_packet_frame(
    decoded_frame: &ffmpeg_next::frame::Video,
    time_base_f64: f64,
    state: &mut DecoderLoopState,
    scaler: &mut ffmpeg_next::software::scaling::Context,
    frame_cache: &mut FrameCache,
    frame_history: &mut FrameHistory,
    event_tx: &mpsc::Sender<DecoderEvent>,
    sync_clock: &Option<SharedSyncClock>,
    last_decoded_for_seek: &mut Option<(ffmpeg_next::frame::Video, f64, bool)>,
    width: u32,
    height: u32,
) -> PacketDecodeResult {
    #[allow(clippy::cast_precision_loss)]
    let pts_secs = if let Some(pts) = decoded_frame.timestamp() {
        pts as f64 * time_base_f64
    } else {
        0.0
    };
    let is_keyframe = decoded_frame.is_key();

    // Precise seeking with timeout protection
    if let Some(target) = state.seek_target_secs {
        if pts_secs < target {
            state.seek_frames_skipped += 1;
            if state.seek_frames_skipped >= MAX_SEEK_FRAMES {
                let _ = event_tx.blocking_send(DecoderEvent::Error(
                    "Seek timeout: target position may be beyond end of file".to_string(),
                ));
                state.seek_target_secs = None;
                if let Some((_frame, pts, _)) = last_decoded_for_seek.take() {
                    state.first_pts = Some(pts);
                }
                return PacketDecodeResult::SeekTimeout;
            }
            *last_decoded_for_seek = Some((decoded_frame.clone(), pts_secs, is_keyframe));
            return PacketDecodeResult::ContinueDecoding;
        }
        state.first_pts = Some(target);
        state.seek_target_secs = None;
    }

    // Scale to RGBA
    let mut rgb_frame = ffmpeg_next::frame::Video::empty();
    if scaler.run(decoded_frame, &mut rgb_frame).is_err() {
        let _ = event_tx.blocking_send(DecoderEvent::Error("Scaling failed".to_string()));
        return PacketDecodeResult::Error;
    }

    // Frame pacing during playback
    if state.is_playing {
        let pacing = apply_frame_pacing(
            pts_secs,
            &mut state.first_pts,
            state.playback_speed,
            sync_clock,
            state.playback_start_time,
            &mut state.consecutive_skips,
        );
        if pacing == PacingResult::SkipFrame {
            return PacketDecodeResult::FrameSkipped;
        }
    }

    // Emit the frame
    if emit_frame(
        &rgb_frame,
        pts_secs,
        is_keyframe,
        state,
        frame_cache,
        frame_history,
        event_tx,
        width,
        height,
    ) {
        PacketDecodeResult::FrameEmitted
    } else {
        PacketDecodeResult::ChannelClosed
    }
}

/// Result of processing a decoded frame.
enum FrameProcessingResult {
    /// Frame was emitted successfully.
    Emitted,
    /// Frame was stored for seeking (before target PTS).
    StoredForSeek(ffmpeg_next::frame::Video, f64, bool),
    /// Frame should be skipped (A/V sync).
    Skip,
    /// Channel closed, break from loop.
    ChannelClosed,
    /// Scaling failed, continue to next.
    ScalingFailed,
}

/// Processes a decoded video frame: handles seeking, pacing, scaling, and emission.
///
/// This function consolidates the frame processing logic that appears in both
/// the buffered frame path and the packet decoding path.
#[allow(clippy::too_many_arguments)]
fn process_decoded_frame(
    frame: &ffmpeg_next::frame::Video,
    time_base_f64: f64,
    state: &mut DecoderLoopState,
    scaler: &mut ffmpeg_next::software::scaling::Context,
    frame_cache: &mut FrameCache,
    frame_history: &mut FrameHistory,
    event_tx: &mpsc::Sender<DecoderEvent>,
    sync_clock: &Option<SharedSyncClock>,
    width: u32,
    height: u32,
) -> FrameProcessingResult {
    #[allow(clippy::cast_precision_loss)]
    let pts_secs = if let Some(pts) = frame.timestamp() {
        pts as f64 * time_base_f64
    } else {
        0.0
    };
    let is_keyframe = frame.is_key();

    // Precise seeking: skip frames before target PTS
    if let Some(target) = state.seek_target_secs {
        if pts_secs < target {
            return FrameProcessingResult::StoredForSeek(frame.clone(), pts_secs, is_keyframe);
        }
        state.first_pts = Some(target);
        state.seek_target_secs = None;
    }

    // Scale to RGBA
    let mut rgb_frame = ffmpeg_next::frame::Video::empty();
    if scaler.run(frame, &mut rgb_frame).is_err() {
        return FrameProcessingResult::ScalingFailed;
    }

    // Frame pacing during playback
    if state.is_playing {
        let pacing = apply_frame_pacing(
            pts_secs,
            &mut state.first_pts,
            state.playback_speed,
            sync_clock,
            state.playback_start_time,
            &mut state.consecutive_skips,
        );
        if pacing == PacingResult::SkipFrame {
            return FrameProcessingResult::Skip;
        }
    }

    // Emit the frame
    if emit_frame(
        &rgb_frame,
        pts_secs,
        is_keyframe,
        state,
        frame_cache,
        frame_history,
        event_tx,
        width,
        height,
    ) {
        FrameProcessingResult::Emitted
    } else {
        FrameProcessingResult::ChannelClosed
    }
}

/// Emits a decoded frame after scaling and optional caching.
///
/// Returns true if the frame was sent successfully, false if the channel is closed.
fn emit_frame(
    rgb_frame: &ffmpeg_next::frame::Video,
    pts_secs: f64,
    is_keyframe: bool,
    state: &mut DecoderLoopState,
    frame_cache: &mut FrameCache,
    frame_history: &mut FrameHistory,
    event_tx: &mpsc::Sender<DecoderEvent>,
    width: u32,
    height: u32,
) -> bool {
    let rgba_data = AsyncDecoder::extract_rgba_data(rgb_frame);
    let output_frame = DecodedFrame {
        rgba_data: Arc::new(rgba_data),
        width,
        height,
        pts_secs,
    };

    if is_keyframe {
        frame_cache.insert(output_frame.clone(), true);
    }
    if !state.is_playing && !state.in_stepping_mode {
        state.last_paused_frame = Some(output_frame.clone());
    }
    if state.in_stepping_mode {
        frame_history.push(output_frame.clone());
    }

    event_tx
        .blocking_send(DecoderEvent::FrameReady(output_frame))
        .is_ok()
}

/// Processes a single decoder command.
///
/// Returns `CommandResult` indicating what the main loop should do next.
#[allow(clippy::too_many_arguments)]
fn handle_decoder_command(
    command: DecoderCommand,
    state: &mut DecoderLoopState,
    ictx: &mut ffmpeg_next::format::context::Input,
    decoder: &mut ffmpeg_next::decoder::Video,
    frame_history: &mut FrameHistory,
    event_tx: &mpsc::Sender<DecoderEvent>,
    width: u32,
    height: u32,
) -> CommandResult {
    match command {
        DecoderCommand::Play { .. } => {
            state.is_playing = true;
            state.playback_start_time = Some(std::time::Instant::now());
            state.clear_stepping(frame_history);
            let _ = event_tx.blocking_send(DecoderEvent::Buffering);
        }
        DecoderCommand::Pause => {
            state.is_playing = false;
            state.playback_start_time = None;
            state.first_pts = None;
        }
        DecoderCommand::Seek { target_secs } => {
            #[allow(clippy::cast_possible_truncation)]
            let timestamp = (target_secs * 1_000_000.0) as i64;
            if let Err(e) = ictx.seek(timestamp, ..timestamp) {
                let _ = event_tx.blocking_send(DecoderEvent::Error(format!("Seek failed: {e}")));
            } else {
                decoder.flush();
                state.reset_timing();
                state.clear_stepping(frame_history);
                state.seek_target_secs = Some(target_secs);
                state.seek_frames_skipped = 0;
                let _ = event_tx.blocking_send(DecoderEvent::Buffering);
                if !state.is_playing {
                    state.decode_single_frame = true;
                }
            }
        }
        DecoderCommand::StepFrame => {
            if !state.is_playing {
                state.seek_target_secs = None;
                if !state.in_stepping_mode {
                    if let Some(ref initial_frame) = state.last_paused_frame {
                        frame_history.push(initial_frame.clone());
                    }
                    state.in_stepping_mode = true;
                }
                if let Some(next_frame) = frame_history.step_forward() {
                    let output_frame = DecodedFrame {
                        rgba_data: Arc::clone(&next_frame.rgba_data),
                        width: next_frame.width,
                        height: next_frame.height,
                        pts_secs: next_frame.pts_secs,
                    };
                    let _ = event_tx.blocking_send(DecoderEvent::FrameReady(output_frame));
                    return CommandResult::FrameEmitted;
                }
                state.decode_single_frame = true;
            }
        }
        DecoderCommand::StepBackward => {
            if !state.is_playing && state.in_stepping_mode {
                state.seek_target_secs = None;
                if let Some(prev_frame) = frame_history.step_back() {
                    let output_frame = DecodedFrame {
                        rgba_data: Arc::clone(&prev_frame.rgba_data),
                        width,
                        height,
                        pts_secs: prev_frame.pts_secs,
                    };
                    let _ = event_tx.blocking_send(DecoderEvent::FrameReady(output_frame));
                    return CommandResult::FrameEmitted;
                }
                let _ = event_tx.blocking_send(DecoderEvent::HistoryExhausted);
            }
        }
        DecoderCommand::Stop => return CommandResult::Break,
        DecoderCommand::SetPlaybackSpeed {
            speed,
            instant,
            reference_pts,
        } => {
            state.playback_speed = speed.value();
            if state.is_playing {
                state.playback_start_time = Some(instant);
                state.first_pts = Some(reference_pts);
            }
        }
    }
    CommandResult::Continue
}

/// Represents a decoded video frame ready for display.
#[derive(Debug, Clone)]
pub struct DecodedFrame {
    /// RGBA pixel data (width × height × 4 bytes).
    pub rgba_data: Arc<Vec<u8>>,

    /// Frame width in pixels.
    pub width: u32,

    /// Frame height in pixels.
    pub height: u32,

    /// Presentation timestamp in seconds.
    /// Indicates when this frame should be displayed.
    pub pts_secs: f64,
}

impl DecodedFrame {
    /// Returns the total size in bytes.
    #[must_use]
    pub fn size_bytes(&self) -> usize {
        self.rgba_data.len()
    }
}

/// Commands sent to the decoder task.
#[derive(Debug, Clone)]
pub enum DecoderCommand {
    /// Start or resume playback.
    /// If `resume_position_secs` is Some, seek to that position before playing.
    /// If None, start from the beginning.
    Play { resume_position_secs: Option<f64> },

    /// Pause decoding (stop sending frames).
    Pause,

    /// Seek to a specific timestamp and pause.
    Seek { target_secs: f64 },

    /// Step forward one frame (decode next frame without seeking).
    /// Used for frame-by-frame navigation when paused.
    StepFrame,

    /// Step backward one frame (return previous frame from history).
    /// Used for frame-by-frame backward navigation when paused.
    StepBackward,

    /// Stop decoding and clean up resources.
    Stop,

    /// Set playback speed.
    /// Affects frame pacing timing.
    /// - `speed`: Validated playback speed (guaranteed within valid range)
    /// - `instant`: Wall clock reference for timing synchronization
    /// - `reference_pts`: Video position (in seconds) at the moment of speed change.
    ///   Both video and audio decoders use this as their timing reference.
    SetPlaybackSpeed {
        speed: super::PlaybackSpeed,
        instant: std::time::Instant,
        reference_pts: f64,
    },
}

/// Events sent from the decoder to the UI.
#[derive(Debug, Clone)]
pub enum DecoderEvent {
    /// A new frame is ready for display.
    FrameReady(DecodedFrame),

    /// Decoder is buffering (loading frames).
    Buffering,

    /// Playback reached the end of the video.
    EndOfStream,

    /// An error occurred during decoding.
    Error(String),

    /// Frame history is exhausted (no more frames to step backward).
    /// Sent when `StepBackward` is requested but no previous frame is available.
    HistoryExhausted,
}

/// Async video decoder that runs in a Tokio task.
pub struct AsyncDecoder {
    /// Channel for sending commands to the decoder task.
    command_tx: mpsc::UnboundedSender<DecoderCommand>,

    /// Channel for receiving events from the decoder task.
    /// Bounded to prevent memory accumulation during rapid seeks.
    event_rx: mpsc::Receiver<DecoderEvent>,
}

impl AsyncDecoder {
    /// Creates a new async decoder for the given video file.
    ///
    /// Spawns a Tokio task that handles decoding in the background.
    /// Returns the decoder handle with channels for communication.
    ///
    /// The `cache_config` parameter controls frame caching behavior for
    /// optimized seek performance. Use `CacheConfig::default()` for standard
    /// caching or `CacheConfig::disabled()` to disable caching.
    ///
    /// The `history_mb` parameter controls the maximum memory for frame history
    /// (used for backward frame stepping). Set to 0 to use a default based on `cache_config`.
    ///
    /// The `sync_clock` parameter, if provided, enables A/V synchronization.
    /// The decoder will use the audio clock to decide when to display, skip,
    /// or wait for video frames.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The video file does not exist
    /// - The file cannot be opened or parsed by `FFmpeg`
    /// - No video stream is found in the file
    pub fn new<P: AsRef<Path>>(
        video_path: P,
        cache_config: CacheConfig,
        history_mb: u32,
        sync_clock: Option<SharedSyncClock>,
    ) -> Result<Self> {
        let path = video_path.as_ref().to_path_buf();

        // Validate file exists
        if !path.exists() {
            return Err(Error::Io(format!(
                "Video file not found: {}",
                path.display()
            )));
        }

        // Create channels for bidirectional communication
        // Commands: unbounded (UI needs to send without blocking)
        // Events: bounded to prevent memory accumulation during seeks
        // Capacity of 2 frames ensures backpressure while allowing some buffering
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::channel(2);

        // Spawn the decoder task in a blocking thread
        // FFmpeg operations are not Send, so we use spawn_blocking
        tokio::task::spawn_blocking(move || {
            if let Err(e) = Self::decoder_loop_blocking(
                path,
                command_rx,
                event_tx,
                cache_config,
                history_mb,
                sync_clock,
            ) {
                eprintln!("Decoder task failed: {e}");
            }
        });

        Ok(Self {
            command_tx,
            event_rx,
        })
    }

    /// Sends a command to the decoder task.
    ///
    /// # Errors
    ///
    /// Returns an error if the decoder task is not running (channel closed).
    pub fn send_command(&self, command: DecoderCommand) -> Result<()> {
        self.command_tx
            .send(command)
            .map_err(|_| Error::Io("Decoder task is not running".into()))
    }

    /// Receives the next event from the decoder (non-blocking).
    ///
    /// Returns `None` if no events are available.
    pub fn try_recv_event(&mut self) -> Option<DecoderEvent> {
        self.event_rx.try_recv().ok()
    }

    /// Receives the next event from the decoder (blocking).
    ///
    /// Returns `None` if the decoder task has terminated.
    pub async fn recv_event(&mut self) -> Option<DecoderEvent> {
        self.event_rx.recv().await
    }

    /// Main decoder loop running in a blocking thread.
    ///
    /// This is the core decoding logic using `FFmpeg` for frame decoding.
    /// It maintains playback state and responds to commands.
    /// Runs in a separate blocking thread since `FFmpeg` types are not `Send`.
    ///
    /// The frame cache is used to optimize seek operations by caching
    /// keyframes (I-frames) that can be independently decoded.
    ///
    /// If `sync_clock` is provided, frame pacing uses the audio clock for A/V sync.
    /// Otherwise, falls back to wall-clock based timing.
    #[allow(clippy::needless_pass_by_value)] // PathBuf/Sender need ownership
    #[allow(clippy::too_many_lines)] // Core state machine with inherent complexity (154 lines after refactoring from 607)
    fn decoder_loop_blocking(
        video_path: std::path::PathBuf,
        mut command_rx: mpsc::UnboundedReceiver<DecoderCommand>,
        event_tx: mpsc::Sender<DecoderEvent>,
        cache_config: CacheConfig,
        history_mb: u32,
        sync_clock: Option<SharedSyncClock>,
    ) -> Result<()> {
        // Initialize FFmpeg (with log level set to suppress warnings)
        crate::media::video::init_ffmpeg()?;

        // Open video file
        let mut ictx = ffmpeg_next::format::input(&video_path)
            .map_err(|e| Error::Io(format!("Failed to open video: {e}")))?;

        // Find video stream
        let input = ictx
            .streams()
            .best(ffmpeg_next::media::Type::Video)
            .ok_or_else(|| Error::Io("No video stream found".to_string()))?;
        let video_stream_index = input.index();

        // Create decoder
        let context_decoder =
            ffmpeg_next::codec::context::Context::from_parameters(input.parameters())
                .map_err(|e| Error::Io(format!("Failed to create codec context: {e}")))?;
        let mut decoder = context_decoder
            .decoder()
            .video()
            .map_err(|e| Error::Io(format!("Failed to create video decoder: {e}")))?;

        let width = decoder.width();
        let height = decoder.height();

        // Setup scaler to convert to RGBA
        let mut scaler = ffmpeg_next::software::scaling::Context::get(
            decoder.format(),
            width,
            height,
            ffmpeg_next::format::Pixel::RGBA,
            width,
            height,
            ffmpeg_next::software::scaling::Flags::BILINEAR,
        )
        .map_err(|e| Error::Io(format!("Failed to create scaler: {e}")))?;

        // Extract time base for PTS calculation
        let time_base = input.time_base();
        let time_base_f64 = f64::from(time_base.numerator()) / f64::from(time_base.denominator());

        // Playback state (grouped in struct for cleaner helper function calls)
        let mut state = DecoderLoopState::new();

        // Frame cache for optimized seeking
        let mut frame_cache = FrameCache::new(cache_config);

        // Frame history for backward stepping
        // Use provided history_mb, or fall back to a default based on cache config
        let effective_history_mb = if history_mb > 0 {
            history_mb
        } else {
            // Safe: max_bytes is typically < 1GB, and clamp ensures result fits in u32
            #[allow(clippy::cast_possible_truncation)]
            let mb = (cache_config.max_bytes / (1024 * 1024)).clamp(32, 512) as u32;
            mb
        };
        let mut frame_history = FrameHistory::new(effective_history_mb);

        // Main loop: process commands and decode frames
        loop {
            // Process commands (non-blocking)
            match command_rx.try_recv() {
                Ok(cmd) => {
                    let result = handle_decoder_command(
                        cmd,
                        &mut state,
                        &mut ictx,
                        &mut decoder,
                        &mut frame_history,
                        &event_tx,
                        width,
                        height,
                    );
                    match result {
                        CommandResult::Break => break,
                        CommandResult::FrameEmitted => continue,
                        CommandResult::Continue => {}
                    }
                }
                Err(mpsc::error::TryRecvError::Disconnected) => break,
                Err(mpsc::error::TryRecvError::Empty) => {}
            }

            // If not playing and no single frame needed, yield to avoid busy-waiting
            if !state.is_playing && !state.decode_single_frame {
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }

            // Decode next frame
            let mut frame_decoded = false;
            let mut last_decoded_for_seek: Option<(ffmpeg_next::frame::Video, f64, bool)> = None;

            // Try to receive a frame from the decoder's buffer first
            let mut buffered_frame = ffmpeg_next::frame::Video::empty();
            if decoder.receive_frame(&mut buffered_frame).is_ok() {
                match process_decoded_frame(
                    &buffered_frame,
                    time_base_f64,
                    &mut state,
                    &mut scaler,
                    &mut frame_cache,
                    &mut frame_history,
                    &event_tx,
                    &sync_clock,
                    width,
                    height,
                ) {
                    FrameProcessingResult::Emitted => {
                        frame_decoded = true;
                        state.decode_single_frame = false;
                    }
                    FrameProcessingResult::StoredForSeek(frame, pts, keyframe) => {
                        last_decoded_for_seek = Some((frame, pts, keyframe));
                    }
                    FrameProcessingResult::Skip => continue,
                    FrameProcessingResult::ChannelClosed => break,
                    FrameProcessingResult::ScalingFailed => {}
                }
            }

            if frame_decoded {
                continue;
            }

            'packet_loop: for (stream, packet) in ictx.packets() {
                if stream.index() != video_stream_index {
                    continue;
                }

                if let Err(e) = decoder.send_packet(&packet) {
                    let _ = event_tx
                        .blocking_send(DecoderEvent::Error(format!("Packet send failed: {e}")));
                    continue;
                }

                let mut decoded_frame = ffmpeg_next::frame::Video::empty();
                if decoder.receive_frame(&mut decoded_frame).is_ok() {
                    match process_packet_frame(
                        &decoded_frame,
                        time_base_f64,
                        &mut state,
                        &mut scaler,
                        &mut frame_cache,
                        &mut frame_history,
                        &event_tx,
                        &sync_clock,
                        &mut last_decoded_for_seek,
                        width,
                        height,
                    ) {
                        PacketDecodeResult::FrameEmitted => {
                            frame_decoded = true;
                            state.decode_single_frame = false;
                            break 'packet_loop;
                        }
                        PacketDecodeResult::ContinueDecoding
                        | PacketDecodeResult::SeekTimeout
                        | PacketDecodeResult::FrameSkipped
                        | PacketDecodeResult::Error => continue,
                        PacketDecodeResult::ChannelClosed => break
                    }
                }
            }

            // End of stream handling
            if !frame_decoded {
                let emitted = handle_end_of_stream(
                    last_decoded_for_seek,
                    &mut state,
                    &mut scaler,
                    &mut frame_cache,
                    &mut frame_history,
                    &event_tx,
                    width,
                    height,
                );
                if !emitted {
                    let _ = event_tx.blocking_send(DecoderEvent::EndOfStream);
                }
                state.is_playing = false;
                state.playback_start_time = None;
                state.first_pts = None;
                state.decode_single_frame = false;
            }
        }

        Ok(())
    }

    /// Extracts RGBA data from a decoded frame, handling stride correctly.
    #[allow(clippy::cast_possible_truncation)] // stride is always < u32::MAX for video frames
    fn extract_rgba_data(frame: &ffmpeg_next::frame::Video) -> Vec<u8> {
        let width = frame.width();
        let height = frame.height();
        let data = frame.data(0);
        let stride = frame.stride(0);

        let mut rgba_bytes = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            let row_start = (y * stride as u32) as usize;
            let row_end = row_start + (width * 4) as usize;
            rgba_bytes.extend_from_slice(&data[row_start..row_end]);
        }

        rgba_bytes
    }
}

/// Ring buffer for storing recent decoded frames during stepping mode.
///
/// This allows backward frame stepping by keeping a history of recently
/// decoded frames. The history is only populated during stepping mode
/// to save memory during normal playback.
struct FrameHistory {
    /// Frames stored in order (oldest to newest).
    frames: std::collections::VecDeque<DecodedFrame>,
    /// Current position in the history (for backward navigation).
    /// When stepping forward, frames are added and position is at end.
    /// When stepping backward, position moves toward front.
    position: usize,
    /// Maximum total bytes for all frames.
    max_bytes: usize,
    /// Current total bytes.
    current_bytes: usize,
}

impl FrameHistory {
    /// Creates a new frame history with the given max size in MB.
    fn new(max_mb: u32) -> Self {
        Self {
            frames: std::collections::VecDeque::new(),
            position: 0,
            max_bytes: (max_mb as usize) * 1024 * 1024,
            current_bytes: 0,
        }
    }

    /// Clears all frames from history.
    fn clear(&mut self) {
        self.frames.clear();
        self.position = 0;
        self.current_bytes = 0;
    }

    /// Adds a frame to the history during forward stepping.
    ///
    /// If we're not at the end of history (after stepping backward),
    /// truncate everything after current position before adding.
    fn push(&mut self, frame: DecodedFrame) {
        let frame_size = frame.size_bytes();

        // If we stepped backward, truncate frames after current position
        if self.position < self.frames.len() {
            // Remove frames after current position
            while self.frames.len() > self.position {
                if let Some(removed) = self.frames.pop_back() {
                    self.current_bytes = self.current_bytes.saturating_sub(removed.size_bytes());
                }
            }
        }

        // Remove oldest frames if we'd exceed max size
        while self.current_bytes + frame_size > self.max_bytes && !self.frames.is_empty() {
            if let Some(removed) = self.frames.pop_front() {
                self.current_bytes = self.current_bytes.saturating_sub(removed.size_bytes());
                // Adjust position since we removed from front
                self.position = self.position.saturating_sub(1);
            }
        }

        // Add new frame
        self.frames.push_back(frame);
        self.current_bytes += frame_size;
        self.position = self.frames.len();
    }

    /// Gets the previous frame (for backward stepping).
    ///
    /// Returns None if we're already at the beginning of history.
    fn step_back(&mut self) -> Option<&DecodedFrame> {
        if self.position > 1 {
            // Move to previous frame (position - 2 because position is 1-indexed after last frame)
            self.position -= 1;
            self.frames.get(self.position - 1)
        } else {
            // Already at first frame or empty - can't go further back
            None
        }
    }

    /// Gets the next frame from history (for forward stepping after backward).
    ///
    /// Returns None if we're already at the end of history (need to decode new frame).
    fn step_forward(&mut self) -> Option<&DecodedFrame> {
        if self.position < self.frames.len() {
            self.position += 1;
            self.frames.get(self.position - 1)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn decoder_can_be_created() {
        // Create a temporary test file
        let temp_dir = tempfile::tempdir().unwrap();
        let video_path = temp_dir.path().join("test.mp4");
        std::fs::write(&video_path, b"fake video data").unwrap();

        let decoder = AsyncDecoder::new(&video_path, CacheConfig::default(), 0, None);
        assert!(decoder.is_ok());
    }

    #[tokio::test]
    async fn decoder_fails_for_nonexistent_file() {
        let result = AsyncDecoder::new("/nonexistent/video.mp4", CacheConfig::default(), 0, None);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn decoder_accepts_commands() {
        let temp_dir = tempfile::tempdir().unwrap();
        let video_path = temp_dir.path().join("test.mp4");
        std::fs::write(&video_path, b"fake video data").unwrap();

        let decoder = AsyncDecoder::new(&video_path, CacheConfig::default(), 0, None).unwrap();

        // Send commands (should not error)
        assert!(decoder
            .send_command(DecoderCommand::Play {
                resume_position_secs: None
            })
            .is_ok());
        assert!(decoder.send_command(DecoderCommand::Pause).is_ok());
        assert!(decoder
            .send_command(DecoderCommand::Seek { target_secs: 5.0 })
            .is_ok());
        assert!(decoder.send_command(DecoderCommand::Stop).is_ok());
    }

    #[tokio::test]
    async fn decoder_sends_events() {
        // Use real test video file
        let video_path = "tests/data/sample.mp4";
        if !std::path::Path::new(video_path).exists() {
            eprintln!("Test video not found, skipping test");
            return;
        }

        let mut decoder = AsyncDecoder::new(video_path, CacheConfig::default(), 0, None).unwrap();

        // Send play command
        decoder
            .send_command(DecoderCommand::Play {
                resume_position_secs: None,
            })
            .unwrap();

        // Wait for event (with timeout)
        let event = tokio::time::timeout(Duration::from_millis(500), decoder.recv_event()).await;

        // Should receive Buffering or FrameReady event for real video
        assert!(event.is_ok(), "Timeout waiting for decoder event");
        match event.unwrap() {
            Some(DecoderEvent::Buffering) => {
                // Expected when starting playback
            }
            Some(DecoderEvent::FrameReady(_)) => {
                // Also valid if frame is decoded quickly
            }
            Some(DecoderEvent::Error(msg)) => {
                panic!("Unexpected error from decoder: {msg}");
            }
            other => {
                panic!("Expected Buffering or FrameReady event, got: {other:?}");
            }
        }

        // Clean up
        decoder.send_command(DecoderCommand::Stop).unwrap();
    }

    #[test]
    fn decoded_frame_calculates_size() {
        let frame = DecodedFrame {
            rgba_data: Arc::new(vec![0u8; 1920 * 1080 * 4]),
            width: 1920,
            height: 1080,
            pts_secs: 0.0,
        };

        assert_eq!(frame.size_bytes(), 1920 * 1080 * 4);
        assert_eq!(frame.width, 1920);
        assert_eq!(frame.height, 1080);
    }
}
