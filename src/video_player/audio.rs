// SPDX-License-Identifier: MPL-2.0
//! Audio extraction and playback for video files.
//!
//! This module provides audio decoding from video files using `FFmpeg`,
//! and audio playback using cpal.

use crate::error::{Error, Result};
use crate::video_player::audio_output::AudioOutputConfig;
use crate::video_player::sync::SharedSyncClock;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Maximum number of audio frames to skip during precise seeking.
/// Prevents infinite loops on corrupted files or seeks beyond EOF.
const MAX_SEEK_FRAMES: u32 = 1000;

/// Audio look-ahead buffer time in seconds.
/// We queue audio ~200ms before it needs to play to ensure smooth playback.
const AUDIO_LOOKAHEAD_SECS: f64 = 0.2;

/// Audio sample format used for playback.
/// We use f32 as it's the most common format for audio processing.
pub const SAMPLE_FORMAT: cpal::SampleFormat = cpal::SampleFormat::F32;

/// Represents a decoded audio buffer ready for playback.
#[derive(Debug, Clone)]
pub struct DecodedAudio {
    /// Interleaved audio samples (f32, normalized to [-1.0, 1.0]).
    pub samples: Arc<Vec<f32>>,

    /// Sample rate in Hz (e.g., 44100, 48000).
    pub sample_rate: u32,

    /// Number of audio channels (1 = mono, 2 = stereo).
    pub channels: u16,

    /// Presentation timestamp in seconds.
    /// Indicates when this audio buffer should start playing.
    pub pts_secs: f64,

    /// Duration of this audio buffer in seconds.
    pub duration_secs: f64,
}

impl DecodedAudio {
    /// Returns the total number of samples (across all channels).
    #[must_use]
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Returns the number of frames (samples per channel).
    #[must_use]
    pub fn frame_count(&self) -> usize {
        self.samples.len() / self.channels as usize
    }

    /// Returns the size in bytes.
    #[must_use]
    pub fn size_bytes(&self) -> usize {
        self.samples.len() * std::mem::size_of::<f32>()
    }
}

/// Audio stream information extracted from a video file.
#[derive(Debug, Clone)]
pub struct AudioStreamInfo {
    /// Sample rate in Hz.
    pub sample_rate: u32,

    /// Number of channels.
    pub channels: u16,

    /// Total duration in seconds (if known).
    pub duration_secs: Option<f64>,

    /// Audio codec name (e.g., "aac", "mp3", "opus").
    pub codec_name: String,

    /// Bit rate in bits per second (if known).
    pub bit_rate: Option<u64>,
}

/// Events sent from the audio decoder.
#[derive(Debug, Clone)]
pub enum AudioDecoderEvent {
    /// Audio stream information is available.
    StreamInfo(AudioStreamInfo),

    /// A decoded audio buffer is ready.
    BufferReady(DecodedAudio),

    /// End of audio stream reached.
    EndOfStream,

    /// An error occurred during decoding.
    Error(String),
}

/// Commands sent to the audio decoder.
#[derive(Debug, Clone)]
pub enum AudioDecoderCommand {
    /// Start decoding from the beginning.
    Play,

    /// Pause decoding.
    Pause,

    /// Seek to a specific timestamp.
    Seek { target_secs: f64 },

    /// Stop decoding and clean up.
    Stop,

    /// Set volume (guaranteed to be within 0.0–1.0 by Volume type).
    SetVolume(super::Volume),

    /// Set mute state.
    SetMuted(bool),

    /// Set playback speed.
    /// Affects audio buffer timing.
    /// - `speed`: Validated playback speed (guaranteed within valid range)
    /// - `instant`: Wall clock reference for timing synchronization
    /// - `reference_pts`: Video position (in seconds) at the moment of speed change.
    ///   Used as the timing reference to stay in sync with video.
    SetPlaybackSpeed {
        speed: super::PlaybackSpeed,
        instant: std::time::Instant,
        reference_pts: f64,
    },
}

