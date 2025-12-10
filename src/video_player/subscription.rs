// SPDX-License-Identifier: MPL-2.0
//! Iced subscription for video playback events.
//!
//! This module provides an Iced subscription that connects the async decoder
//! to the UI event loop, delivering frames and playback events.
//!
//! Audio is integrated into the playback loop:
//! - AudioDecoder extracts audio samples from the video file
//! - AudioOutput plays the samples through the system audio device
//! - Synchronization uses audio as the master clock

use super::audio::{AudioDecoder, AudioDecoderCommand, AudioDecoderEvent};
use super::audio_output::{AudioOutput, AudioSamples};
use super::frame_cache::CacheConfig;
use super::normalization::{LufsAnalyzer, SharedLufsCache};
use super::webp_decoder::WebpAnimDecoder;
use super::{AsyncDecoder, DecoderCommand, DecoderEvent};
use iced::futures::SinkExt;
use iced::stream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Checks if a path is an animated WebP file.
fn is_animated_webp(path: &PathBuf) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.eq_ignore_ascii_case("webp"))
        .unwrap_or(false)
        && crate::media::detect_media_type(path) == Some(crate::media::MediaType::Video)
}

/// Subscription ID for video playback.
/// Each playback session gets a unique ID to ensure subscriptions are recreated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VideoPlaybackId(u64);

/// Handle for sending commands to the decoder from UI.
/// This is cloneable and can be stored in the VideoPlayer.
#[derive(Clone)]
pub struct DecoderCommandSender {
    video_tx: mpsc::UnboundedSender<DecoderCommand>,
    audio_tx: Option<mpsc::UnboundedSender<AudioDecoderCommand>>,
}

impl DecoderCommandSender {
    /// Sends a command to the video decoder.
    ///
    /// Note: Play/Pause/Seek/Stop commands are forwarded to the audio decoder
    /// internally by the subscription, not through this sender.
    pub fn send(&self, command: DecoderCommand) -> Result<(), String> {
        // Send to video decoder - subscription will forward to audio decoder
        self.video_tx
            .send(command)
            .map_err(|_| "Video decoder not running".to_string())
    }

    /// Sets the audio volume (0.0 to 1.0).
    pub fn set_volume(&self, volume: f32) -> Result<(), String> {
        if let Some(ref audio_tx) = self.audio_tx {
            audio_tx
                .send(AudioDecoderCommand::SetVolume(volume))
                .map_err(|_| "Audio decoder not running".to_string())?;
        }
        Ok(())
    }

    /// Sets the mute state.
    pub fn set_muted(&self, muted: bool) -> Result<(), String> {
        if let Some(ref audio_tx) = self.audio_tx {
            audio_tx
                .send(AudioDecoderCommand::SetMuted(muted))
                .map_err(|_| "Audio decoder not running".to_string())?;
        }
        Ok(())
    }

    /// Returns true if audio is available.
    pub fn has_audio(&self) -> bool {
        self.audio_tx.is_some()
    }
}

impl std::fmt::Debug for DecoderCommandSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DecoderCommandSender")
            .field("has_audio", &self.audio_tx.is_some())
            .finish()
    }
}

/// Messages emitted by the video playback subscription.
#[derive(Debug, Clone)]
pub enum PlaybackMessage {
    /// Subscription started, provides command sender for pause/play/seek.
    Started(DecoderCommandSender),

    /// A new frame is ready for display.
    FrameReady {
        /// RGBA pixel data.
        rgba_data: Arc<Vec<u8>>,
        /// Frame width.
        width: u32,
        /// Frame height.
        height: u32,
        /// Presentation timestamp in seconds.
        pts_secs: f64,
    },

    /// Audio PTS update for sync tracking.
    AudioPts(f64),

    /// Decoder is buffering.
    Buffering,

    /// Playback reached the end.
    EndOfStream,

    /// An error occurred.
    Error(String),
}

/// Shared normalization gain (stored as f32 bits for atomic access).
struct NormalizationGain(AtomicU32);

impl NormalizationGain {
    fn new() -> Self {
        // Default gain = 1.0 (no change)
        Self(AtomicU32::new(1.0f32.to_bits()))
    }

    fn get(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }

    fn set(&self, gain: f32) {
        self.0.store(gain.to_bits(), Ordering::Relaxed);
    }
}

/// Abstraction over different video decoder types (FFmpeg vs WebP).
enum VideoDecoderKind {
    /// FFmpeg-based decoder for regular videos (MP4, AVI, etc.) and animated GIFs.
    Ffmpeg(AsyncDecoder),
    /// WebP-specific decoder for animated WebP files.
    Webp(WebpAnimDecoder),
}

