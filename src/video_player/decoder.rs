// SPDX-License-Identifier: MPL-2.0
//! Async video frame decoder using FFmpeg.
//!
//! This module provides asynchronous video frame decoding via Tokio tasks,
//! delivering frames through channels for non-blocking UI updates.

use crate::error::{Error, Result};
use crate::video_player::frame_cache::{CacheConfig, FrameCache};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;

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
    pub fn new<P: AsRef<Path>>(video_path: P, cache_config: CacheConfig) -> Result<Self> {
        let path = video_path.as_ref().to_path_buf();

        // Validate file exists
        if !path.exists() {
            return Err(Error::Io(format!("Video file not found: {:?}", path)));
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
            if let Err(e) = Self::decoder_loop_blocking(path, command_rx, event_tx, cache_config) {
                eprintln!("Decoder task failed: {}", e);
            }
        });

        Ok(Self {
            command_tx,
            event_rx,
        })
    }

    /// Sends a command to the decoder task.
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
    /// This is the core decoding logic using FFmpeg for frame decoding.
    /// It maintains playback state and responds to commands.
    /// Runs in a separate blocking thread since FFmpeg types are not Send.
    ///
    /// The frame cache is used to optimize seek operations by caching
    /// keyframes (I-frames) that can be independently decoded.
    fn decoder_loop_blocking(
        video_path: std::path::PathBuf,
        mut command_rx: mpsc::UnboundedReceiver<DecoderCommand>,
        event_tx: mpsc::Sender<DecoderEvent>,
        cache_config: CacheConfig,
    ) -> Result<()> {
        // Initialize FFmpeg (with log level set to suppress warnings)
        crate::media::video::init_ffmpeg()?;

        // Open video file
        let mut ictx = ffmpeg_next::format::input(&video_path)
            .map_err(|e| Error::Io(format!("Failed to open video: {}", e)))?;

        // Find video stream
        let input = ictx
            .streams()
            .best(ffmpeg_next::media::Type::Video)
            .ok_or_else(|| Error::Io("No video stream found".to_string()))?;
        let video_stream_index = input.index();

        // Create decoder
        let context_decoder =
            ffmpeg_next::codec::context::Context::from_parameters(input.parameters())
                .map_err(|e| Error::Io(format!("Failed to create codec context: {}", e)))?;
        let mut decoder = context_decoder
            .decoder()
            .video()
            .map_err(|e| Error::Io(format!("Failed to create video decoder: {}", e)))?;

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
        .map_err(|e| Error::Io(format!("Failed to create scaler: {}", e)))?;

        // Extract time base for PTS calculation
        let time_base = input.time_base();
        let time_base_f64 = f64::from(time_base.numerator()) / f64::from(time_base.denominator());

        // Playback state
        let mut is_playing = false;
        let mut playback_start_time: Option<std::time::Instant> = None;
        let mut first_pts: Option<f64> = None;
        let mut decode_single_frame = false; // Flag to decode one frame after seek while paused
        let mut in_stepping_mode = false; // True when user is stepping through frames
        let mut last_paused_frame: Option<DecodedFrame> = None; // Frame displayed after seek (for history)

        // Frame cache for optimized seeking
        let mut frame_cache = FrameCache::new(cache_config);

        // Frame history for backward stepping
        // Default to 128MB - convert from cache's max_bytes to MB
        let history_mb = (cache_config.max_bytes / (1024 * 1024)).clamp(32, 512) as u32;
        let mut frame_history = FrameHistory::new(history_mb);

        // Main loop: process commands and decode frames
        loop {
            // Check for commands (non-blocking)
            match command_rx.try_recv() {
                Ok(DecoderCommand::Play { .. }) => {
                    // No seek needed on resume - the demuxer maintains its position.
                    // Just like audio, we continue from where we were.
                    is_playing = true;
                    playback_start_time = Some(std::time::Instant::now());
                    first_pts = None;
                    // Exit stepping mode and clear history on play
                    in_stepping_mode = false;
                    frame_history.clear();
                    last_paused_frame = None;
                    let _ = event_tx.blocking_send(DecoderEvent::Buffering);
                }
                Ok(DecoderCommand::Pause) => {
                    is_playing = false;
                    playback_start_time = None;
                    first_pts = None;
                }
                Ok(DecoderCommand::Seek { target_secs }) => {
                    // Check cache first for instant preview (only when paused)
                    // This provides immediate visual feedback while FFmpeg seeks
                    const CACHE_TOLERANCE_SECS: f64 = 0.5;
                    let mut cache_hit = false;
                    if !is_playing {
                        if let Some(cached_frame) = frame_cache.get_at_or_before(target_secs) {
                            let distance = target_secs - cached_frame.pts_secs;
                            if distance <= CACHE_TOLERANCE_SECS {
                                // Cache hit - send cached frame immediately for instant preview
                                let decoded = DecodedFrame {
                                    rgba_data: Arc::clone(&cached_frame.rgba_data),
                                    width: cached_frame.width,
                                    height: cached_frame.height,
                                    pts_secs: cached_frame.pts_secs,
                                };
                                let _ = event_tx.blocking_send(DecoderEvent::FrameReady(decoded));
                                cache_hit = true;
                                // Don't skip FFmpeg seek - we still need to position the demuxer
                                // for subsequent playback (e.g., when restarting from end)
                            }
                        }
                    }

                    // Always do FFmpeg seek to position demuxer correctly
                    // Convert seconds to AV_TIME_BASE (microseconds)
                    let timestamp = (target_secs * 1_000_000.0) as i64;
                    if let Err(e) = ictx.seek(timestamp, ..timestamp) {
                        let _ = event_tx
                            .blocking_send(DecoderEvent::Error(format!("Seek failed: {}", e)));
                    } else {
                        decoder.flush();
                        // Reset timing after seek
                        playback_start_time = Some(std::time::Instant::now());
                        first_pts = None;
                        // Clear frame history on seek - frames after seek won't be sequential
                        in_stepping_mode = false;
                        frame_history.clear();
                        last_paused_frame = None;
                        if !cache_hit {
                            // Only send Buffering if we didn't already send a cached frame
                            let _ = event_tx.blocking_send(DecoderEvent::Buffering);
                        }
                        // If paused and no cache hit, decode one frame to show the seek result
                        if !is_playing && !cache_hit {
                            decode_single_frame = true;
                        }
                    }
                }
                Ok(DecoderCommand::StepFrame) => {
                    // Step forward one frame without seeking
                    // This decodes the next frame in sequence
                    eprintln!("[DEBUG] DECODER: StepFrame command received");
                    if !is_playing {
                        // When entering stepping mode, add the current frame to history first
                        // This allows stepping backward to the frame shown before stepping started
                        if !in_stepping_mode {
                            if let Some(ref initial_frame) = last_paused_frame {
                                eprintln!(
                                    "[DEBUG] DECODER: Adding initial frame to history at PTS {}",
                                    initial_frame.pts_secs
                                );
                                frame_history.push(initial_frame.clone());
                            }
                        }
                        in_stepping_mode = true;
                        decode_single_frame = true;
                    }
                }
                Ok(DecoderCommand::StepBackward) => {
                    // Step backward one frame using frame history
                    eprintln!("[DEBUG] DECODER: StepBackward command received");
                    if !is_playing && in_stepping_mode {
                        if let Some(prev_frame) = frame_history.step_back() {
                            eprintln!(
                                "[DEBUG] DECODER: Got previous frame from history at PTS {}",
                                prev_frame.pts_secs
                            );
                            // Send the previous frame
                            let decoded = DecodedFrame {
                                rgba_data: Arc::clone(&prev_frame.rgba_data),
                                width: prev_frame.width,
                                height: prev_frame.height,
                                pts_secs: prev_frame.pts_secs,
                            };
                            let _ = event_tx.blocking_send(DecoderEvent::FrameReady(decoded));
                        } else {
                            eprintln!("[DEBUG] DECODER: No previous frame in history");
                        }
                    }
                }
                Ok(DecoderCommand::Stop) => {
                    break;
                }
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    // Command channel closed
                    break;
                }
                Err(mpsc::error::TryRecvError::Empty) => {
                    // No commands, continue
                }
            }

            // If not playing and no single frame needed, yield to avoid busy-waiting
            if !is_playing && !decode_single_frame {
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }

            // Decode next frame
            let mut frame_decoded = false;
            for (stream, packet) in ictx.packets() {
                if stream.index() != video_stream_index {
                    continue;
                }

                // Send packet to decoder
                if let Err(e) = decoder.send_packet(&packet) {
                    let _ = event_tx
                        .blocking_send(DecoderEvent::Error(format!("Packet send failed: {}", e)));
                    continue;
                }

                // Try to receive decoded frame
                let mut decoded_frame = ffmpeg_next::frame::Video::empty();
                if decoder.receive_frame(&mut decoded_frame).is_ok() {
                    // Convert to RGBA
                    let mut rgb_frame = ffmpeg_next::frame::Video::empty();
                    if let Err(e) = scaler.run(&decoded_frame, &mut rgb_frame) {
                        let _ = event_tx
                            .blocking_send(DecoderEvent::Error(format!("Scaling failed: {}", e)));
                        continue;
                    }

                    // Extract RGBA data
                    let rgba_data = Self::extract_rgba_data(&rgb_frame);

                    // Calculate PTS in seconds
                    let pts_secs = if let Some(pts) = decoded_frame.timestamp() {
                        pts as f64 * time_base_f64
                    } else {
                        0.0
                    };

                    // Frame pacing: wait until the frame should be displayed
                    if let Some(start_time) = playback_start_time {
                        // Store first frame PTS as reference
                        if first_pts.is_none() {
                            first_pts = Some(pts_secs);
                        }

                        if let Some(first) = first_pts {
                            // Calculate when this frame should be displayed relative to playback start
                            let frame_delay = pts_secs - first;
                            let target_time =
                                start_time + std::time::Duration::from_secs_f64(frame_delay);
                            let now = std::time::Instant::now();

                            // Wait until target time
                            if target_time > now {
                                std::thread::sleep(target_time - now);
                            }
                        }
                    }

                    // Check if this is a keyframe for caching
                    let is_keyframe = decoded_frame.is_key();

                    // Send frame event
                    let decoded = DecodedFrame {
                        rgba_data: Arc::new(rgba_data),
                        width,
                        height,
                        pts_secs,
                    };

                    // Cache keyframes for optimized seeking
                    // Only keyframes can be independently decoded, so they're ideal for caching
                    if is_keyframe {
                        frame_cache.insert(decoded.clone(), true);
                    }

                    // Store the frame shown while paused (for stepping mode history)
                    // This allows backward stepping to return to the frame shown before stepping started
                    if !is_playing && !in_stepping_mode {
                        eprintln!("[DEBUG] DECODER: Storing paused frame at PTS {}", pts_secs);
                        last_paused_frame = Some(decoded.clone());
                    }

                    // Store frame in history during stepping mode for backward navigation
                    if in_stepping_mode {
                        eprintln!(
                            "[DEBUG] DECODER: Adding frame to history at PTS {}",
                            pts_secs
                        );
                        frame_history.push(decoded.clone());
                    }

                    if event_tx
                        .blocking_send(DecoderEvent::FrameReady(decoded))
                        .is_err()
                    {
                        // Event channel closed
                        break;
                    }

                    frame_decoded = true;
                    // Clear single frame flag after decoding
                    decode_single_frame = false;
                    break;
                }
            }

            // If no frame was decoded, we've reached end of stream
            if !frame_decoded {
                let _ = event_tx.blocking_send(DecoderEvent::EndOfStream);
                is_playing = false;
                playback_start_time = None;
                first_pts = None;
                decode_single_frame = false;
            }
        }

        Ok(())
    }

    /// Extracts RGBA data from a decoded frame, handling stride correctly.
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
        } else if self.position == 1 && !self.frames.is_empty() {
            // Already at first frame, return it again (can't go further back)
            self.frames.front()
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

        let decoder = AsyncDecoder::new(&video_path, CacheConfig::default());
        assert!(decoder.is_ok());
    }

    #[tokio::test]
    async fn decoder_fails_for_nonexistent_file() {
        let result = AsyncDecoder::new("/nonexistent/video.mp4", CacheConfig::default());
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn decoder_accepts_commands() {
        let temp_dir = tempfile::tempdir().unwrap();
        let video_path = temp_dir.path().join("test.mp4");
        std::fs::write(&video_path, b"fake video data").unwrap();

        let decoder = AsyncDecoder::new(&video_path, CacheConfig::default()).unwrap();

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

        let mut decoder = AsyncDecoder::new(video_path, CacheConfig::default()).unwrap();

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
                panic!("Unexpected error from decoder: {}", msg);
            }
            other => {
                panic!("Expected Buffering or FrameReady event, got: {:?}", other);
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
