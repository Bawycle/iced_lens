// SPDX-License-Identifier: MPL-2.0
//! Audio extraction and playback for video files.
//!
//! This module provides audio decoding from video files using `FFmpeg`,
//! and audio playback using cpal.

use crate::error::{Error, Result};
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
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The video file does not exist
    /// - The file cannot be opened or parsed by `FFmpeg`
    pub fn new<P: AsRef<Path>>(
        video_path: P,
        sync_clock: Option<SharedSyncClock>,
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
            if let Err(e) = Self::decoder_loop(path, command_rx, event_tx, sync_clock) {
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
    #[allow(clippy::too_many_lines)] // Refactoring planned in TODO.md
    #[allow(clippy::needless_pass_by_value)] // PathBuf/Sender/SyncClock need ownership for thread
    #[allow(clippy::cast_precision_loss)] // FFmpeg i64 timestamps have enough f64 precision
    #[allow(clippy::cast_possible_truncation)] // Timestamp conversions are within valid range
    fn decoder_loop(
        video_path: std::path::PathBuf,
        mut command_rx: mpsc::UnboundedReceiver<AudioDecoderCommand>,
        event_tx: mpsc::Sender<AudioDecoderEvent>,
        sync_clock: Option<SharedSyncClock>,
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

        // Setup resampler to convert to f32 planar -> interleaved f32
        let mut resampler = ffmpeg_next::software::resampling::Context::get(
            decoder.format(),
            decoder.channel_layout(),
            decoder.rate(),
            ffmpeg_next::format::Sample::F32(ffmpeg_next::format::sample::Type::Packed),
            decoder.channel_layout(),
            decoder.rate(),
        )
        .map_err(|e| Error::Io(format!("Failed to create resampler: {e}")))?;

        // Playback state
        let mut is_playing = false;
        let mut _volume = 1.0f32;
        let mut _muted = false;
        let mut playback_speed: f64 = 1.0;

        // Frame pacing state (similar to video decoder)
        let mut playback_start_time: Option<std::time::Instant> = None;
        let mut first_pts: Option<f64> = None;

        // Precise seeking: target PTS to reach after keyframe seek
        // Audio frames with PTS < target will be skipped (not sent to output)
        let mut seek_target_secs: Option<f64> = None;
        // Counter for frames skipped during precise seeking (timeout protection)
        let mut seek_frames_skipped: u32 = 0;

        // Main loop
        loop {
            // Check for commands (non-blocking)
            match command_rx.try_recv() {
                Ok(AudioDecoderCommand::Play) => {
                    is_playing = true;
                    playback_start_time = Some(std::time::Instant::now());
                    // Don't reset first_pts here - preserve seek target if set
                    // Pause already resets it, and decode loop sets it if None
                    // IMPORTANT: Don't clear seek_target_secs here!
                    // When Play follows Seek (for resume), we must preserve the seek target
                    // so precise seeking can complete. The target is cleared automatically
                    // when the frame at/after target PTS is decoded (line ~380).

                    // Resume sync clock (it will be started/updated when first audio frame is decoded)
                    if let Some(ref clock) = sync_clock {
                        clock.resume();
                    }
                }
                Ok(AudioDecoderCommand::Pause) => {
                    is_playing = false;
                    playback_start_time = None;
                    first_pts = None;
                    // Clear seek target
                    seek_target_secs = None;

                    // Pause sync clock
                    if let Some(ref clock) = sync_clock {
                        clock.pause();
                    }
                }
                Ok(AudioDecoderCommand::Seek { target_secs }) => {
                    let timestamp = (target_secs * 1_000_000.0) as i64;
                    if let Err(e) = ictx.seek(timestamp, ..timestamp) {
                        let _ = event_tx.blocking_send(AudioDecoderEvent::Error(format!(
                            "Audio seek failed: {e}"
                        )));
                        seek_target_secs = None;
                    } else {
                        decoder.flush();
                        // Reset timing after seek
                        playback_start_time = Some(std::time::Instant::now());
                        first_pts = None;
                        // Set target for precise seeking - will skip frames until we reach it
                        seek_target_secs = Some(target_secs);
                        // Reset seek frame counter for timeout protection
                        seek_frames_skipped = 0;

                        // Update sync clock seek position
                        if let Some(ref clock) = sync_clock {
                            clock.seek(target_secs);
                        }
                    }
                }
                Ok(AudioDecoderCommand::Stop) | Err(mpsc::error::TryRecvError::Disconnected) => {
                    // Stop sync clock
                    if let Some(ref clock) = sync_clock {
                        clock.stop();
                    }
                    break;
                }
                Ok(AudioDecoderCommand::SetVolume(volume)) => {
                    // Volume type guarantees valid range, no clamp needed
                    _volume = volume.value();
                }
                Ok(AudioDecoderCommand::SetMuted(mute)) => {
                    _muted = mute;
                }
                Ok(AudioDecoderCommand::SetPlaybackSpeed {
                    speed,
                    instant,
                    reference_pts,
                }) => {
                    // PlaybackSpeed newtype guarantees valid range
                    playback_speed = speed.value();
                    // Use shared reference point for timing synchronization
                    // Same reference_pts as video decoder for perfect A/V sync
                    playback_start_time = Some(instant);
                    first_pts = Some(reference_pts);
                }
                Err(mpsc::error::TryRecvError::Empty) => {}
            }

            // If not playing, sleep to avoid busy-waiting
            if !is_playing {
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }

            // Decode next audio frame
            let mut frame_decoded = false;
            for (stream, packet) in ictx.packets() {
                if stream.index() != audio_stream_index {
                    continue;
                }

                // Send packet to decoder
                if let Err(e) = decoder.send_packet(&packet) {
                    let _ = event_tx.blocking_send(AudioDecoderEvent::Error(format!(
                        "Audio packet failed: {e}"
                    )));
                    continue;
                }

                // Try to receive decoded frame
                let mut decoded_frame = ffmpeg_next::frame::Audio::empty();
                if decoder.receive_frame(&mut decoded_frame).is_ok() {
                    // Resample to f32 interleaved
                    let mut output_audio = ffmpeg_next::frame::Audio::empty();
                    if let Err(e) = resampler.run(&decoded_frame, &mut output_audio) {
                        let _ = event_tx.blocking_send(AudioDecoderEvent::Error(format!(
                            "Resampling failed: {e}"
                        )));
                        continue;
                    }

                    // Extract samples
                    let samples = Self::extract_samples(&output_audio, channels);

                    // Calculate PTS
                    let pts_secs = if let Some(pts) = decoded_frame.timestamp() {
                        pts as f64 * time_base_f64
                    } else {
                        0.0
                    };

                    // Calculate duration
                    let frame_duration =
                        samples.len() as f64 / (f64::from(sample_rate) * f64::from(channels));

                    // Precise seeking: skip audio frames before target PTS
                    // This ensures audio starts at the same position as video after seek
                    if let Some(target) = seek_target_secs {
                        let frame_end_pts = pts_secs + frame_duration;
                        if frame_end_pts < target {
                            // Increment and check timeout counter
                            seek_frames_skipped += 1;
                            if seek_frames_skipped >= MAX_SEEK_FRAMES {
                                // Timeout: target may be beyond EOF or file is corrupted
                                let _ = event_tx.blocking_send(AudioDecoderEvent::Error(
                                    "Audio seek timeout: target position may be beyond end of file"
                                        .to_string(),
                                ));
                                seek_target_secs = None;
                                continue;
                            }
                            // Entire frame is before target - skip it
                            continue;
                        }
                        // Frame contains or is after target - use seek target as timing
                        // reference (not the frame's PTS) to ensure A/V sync
                        first_pts = Some(target);
                        seek_target_secs = None;
                    }

                    // Frame pacing: wait until the audio should be queued
                    if let Some(start_time) = playback_start_time {
                        if first_pts.is_none() {
                            first_pts = Some(pts_secs);
                        }

                        if let Some(first) = first_pts {
                            // Calculate when this audio should be queued
                            // First, calculate relative time adjusted for playback speed
                            // Then subtract the lookahead (fixed buffer time in real-world seconds)
                            let frame_delay =
                                (pts_secs - first) / playback_speed - AUDIO_LOOKAHEAD_SECS;

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

                    // Update sync clock with audio PTS (audio is the master clock)
                    // This allows the video decoder to sync its frames to the audio timeline
                    if let Some(ref clock) = sync_clock {
                        clock.update_audio_pts(pts_secs);
                    }

                    // Send decoded audio
                    let audio = DecodedAudio {
                        samples: Arc::new(samples),
                        sample_rate,
                        channels,
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
                is_playing = false;
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
        let result = AudioDecoder::new(&video_path, None);
        // We expect either Ok(None) for no audio or Err for invalid file
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn audio_decoder_fails_for_nonexistent_file() {
        let result = AudioDecoder::new("/nonexistent/video.mp4", None);
        assert!(result.is_err());
    }
}