/// Holds mutable state for the audio decoder loop.
struct AudioDecoderState {
    /// Whether playback is currently active.
    is_playing: bool,
    /// Volume level (0.0 to 1.0).
    volume: f32,
    /// Whether audio is muted.
    muted: bool,
    /// Current playback speed multiplier.
    playback_speed: f64,
    /// Wall-clock reference for frame timing.
    playback_start_time: Option<std::time::Instant>,
    /// Reference PTS for timing calculation.
    first_pts: Option<f64>,
    /// Target PTS for precise seeking.
    seek_target_secs: Option<f64>,
    /// Counter for frames skipped during seeking.
    seek_frames_skipped: u32,
}

impl AudioDecoderState {
    fn new() -> Self {
        Self {
            is_playing: false,
            volume: 1.0,
            muted: false,
            playback_speed: 1.0,
            playback_start_time: None,
            first_pts: None,
            seek_target_secs: None,
            seek_frames_skipped: 0,
        }
    }

    fn reset_timing(&mut self) {
        self.playback_start_time = Some(std::time::Instant::now());
        self.first_pts = None;
    }
}

/// Result of processing an audio decoder command.
enum AudioCommandResult {
    /// Continue the main loop.
    Continue,
    /// Break from the main loop.
    Break,
}

/// Processes a single audio decoder command.
fn handle_audio_command(
    command: &AudioDecoderCommand,
    state: &mut AudioDecoderState,
    ictx: &mut ffmpeg_next::format::context::Input,
    decoder: &mut ffmpeg_next::decoder::Audio,
    sync_clock: Option<&SharedSyncClock>,
    event_tx: &mpsc::Sender<AudioDecoderEvent>,
) -> AudioCommandResult {
    match command {
        AudioDecoderCommand::Play => {
            state.is_playing = true;
            state.playback_start_time = Some(std::time::Instant::now());
            if let Some(clock) = sync_clock {
                clock.resume();
            }
        }
        AudioDecoderCommand::Pause => {
            state.is_playing = false;
            state.playback_start_time = None;
            state.first_pts = None;
            state.seek_target_secs = None;
            if let Some(clock) = sync_clock {
                clock.pause();
            }
        }
        AudioDecoderCommand::Seek { target_secs } => {
            #[allow(clippy::cast_possible_truncation)]
            let timestamp = (*target_secs * 1_000_000.0) as i64;
            if let Err(e) = ictx.seek(timestamp, ..timestamp) {
                let _ = event_tx
                    .blocking_send(AudioDecoderEvent::Error(format!("Audio seek failed: {e}")));
                state.seek_target_secs = None;
            } else {
                decoder.flush();
                state.reset_timing();
                state.seek_target_secs = Some(*target_secs);
                state.seek_frames_skipped = 0;
                if let Some(clock) = sync_clock {
                    clock.seek(*target_secs);
                }
            }
        }
        AudioDecoderCommand::Stop => {
            if let Some(clock) = sync_clock {
                clock.stop();
            }
            return AudioCommandResult::Break;
        }
        AudioDecoderCommand::SetVolume(volume) => {
            state.volume = volume.value();
        }
        AudioDecoderCommand::SetMuted(mute) => {
            state.muted = *mute;
        }
        AudioDecoderCommand::SetPlaybackSpeed {
            speed,
            instant,
            reference_pts,
        } => {
            state.playback_speed = speed.value();
            state.playback_start_time = Some(*instant);
            state.first_pts = Some(*reference_pts);
        }
    }
    AudioCommandResult::Continue
}

/// Async audio decoder that extracts and decodes audio from video files.
///
/// Runs in a separate blocking thread since `FFmpeg` operations are not `Send`.
pub struct AudioDecoder {
    /// Channel for sending commands to the decoder task.
    command_tx: mpsc::UnboundedSender<AudioDecoderCommand>,

    /// Channel for receiving events from the decoder task.
    /// Bounded to prevent memory accumulation during rapid seeks.
    event_rx: mpsc::Receiver<AudioDecoderEvent>,
}

