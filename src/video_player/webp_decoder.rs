// SPDX-License-Identifier: MPL-2.0
//! Animated WebP decoder using webp-animation crate.
//!
//! This module provides asynchronous animated WebP frame decoding via Tokio tasks,
//! delivering frames through channels for non-blocking UI updates.
//!
//! `FFmpeg` doesn't support animated WebP well, so we use the dedicated webp-animation
//! crate which wraps Google's libwebp library.

use super::decoder::{DecodedFrame, DecoderCommand, DecoderEvent};
use crate::error::{Error, Result};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Async animated WebP decoder that runs in a Tokio task.
///
/// This decoder handles animated WebP files that `FFmpeg` cannot decode properly.
/// It provides the same interface as `AsyncDecoder` for seamless integration.
#[derive(Debug)]
pub struct WebpAnimDecoder {
    /// Channel for sending commands to the decoder task.
    command_tx: mpsc::UnboundedSender<DecoderCommand>,

    /// Channel for receiving events from the decoder task.
    event_rx: mpsc::Receiver<DecoderEvent>,
}

impl WebpAnimDecoder {
    /// Creates a new async WebP decoder for the given animated WebP file.
    ///
    /// Spawns a Tokio task that handles decoding in the background.
    /// Returns the decoder handle with channels for communication.
    ///
    /// # Errors
    ///
    /// Returns an error if the file does not exist or cannot be read.
    pub fn new<P: AsRef<Path>>(webp_path: P) -> Result<Self> {
        let path = webp_path.as_ref().to_path_buf();

        // Validate file exists
        if !path.exists() {
            return Err(Error::Io(format!("WebP file not found: {}", path.display())));
        }

        // Read the entire file into memory (WebP decoder requires full buffer)
        let webp_data = std::fs::read(&path)
            .map_err(|e| Error::Io(format!("Failed to read WebP file: {e}")))?;

        // Create channels for bidirectional communication
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::channel(2);

        // Spawn the decoder task in a blocking thread
        // webp-animation Decoder is not Send, so we use spawn_blocking
        tokio::task::spawn_blocking(move || {
            if let Err(e) = Self::decoder_loop_blocking(webp_data, command_rx, event_tx) {
                eprintln!("WebP decoder task failed: {e}");
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
    /// Returns an error if the decoder task is not running.
    pub fn send_command(&self, command: DecoderCommand) -> Result<()> {
        self.command_tx
            .send(command)
            .map_err(|_| Error::Io("WebP decoder task is not running".into()))
    }

    /// Receives the next event from the decoder (blocking).
    pub async fn recv_event(&mut self) -> Option<DecoderEvent> {
        self.event_rx.recv().await
    }

    /// Main decoder loop running in a blocking thread.
    // Allow too_many_lines: decoder loop with command handling and frame pacing.
    // Marginal benefit from extraction (131 lines vs 100 limit).
    // Allow similar_names: `decoder` vs `decoded` is intentional -
    // they represent the decoder object and its decoded output respectively.
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::similar_names)]
    #[allow(clippy::needless_pass_by_value)] // Vec and Sender are consumed
    fn decoder_loop_blocking(
        webp_data: Vec<u8>,
        mut command_rx: mpsc::UnboundedReceiver<DecoderCommand>,
        event_tx: mpsc::Sender<DecoderEvent>,
    ) -> Result<()> {
        // Create WebP decoder
        let decoder = webp_animation::Decoder::new(&webp_data)
            .map_err(|e| Error::Io(format!("Failed to create WebP decoder: {e:?}")))?;

        let (width, height) = decoder.dimensions();

        // Collect all frames into memory with their timestamps
        // WebP animations are typically short, so this is acceptable
        let mut frames: Vec<(Vec<u8>, i32)> = Vec::new();
        for frame in decoder {
            let data = frame.data().to_vec();
            let timestamp_ms = frame.timestamp();
            frames.push((data, timestamp_ms));
        }

        if frames.is_empty() {
            return Err(Error::Io("No frames found in WebP file".to_string()));
        }

        // Calculate frame durations from timestamps
        // timestamp(i) is when frame i ends, so duration = timestamp(i) - timestamp(i-1)
        let mut frame_durations_ms: Vec<i32> = Vec::with_capacity(frames.len());
        let mut prev_timestamp = 0;
        for (_, timestamp) in &frames {
            let duration = *timestamp - prev_timestamp;
            frame_durations_ms.push(duration.max(16)); // Minimum 16ms (~60fps)
            prev_timestamp = *timestamp;
        }

        // Playback state
        let mut is_playing = false;
        let mut current_frame_idx = 0usize;
        let mut playback_start_time: Option<std::time::Instant> = None;
        let loop_enabled = true; // WebP animations typically loop
        let mut decode_single_frame = false;
        let mut playback_speed: f64 = 1.0;

        // Main loop: process commands and send frames
        loop {
            // Check for commands (non-blocking)
            match command_rx.try_recv() {
                Ok(DecoderCommand::Play {
                    resume_position_secs: _,
                }) => {
                    // Note: WebP animations don't support precise position resume,
                    // they restart from current frame index which is preserved on pause
                    is_playing = true;
                    playback_start_time = Some(std::time::Instant::now());
                    let _ = event_tx.blocking_send(DecoderEvent::Buffering);
                }
                Ok(DecoderCommand::Pause) => {
                    is_playing = false;
                    playback_start_time = None;
                }
                Ok(DecoderCommand::Seek { target_secs }) => {
                    // Find the frame closest to target time
                    // WebP animations are typically short, so i32 ms is sufficient (~24 days max)
                    #[allow(clippy::cast_possible_truncation)]
                    let target_ms = (target_secs * 1000.0) as i32;
                    let mut accumulated_ms = 0i32;
                    current_frame_idx = 0;

                    for (idx, &duration) in frame_durations_ms.iter().enumerate() {
                        if accumulated_ms + duration > target_ms {
                            current_frame_idx = idx;
                            break;
                        }
                        accumulated_ms += duration;
                        current_frame_idx = idx;
                    }

                    // Reset timing for seek
                    playback_start_time = Some(std::time::Instant::now());
                    let _ = event_tx.blocking_send(DecoderEvent::Buffering);

                    // If paused, show the seeked frame
                    if !is_playing {
                        decode_single_frame = true;
                    }
                }
                Ok(DecoderCommand::StepFrame) => {
                    // Step forward one frame in WebP animation
                    if !is_playing {
                        decode_single_frame = true;
                    }
                }
                Ok(DecoderCommand::StepBackward) => {
                    // Step backward one frame in WebP animation
                    // WebP animations have random access to all frames, so we can just go back
                    if !is_playing && current_frame_idx > 0 {
                        current_frame_idx -= 1;
                        decode_single_frame = true;
                    }
                }
                Ok(DecoderCommand::Stop) | Err(mpsc::error::TryRecvError::Disconnected) => {
                    break;
                }
                Ok(DecoderCommand::SetPlaybackSpeed {
                    speed,
                    instant,
                    reference_pts: _,
                }) => {
                    // PlaybackSpeed newtype guarantees valid range
                    playback_speed = speed.value();
                    // Reset timing references for smooth speed transition
                    // WebP animations don't have audio, so reference_pts is unused
                    if is_playing {
                        playback_start_time = Some(instant);
                    }
                }
                Err(mpsc::error::TryRecvError::Empty) => {
                    // No commands, continue
                }
            }

            // If not playing and no single frame needed, yield
            if !is_playing && !decode_single_frame {
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }

            // Get current frame
            if current_frame_idx >= frames.len() {
                if loop_enabled {
                    // Loop back to start
                    current_frame_idx = 0;
                    playback_start_time = Some(std::time::Instant::now());
                } else {
                    // End of stream
                    let _ = event_tx.blocking_send(DecoderEvent::EndOfStream);
                    is_playing = false;
                    playback_start_time = None;
                    continue;
                }
            }

            let (rgba_data, _) = &frames[current_frame_idx];

            // Calculate PTS for this frame
            let pts_secs: f64 = frame_durations_ms[..current_frame_idx]
                .iter()
                .map(|&d| f64::from(d) / 1000.0)
                .sum();

            // Frame pacing: wait until the frame should be displayed
            // Divide by playback_speed: at 2x speed, delay is halved
            if let Some(start_time) = playback_start_time {
                let adjusted_pts = pts_secs / playback_speed;
                let target_time = start_time + std::time::Duration::from_secs_f64(adjusted_pts);
                let now = std::time::Instant::now();

                if target_time > now && is_playing {
                    std::thread::sleep(target_time - now);
                }
            }

            // Send frame event
            let decoded = DecodedFrame {
                rgba_data: Arc::new(rgba_data.clone()),
                width,
                height,
                pts_secs,
            };

            if event_tx
                .blocking_send(DecoderEvent::FrameReady(decoded))
                .is_err()
            {
                // Event channel closed
                break;
            }

            // Advance to next frame
            if is_playing {
                current_frame_idx += 1;
            }

            // Clear single frame flag
            decode_single_frame = false;
        }

        Ok(())
    }

    /// Returns metadata for an animated WebP file.
    ///
    /// This is used to populate `VideoData` when loading the media.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or is not a valid WebP.
    // Allow cast_precision_loss: WebP animations are short - frame count easily fits in f64.
    #[allow(clippy::cast_precision_loss)]
    pub fn get_metadata<P: AsRef<Path>>(webp_path: P) -> Result<WebpMetadata> {
        let webp_data = std::fs::read(webp_path.as_ref())
            .map_err(|e| Error::Io(format!("Failed to read WebP file: {e}")))?;

        let decoder = webp_animation::Decoder::new(&webp_data)
            .map_err(|e| Error::Io(format!("Failed to create WebP decoder: {e:?}")))?;

        let (width, height) = decoder.dimensions();

        // Calculate duration from timestamps
        let mut last_timestamp_ms = 0i32;
        let mut frame_count = 0usize;

        for frame in decoder {
            last_timestamp_ms = frame.timestamp();
            frame_count += 1;
        }

        let duration_secs = f64::from(last_timestamp_ms) / 1000.0;
        let fps = if duration_secs > 0.0 {
            frame_count as f64 / duration_secs
        } else {
            30.0 // Default FPS
        };

        Ok(WebpMetadata {
            width,
            height,
            duration_secs,
            fps,
            frame_count,
        })
    }
}

/// Metadata for an animated WebP file.
#[derive(Debug, Clone)]
pub struct WebpMetadata {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Duration in seconds.
    pub duration_secs: f64,
    /// Frames per second (estimated).
    pub fps: f64,
    /// Total number of frames.
    pub frame_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn metadata_can_be_extracted() {
        let webp_path = "tests/data/test_animated.webp";
        if !std::path::Path::new(webp_path).exists() {
            eprintln!("Test WebP not found, skipping test");
            return;
        }

        let metadata = WebpAnimDecoder::get_metadata(webp_path);
        assert!(metadata.is_ok(), "Failed to get metadata: {metadata:?}");

        let meta = metadata.unwrap();
        assert!(meta.width > 0);
        assert!(meta.height > 0);
        assert!(meta.duration_secs > 0.0);
        assert!(meta.frame_count > 1);
    }

    #[tokio::test]
    async fn decoder_can_be_created() {
        let webp_path = "tests/data/test_animated.webp";
        if !std::path::Path::new(webp_path).exists() {
            eprintln!("Test WebP not found, skipping test");
            return;
        }

        let decoder = WebpAnimDecoder::new(webp_path);
        assert!(decoder.is_ok(), "Failed to create decoder: {decoder:?}");
    }

    #[tokio::test]
    async fn decoder_fails_for_nonexistent_file() {
        let result = WebpAnimDecoder::new("/nonexistent/animation.webp");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn decoder_sends_frames() {
        let webp_path = "tests/data/test_animated.webp";
        if !std::path::Path::new(webp_path).exists() {
            eprintln!("Test WebP not found, skipping test");
            return;
        }

        let mut decoder = WebpAnimDecoder::new(webp_path).unwrap();

        // Send play command
        decoder
            .send_command(DecoderCommand::Play {
                resume_position_secs: None,
            })
            .unwrap();

        // Wait for event (with timeout)
        let event = tokio::time::timeout(Duration::from_millis(500), decoder.recv_event()).await;

        assert!(event.is_ok(), "Timeout waiting for decoder event");
        match event.unwrap() {
            Some(DecoderEvent::Buffering) => {
                // Expected when starting playback
            }
            Some(DecoderEvent::FrameReady(frame)) => {
                assert!(frame.width > 0);
                assert!(frame.height > 0);
                assert!(!frame.rgba_data.is_empty());
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
}