impl VideoDecoderKind {
    /// Sends a command to the underlying decoder.
    fn send_command(&self, command: DecoderCommand) -> crate::error::Result<()> {
        match self {
            VideoDecoderKind::Ffmpeg(dec) => dec.send_command(command),
            VideoDecoderKind::Webp(dec) => dec.send_command(command),
        }
    }

    /// Receives the next event from the underlying decoder.
    async fn recv_event(&mut self) -> Option<DecoderEvent> {
        match self {
            VideoDecoderKind::Ffmpeg(dec) => dec.recv_event().await,
            VideoDecoderKind::Webp(dec) => dec.recv_event().await,
        }
    }
}

/// State of the video playback subscription.
enum State {
    /// Waiting to start.
    Idle,

    /// Decoder is active and we have a command forwarder.
    Decoding {
        video_decoder: VideoDecoderKind,
        audio_decoder: Option<AudioDecoder>,
        audio_output: Option<AudioOutput>,
        external_cmd_rx: mpsc::UnboundedReceiver<DecoderCommand>,
        audio_cmd_rx: Option<mpsc::UnboundedReceiver<AudioDecoderCommand>>,
        /// Normalization gain to apply to audio samples.
        normalization_gain: Arc<NormalizationGain>,
    },
}

/// Creates a video playback subscription.
///
/// This subscription manages the async decoder lifecycle and translates
/// decoder events into Iced messages.
///
/// The session_id ensures each playback session gets a unique subscription ID,
/// allowing the subscription to be recreated when playback restarts.
///
/// The subscription sends a `Started` message with a `DecoderCommandSender` that
/// can be used to send pause/play/seek commands to the decoder.
///
/// Audio playback is automatically integrated if the video has an audio track.
///
/// If `lufs_cache` is provided, audio normalization will be applied to level
/// volume between different media files.
///
/// The `cache_config` parameter controls frame caching for optimized seek
/// performance. Use `CacheConfig::default()` for standard caching.
pub fn video_playback(
    video_path: PathBuf,
    session_id: u64,
    lufs_cache: Option<SharedLufsCache>,
    normalization_enabled: bool,
    cache_config: CacheConfig,
) -> iced::Subscription<PlaybackMessage> {
    // Note: cache_config changes take effect on next video load, not immediately
    // This avoids restarting playback when user changes cache size in settings
    iced::Subscription::run_with_id(
        VideoPlaybackId(session_id),
        stream::channel(100, move |mut output| async move {
            let mut state = State::Idle;

            loop {
                match &mut state {
                    State::Idle => {
                        // Create external command channels for UI to send commands
                        let (external_cmd_tx, external_cmd_rx) = mpsc::unbounded_channel();
                        let (audio_cmd_tx, audio_cmd_rx) = mpsc::unbounded_channel();

                        // Check if this is an animated WebP (requires special decoder)
                        let use_webp_decoder = is_animated_webp(&video_path);

                        // Try to create video decoder
                        let video_decoder: VideoDecoderKind = if use_webp_decoder {
                            // Use WebP decoder for animated WebP files
                            match WebpAnimDecoder::new(&video_path) {
                                Ok(decoder) => VideoDecoderKind::Webp(decoder),
                                Err(e) => {
                                    let _ =
                                        output.send(PlaybackMessage::Error(e.to_string())).await;
                                    break;
                                }
                            }
                        } else {
                            // Use FFmpeg decoder for regular videos
                            match AsyncDecoder::new(&video_path, cache_config) {
                                Ok(decoder) => VideoDecoderKind::Ffmpeg(decoder),
                                Err(e) => {
                                    let _ =
                                        output.send(PlaybackMessage::Error(e.to_string())).await;
                                    break;
                                }
                            }
                        };

                        // Try to create audio decoder (optional - video might not have audio)
                        // Note: WebP animations don't have audio, so skip for them
                        let audio_decoder = if use_webp_decoder {
                            None
                        } else {
                            match AudioDecoder::new(&video_path) {
                                Ok(Some(decoder)) => Some(decoder),
                                Ok(None) => {
                                    // No audio stream in video - this is fine
                                    None
                                }
                                Err(e) => {
                                    // Log error but continue without audio
                                    eprintln!("Audio decoder failed: {}", e);
                                    None
                                }
                            }
                        };

                        // Create audio output if we have audio
                        let audio_output = if audio_decoder.is_some() {
                            match AudioOutput::new(0.8) {
                                Ok(output) => Some(output),
                                Err(e) => {
                                    eprintln!("Audio output failed: {}", e);
                                    None
                                }
                            }
                        } else {
                            None
                        };

                        // Create normalization gain
                        let normalization_gain = Arc::new(NormalizationGain::new());

                        // Launch LUFS analysis in background if normalization is enabled
                        if normalization_enabled && audio_decoder.is_some() {
                            let gain_clone = Arc::clone(&normalization_gain);
                            let path_clone = video_path.clone();
                            let cache_clone = lufs_cache.clone();

                            tokio::task::spawn_blocking(move || {
                                // Check cache first
                                let path_str = path_clone.to_string_lossy().to_string();
                                if let Some(ref cache) = cache_clone {
                                    if let Some(cached_lufs) = cache.get(&path_str) {
                                        let analyzer = LufsAnalyzer::default();
                                        let gain_db = analyzer.calculate_gain(cached_lufs);
                                        let gain_linear = LufsAnalyzer::db_to_linear(gain_db);
                                        gain_clone.set(gain_linear as f32);
                                        return;
                                    }
                                }

                                // Analyze LUFS (this is slow, ~1-5 seconds)
                                let analyzer = LufsAnalyzer::default();
                                match analyzer.analyze_file(&path_clone) {
                                    Ok(measured_lufs) => {
                                        // Cache the result
                                        if let Some(ref cache) = cache_clone {
                                            cache.insert(path_str, measured_lufs);
                                        }

                                        // Calculate and apply gain
                                        let gain_db = analyzer.calculate_gain(measured_lufs);
                                        let gain_linear = LufsAnalyzer::db_to_linear(gain_db);
                                        gain_clone.set(gain_linear as f32);
                                    }
                                    Err(_) => {
                                        // Keep default gain of 1.0
                                    }
                                }
                            });
                        }

                        // Send the command sender to UI
                        let cmd_sender = DecoderCommandSender {
                            video_tx: external_cmd_tx,
                            audio_tx: if audio_decoder.is_some() {
                                Some(audio_cmd_tx)
                            } else {
                                None
                            },
                        };
                        let _ = output.send(PlaybackMessage::Started(cmd_sender)).await;

                        let has_audio = audio_decoder.is_some();
                        state = State::Decoding {
                            video_decoder,
                            audio_decoder,
                            audio_output,
                            external_cmd_rx,
                            audio_cmd_rx: if has_audio { Some(audio_cmd_rx) } else { None },
                            normalization_gain,
                        };
                    }

                    State::Decoding {
                        video_decoder,
                        audio_decoder,
                        audio_output,
                        external_cmd_rx,
                        audio_cmd_rx,
                        normalization_gain,
                    } => {
                        // Use select to handle commands, video events, and audio events
                        tokio::select! {
                            // Check for external commands from UI
                            cmd = external_cmd_rx.recv() => {
                                if let Some(command) = cmd {
                                    // Handle audio output commands
                                    if let Some(ref audio_out) = audio_output {
                                        match &command {
                                            DecoderCommand::Pause => {
                                                let _ = audio_out.pause();
                                            }
                                            DecoderCommand::Play => {
                                                let _ = audio_out.resume();
                                            }
                                            DecoderCommand::Stop => {
                                                let _ = audio_out.stop();
                                            }
                                            DecoderCommand::Seek { .. } => {
                                                // Clear audio buffer on seek
                                                let _ = audio_out.stop();
                                            }
                                        }
                                    }

                                    // Forward command to audio decoder for timing sync
                                    if let Some(ref audio_dec) = audio_decoder {
                                        let audio_cmd = match &command {
                                            DecoderCommand::Play => AudioDecoderCommand::Play,
                                            DecoderCommand::Pause => AudioDecoderCommand::Pause,
                                            DecoderCommand::Seek { target_secs } => {
                                                AudioDecoderCommand::Seek { target_secs: *target_secs }
                                            }
                                            DecoderCommand::Stop => AudioDecoderCommand::Stop,
                                        };
                                        let _ = audio_dec.send_command(audio_cmd);
                                    }

                                    // Forward command to video decoder
                                    if let Err(e) = video_decoder.send_command(command) {
                                        let _ = output.send(PlaybackMessage::Error(e.to_string())).await;
                                    }
                                }
                            }

                            // Check for audio commands from UI (volume, mute, play, pause, seek, stop)
                            Some(audio_cmd) = async {
                                if let Some(ref mut rx) = audio_cmd_rx {
                                    rx.recv().await
                                } else {
                                    std::future::pending::<Option<AudioDecoderCommand>>().await
                                }
                            } => {
                                // Forward to audio decoder
                                if let Some(ref audio_dec) = audio_decoder {
                                    let _ = audio_dec.send_command(audio_cmd.clone());
                                }

                                // Handle audio output commands
                                if let Some(ref audio_out) = audio_output {
                                    match audio_cmd {
                                        AudioDecoderCommand::Play => {
                                            let _ = audio_out.resume();
                                        }
                                        AudioDecoderCommand::Pause => {
                                            let _ = audio_out.pause();
                                        }
                                        AudioDecoderCommand::Seek { .. } => {
                                            // Clear buffer and pause on seek, decoder will restart
                                            let _ = audio_out.stop();
                                        }
                                        AudioDecoderCommand::Stop => {
                                            let _ = audio_out.stop();
                                        }
                                        AudioDecoderCommand::SetVolume(vol) => {
                                            let _ = audio_out.set_volume(vol);
                                        }
                                        AudioDecoderCommand::SetMuted(muted) => {
                                            let _ = audio_out.set_muted(muted);
                                        }
                                    }
                                }
                            }

                            // Check for events from video decoder
                            event = video_decoder.recv_event() => {
                                if let Some(event) = event {
                                    let message = match event {
                                        DecoderEvent::FrameReady(frame) => PlaybackMessage::FrameReady {
                                            rgba_data: frame.rgba_data,
                                            width: frame.width,
                                            height: frame.height,
                                            pts_secs: frame.pts_secs,
                                        },
                                        DecoderEvent::Buffering => PlaybackMessage::Buffering,
                                        DecoderEvent::EndOfStream => PlaybackMessage::EndOfStream,
                                        DecoderEvent::Error(msg) => PlaybackMessage::Error(msg),
                                    };

                                    let _ = output.send(message).await;
                                } else {
                                    // Decoder closed, exit loop
                                    break;
                                }
                            }

                            // Check for events from audio decoder
                            Some(audio_event) = async {
                                if let Some(ref mut audio_dec) = audio_decoder {
                                    audio_dec.recv_event().await
                                } else {
                                    std::future::pending::<Option<AudioDecoderEvent>>().await
                                }
                            } => {
                                match audio_event {
                                    AudioDecoderEvent::BufferReady(audio) => {
                                        // Send audio samples to output with normalization gain
                                        if let Some(ref audio_out) = audio_output {
                                            let gain = normalization_gain.get();

                                            // Apply normalization gain if not 1.0
                                            let samples: AudioSamples = if (gain - 1.0).abs() > 0.001 {
                                                // Apply gain to samples
                                                let normalized: Vec<f32> = audio
                                                    .samples
                                                    .iter()
                                                    .map(|s| (s * gain).clamp(-1.0, 1.0))
                                                    .collect();
                                                Arc::new(normalized)
                                            } else {
                                                audio.samples
                                            };

                                            let _ = audio_out.play(samples);
                                        }

                                        // Send PTS update for sync tracking
                                        let _ = output.send(PlaybackMessage::AudioPts(audio.pts_secs)).await;
                                    }
                                    AudioDecoderEvent::StreamInfo(_info) => {
                                        // Stream info available if needed for UI display
                                    }
                                    AudioDecoderEvent::EndOfStream => {
                                        // Audio finished - video might still be playing
                                    }
                                    AudioDecoderEvent::Error(msg) => {
                                        eprintln!("Audio error: {}", msg);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Keep subscription alive but idle
            std::future::pending::<()>().await;
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playback_message_can_be_cloned() {
        let msg = PlaybackMessage::Buffering;
        let cloned = msg.clone();
        assert!(matches!(cloned, PlaybackMessage::Buffering));
    }

    #[test]
    fn playback_message_can_be_debugged() {
        let msg = PlaybackMessage::Error("test error".to_string());
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("test error"));
    }

    #[test]
    fn subscription_id_is_consistent() {
        let id1 = VideoPlaybackId(42);
        let id2 = VideoPlaybackId(42);
        assert_eq!(id1, id2);

        // Different session IDs should be different
        let id3 = VideoPlaybackId(43);
        assert_ne!(id1, id3);
    }

    #[test]
    fn audio_pts_message_can_be_created() {
        let msg = PlaybackMessage::AudioPts(10.5);
        assert!(matches!(msg, PlaybackMessage::AudioPts(pts) if (pts - 10.5).abs() < 0.001));
    }
}