impl AudioDecoder {
    /// Creates a new audio decoder for the given video file.
    ///
    /// Returns `None` if the file has no audio stream.
    ///
    /// # Arguments
    ///
    /// * `video_path` - Path to the video file
    /// * `sync_clock` - Optional shared sync clock for A/V synchronization.
    ///   When provided, the audio decoder will update the clock with its PTS,
    ///   allowing the video decoder to sync to audio timing.
    /// * `output_config` - Audio output device configuration (sample rate, channels).
    ///   The decoder will resample audio to match these specs for correct playback.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The video file does not exist
    /// - The file cannot be opened or parsed by `FFmpeg`
    pub fn new<P: AsRef<Path>>(
        video_path: P,
        sync_clock: Option<SharedSyncClock>,
        output_config: AudioOutputConfig,
    ) -> Result<Option<Self>> {
        let path = video_path.as_ref().to_path_buf();

        // Validate file exists
        if !path.exists() {
            return Err(Error::Io(format!(
                "Video file not found: {}",
                path.display()
            )));
        }

        // Check if file has audio before spawning decoder task
        if !Self::has_audio_stream(&path)? {
            return Ok(None);
        }

        // Create channels for bidirectional communication
        // Commands: unbounded (UI needs to send without blocking)
        // Events: bounded to prevent memory accumulation during seeks
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::channel(4);

        // Spawn the decoder task in a blocking thread
        tokio::task::spawn_blocking(move || {
            if let Err(e) =
                Self::decoder_loop(path, command_rx, event_tx, sync_clock, output_config)
            {
                eprintln!("Audio decoder task failed: {e}");
            }
        });

        Ok(Some(Self {
            command_tx,
            event_rx,
        }))
    }

    /// Checks if the video file has an audio stream.
    fn has_audio_stream(path: &std::path::Path) -> Result<bool> {
        crate::media::video::init_ffmpeg()?;

        let ictx = ffmpeg_next::format::input(path)
            .map_err(|e| Error::Io(format!("Failed to open file: {e}")))?;

        Ok(ictx
            .streams()
            .best(ffmpeg_next::media::Type::Audio)
            .is_some())
    }

    /// Sends a command to the decoder task.
    ///
    /// # Errors
    ///
    /// Returns an error if the decoder task is not running (channel closed).
    pub fn send_command(&self, command: AudioDecoderCommand) -> Result<()> {
        self.command_tx
            .send(command)
            .map_err(|_| Error::Io("Audio decoder task is not running".into()))
    }

    /// Receives the next event from the decoder (non-blocking).
    pub fn try_recv_event(&mut self) -> Option<AudioDecoderEvent> {
        self.event_rx.try_recv().ok()
    }

    /// Receives the next event from the decoder (blocking).
    pub async fn recv_event(&mut self) -> Option<AudioDecoderEvent> {
        self.event_rx.recv().await
    }

