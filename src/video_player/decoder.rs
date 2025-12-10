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
    /// Start decoding from the beginning.
    Play,

    /// Pause decoding (stop sending frames).
    Pause,

    /// Seek to a specific timestamp and pause.
    Seek { target_secs: f64 },

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
        let mut current_pts_secs: f64 = 0.0; // Track current position for pause/resume
        let mut decode_single_frame = false; // Flag to decode one frame after seek while paused

        // Frame cache for optimized seeking
        let mut frame_cache = FrameCache::new(cache_config);

        // Main loop: process commands and decode frames
        loop {
            // Check for commands (non-blocking)
            match command_rx.try_recv() {
                Ok(DecoderCommand::Play) => {
                    // If resuming from pause, seek to the paused position
                    if !is_playing && current_pts_secs > 0.0 {
                        // Convert seconds to AV_TIME_BASE (microseconds)
                        // FFmpeg seek uses AV_TIME_BASE which is 1_000_000
                        let timestamp = (current_pts_secs * 1_000_000.0) as i64;
                        // Use RangeTo (..timestamp) to allow FFmpeg to seek backward to keyframe
                        if let Err(e) = ictx.seek(timestamp, ..timestamp) {
                            let _ = event_tx.blocking_send(DecoderEvent::Error(format!(
                                "Resume seek failed: {}",
                                e
                            )));
                        } else {
                            decoder.flush();
                        }
                    }
                    is_playing = true;
                    playback_start_time = Some(std::time::Instant::now());
                    first_pts = None;
                    let _ = event_tx.blocking_send(DecoderEvent::Buffering);
                }
                Ok(DecoderCommand::Pause) => {
                    // Keep current_pts_secs for resume - do NOT reset it
                    is_playing = false;
                    playback_start_time = None;
                    first_pts = None;
                }
                Ok(DecoderCommand::Seek { target_secs }) => {
                    // Check cache first for instant seek (only when paused, not during playback)
                    // Only use cache if we have a keyframe within 0.5 seconds of target
                    // This avoids showing a frame too far from where user seeked
                    const CACHE_TOLERANCE_SECS: f64 = 0.5;
                    if !is_playing {
                        if let Some(cached_frame) = frame_cache.get_at_or_before(target_secs) {
                            let distance = target_secs - cached_frame.pts_secs;
                            if distance <= CACHE_TOLERANCE_SECS {
                                // Cache hit - send cached frame immediately
                                current_pts_secs = cached_frame.pts_secs;
                                let decoded = DecodedFrame {
                                    rgba_data: Arc::clone(&cached_frame.rgba_data),
                                    width: cached_frame.width,
                                    height: cached_frame.height,
                                    pts_secs: cached_frame.pts_secs,
                                };
                                let _ = event_tx.blocking_send(DecoderEvent::FrameReady(decoded));
                                // Skip FFmpeg seek - we already have the frame
                                continue;
                            }
                        }
                    }

                    // Cache miss or playing - do FFmpeg seek
                    // Convert seconds to AV_TIME_BASE (microseconds)
                    let timestamp = (target_secs * 1_000_000.0) as i64;
                    if let Err(e) = ictx.seek(timestamp, ..timestamp) {
                        let _ = event_tx
                            .blocking_send(DecoderEvent::Error(format!("Seek failed: {}", e)));
                    } else {
                        decoder.flush();
                        // Update current position to seek target
                        current_pts_secs = target_secs;
                        // Reset timing after seek
                        playback_start_time = Some(std::time::Instant::now());
                        first_pts = None;
                        let _ = event_tx.blocking_send(DecoderEvent::Buffering);
                        // If paused, decode one frame to show the seek result
                        if !is_playing {
                            decode_single_frame = true;
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

                    // Update current position for pause/resume
                    current_pts_secs = pts_secs;

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
        assert!(decoder.send_command(DecoderCommand::Play).is_ok());
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
        decoder.send_command(DecoderCommand::Play).unwrap();

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