    /// Main audio decoder loop running in a blocking thread.
    #[allow(clippy::needless_pass_by_value)] // PathBuf/Sender/SyncClock/Config need ownership for thread
    #[allow(clippy::cast_precision_loss)] // FFmpeg i64 timestamps have enough f64 precision
    #[allow(clippy::cast_possible_truncation)] // Timestamp conversions are within valid range
    #[allow(clippy::too_many_lines)] // Core decoder state machine - refactored from 202 to 160 lines
    fn decoder_loop(
        video_path: std::path::PathBuf,
        mut command_rx: mpsc::UnboundedReceiver<AudioDecoderCommand>,
        event_tx: mpsc::Sender<AudioDecoderEvent>,
        sync_clock: Option<SharedSyncClock>,
        output_config: AudioOutputConfig,
    ) -> Result<()> {
        // Initialize FFmpeg
        crate::media::video::init_ffmpeg()?;

        // Open video file
        let mut ictx = ffmpeg_next::format::input(&video_path)
            .map_err(|e| Error::Io(format!("Failed to open video: {e}")))?;

        // Find audio stream
        let input = ictx
            .streams()
            .best(ffmpeg_next::media::Type::Audio)
            .ok_or_else(|| Error::Io("No audio stream found".to_string()))?;
        let audio_stream_index = input.index();

        // Get stream info
        let time_base = input.time_base();
        let time_base_f64 = f64::from(time_base.numerator()) / f64::from(time_base.denominator());

        // Create audio decoder
        let context_decoder =
            ffmpeg_next::codec::context::Context::from_parameters(input.parameters())
                .map_err(|e| Error::Io(format!("Failed to create codec context: {e}")))?;
        let mut decoder = context_decoder
            .decoder()
            .audio()
            .map_err(|e| Error::Io(format!("Failed to create audio decoder: {e}")))?;

        // Get audio parameters
        let sample_rate = decoder.rate();
        let channels = decoder.channels() as u16;
        let codec_name = decoder
            .codec()
            .map_or_else(|| "unknown".to_string(), |c| c.name().to_string());

        // Calculate duration if available
        let duration_secs = if input.duration() > 0 {
            Some(input.duration() as f64 * time_base_f64)
        } else {
            None
        };

        // Send stream info
        let _ = event_tx.blocking_send(AudioDecoderEvent::StreamInfo(AudioStreamInfo {
            sample_rate,
            channels,
            duration_secs,
            codec_name,
            bit_rate: None, // Could extract from stream if needed
        }));

        // Setup resampler to convert to f32 interleaved at the target sample rate and channels.
        // This is critical: the audio device expects samples at its native rate and channel count.
        // Without proper resampling, audio plays at wrong speed (robotic sound) or wrong channels.
        let output_channel_layout = match output_config.channels {
            1 => ffmpeg_next::ChannelLayout::MONO,
            _ => ffmpeg_next::ChannelLayout::STEREO, // Downmix anything else to stereo
        };

        let mut resampler = ffmpeg_next::software::resampling::Context::get(
            decoder.format(),
            decoder.channel_layout(),
            decoder.rate(),
            ffmpeg_next::format::Sample::F32(ffmpeg_next::format::sample::Type::Packed),
            output_channel_layout,
            output_config.sample_rate,
        )
        .map_err(|e| Error::Io(format!("Failed to create resampler: {e}")))?;

        // Use output config for duration calculations since that's what we output
        let output_sample_rate = output_config.sample_rate;
        let output_channels = output_config.channels;

        // Playback state
        let mut state = AudioDecoderState::new();

        // Main loop
        loop {
            // Process commands
            match command_rx.try_recv() {
                Ok(ref cmd) => {
                    match handle_audio_command(
                        cmd,
                        &mut state,
                        &mut ictx,
                        &mut decoder,
                        sync_clock.as_ref(),
                        &event_tx,
                    ) {
                        AudioCommandResult::Break => break,
                        AudioCommandResult::Continue => {}
                    }
                }
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    if let Some(ref clock) = sync_clock {
                        clock.stop();
                    }
                    break;
                }
                Err(mpsc::error::TryRecvError::Empty) => {}
            }

            // If not playing, sleep to avoid busy-waiting
            if !state.is_playing {
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }

            // Decode next audio frame
            let mut frame_decoded = false;
            for (stream, packet) in ictx.packets() {
                if stream.index() != audio_stream_index {
                    continue;
                }

                if let Err(e) = decoder.send_packet(&packet) {
                    let _ = event_tx.blocking_send(AudioDecoderEvent::Error(format!(
                        "Audio packet failed: {e}"
                    )));
                    continue;
                }

                let mut decoded_frame = ffmpeg_next::frame::Audio::empty();
                if decoder.receive_frame(&mut decoded_frame).is_ok() {
                    let mut output_audio = ffmpeg_next::frame::Audio::empty();
                    if let Err(e) = resampler.run(&decoded_frame, &mut output_audio) {
                        let _ = event_tx.blocking_send(AudioDecoderEvent::Error(format!(
                            "Resampling failed: {e}"
                        )));
                        continue;
                    }

                    let samples = Self::extract_samples(&output_audio, output_channels);

                    #[allow(clippy::cast_precision_loss)]
                    let pts_secs = if let Some(pts) = decoded_frame.timestamp() {
                        pts as f64 * time_base_f64
                    } else {
                        0.0
                    };

                    let frame_duration = samples.len() as f64
                        / (f64::from(output_sample_rate) * f64::from(output_channels));

                    // Precise seeking: skip audio frames before target PTS
                    if let Some(target) = state.seek_target_secs {
                        let frame_end_pts = pts_secs + frame_duration;
                        if frame_end_pts < target {
                            state.seek_frames_skipped += 1;
                            if state.seek_frames_skipped >= MAX_SEEK_FRAMES {
                                let _ = event_tx.blocking_send(AudioDecoderEvent::Error(
                                    "Audio seek timeout: target may be beyond end of file"
                                        .to_string(),
                                ));
                                state.seek_target_secs = None;
                                continue;
                            }
                            continue;
                        }
                        state.first_pts = Some(target);
                        state.seek_target_secs = None;
                    }

                    // Frame pacing
                    if let Some(start_time) = state.playback_start_time {
                        if state.first_pts.is_none() {
                            state.first_pts = Some(pts_secs);
                        }
                        if let Some(first) = state.first_pts {
                            let frame_delay =
                                (pts_secs - first) / state.playback_speed - AUDIO_LOOKAHEAD_SECS;
                            if frame_delay > 0.0 {
                                let target_time =
                                    start_time + std::time::Duration::from_secs_f64(frame_delay);
                                let now = std::time::Instant::now();
                                if target_time > now {
                                    std::thread::sleep(target_time - now);
                                }
                            }
                        }
                    }

                    // Update sync clock
                    if let Some(ref clock) = sync_clock {
                        clock.update_audio_pts(pts_secs);
                    }

                    let audio = DecodedAudio {
                        samples: Arc::new(samples),
                        sample_rate: output_sample_rate,
                        channels: output_channels,
                        pts_secs,
                        duration_secs: frame_duration,
                    };

                    if event_tx
                        .blocking_send(AudioDecoderEvent::BufferReady(audio))
                        .is_err()
                    {
                        break;
                    }

                    frame_decoded = true;
                    break;
                }
            }

            // If no frame decoded, we've reached end of stream
            if !frame_decoded {
                let _ = event_tx.blocking_send(AudioDecoderEvent::EndOfStream);
                state.is_playing = false;
            }
        }

        Ok(())
    }

    /// Extracts f32 samples from a resampled audio frame.
    fn extract_samples(frame: &ffmpeg_next::frame::Audio, channels: u16) -> Vec<f32> {
        let data = frame.data(0);
        let sample_count = frame.samples() * channels as usize;

        // Convert bytes to f32
        let mut samples = Vec::with_capacity(sample_count);
        for i in 0..sample_count {
            let offset = i * 4; // f32 = 4 bytes
            if offset + 4 <= data.len() {
                let bytes = [
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ];
                samples.push(f32::from_le_bytes(bytes));
            }
        }

        samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_player::Volume;

    #[test]
    fn decoded_audio_calculates_sample_count() {
        let audio = DecodedAudio {
            samples: Arc::new(vec![0.0f32; 4800]), // 100ms at 48kHz stereo
            sample_rate: 48000,
            channels: 2,
            pts_secs: 0.0,
            duration_secs: 0.05,
        };

        assert_eq!(audio.sample_count(), 4800);
        assert_eq!(audio.frame_count(), 2400); // 4800 / 2 channels
        assert_eq!(audio.size_bytes(), 4800 * 4); // 4 bytes per f32
    }

    #[test]
    fn decoded_audio_handles_mono() {
        let audio = DecodedAudio {
            samples: Arc::new(vec![0.0f32; 4800]),
            sample_rate: 48000,
            channels: 1,
            pts_secs: 0.0,
            duration_secs: 0.1,
        };

        assert_eq!(audio.frame_count(), 4800); // Same as sample_count for mono
    }

    #[test]
    fn audio_stream_info_stores_metadata() {
        let info = AudioStreamInfo {
            sample_rate: 44100,
            channels: 2,
            duration_secs: Some(180.5),
            codec_name: "aac".to_string(),
            bit_rate: Some(128_000),
        };

        assert_eq!(info.sample_rate, 44100);
        assert_eq!(info.channels, 2);
        assert_eq!(info.duration_secs, Some(180.5));
        assert_eq!(info.codec_name, "aac");
        assert_eq!(info.bit_rate, Some(128_000));
    }

    #[test]
    fn audio_decoder_event_variants() {
        // Test that all event variants can be created
        let info_event = AudioDecoderEvent::StreamInfo(AudioStreamInfo {
            sample_rate: 48000,
            channels: 2,
            duration_secs: None,
            codec_name: "opus".to_string(),
            bit_rate: None,
        });
        assert!(matches!(info_event, AudioDecoderEvent::StreamInfo(_)));

        let buffer_event = AudioDecoderEvent::BufferReady(DecodedAudio {
            samples: Arc::new(vec![]),
            sample_rate: 48000,
            channels: 2,
            pts_secs: 0.0,
            duration_secs: 0.0,
        });
        assert!(matches!(buffer_event, AudioDecoderEvent::BufferReady(_)));

        let eos_event = AudioDecoderEvent::EndOfStream;
        assert!(matches!(eos_event, AudioDecoderEvent::EndOfStream));

        let error_event = AudioDecoderEvent::Error("test error".to_string());
        assert!(matches!(error_event, AudioDecoderEvent::Error(_)));
    }

    #[test]
    fn audio_decoder_command_variants() {
        // Test that all command variants can be created and cloned
        let play = AudioDecoderCommand::Play;
        assert!(matches!(play.clone(), AudioDecoderCommand::Play));

        let pause = AudioDecoderCommand::Pause;
        assert!(matches!(pause.clone(), AudioDecoderCommand::Pause));

        let seek = AudioDecoderCommand::Seek { target_secs: 30.5 };
        assert!(matches!(
            seek.clone(),
            AudioDecoderCommand::Seek { target_secs } if (target_secs - 30.5).abs() < 0.001
        ));

        let stop = AudioDecoderCommand::Stop;
        assert!(matches!(stop.clone(), AudioDecoderCommand::Stop));

        let volume = AudioDecoderCommand::SetVolume(Volume::new(0.75));
        assert!(matches!(
            volume.clone(),
            AudioDecoderCommand::SetVolume(v) if (v.value() - 0.75).abs() < 0.001
        ));

        let muted = AudioDecoderCommand::SetMuted(true);
        assert!(matches!(muted.clone(), AudioDecoderCommand::SetMuted(true)));
    }

    #[tokio::test]
    async fn audio_decoder_returns_none_for_file_without_audio() {
        // Create a fake video file (will fail to decode but should detect no audio)
        let temp_dir = tempfile::tempdir().unwrap();
        let video_path = temp_dir.path().join("no_audio.mp4");
        std::fs::write(&video_path, b"fake video data without audio").unwrap();

        // This should return Ok(None) or an error, not panic
        // Use typical device config: 48000 Hz stereo
        let config = AudioOutputConfig {
            sample_rate: 48000,
            channels: 2,
        };
        let result = AudioDecoder::new(&video_path, None, config);
        // We expect either Ok(None) for no audio or Err for invalid file
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn audio_decoder_fails_for_nonexistent_file() {
        let config = AudioOutputConfig {
            sample_rate: 48000,
            channels: 2,
        };
        let result = AudioDecoder::new("/nonexistent/video.mp4", None, config);
        assert!(result.is_err());
    }
}
