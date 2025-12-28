// SPDX-License-Identifier: MPL-2.0
//! Audio output using cpal for low-latency playback.
//!
//! This module provides audio playback functionality using the cpal library,
//! supporting real-time volume control and muting.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::error::{Error, Result};

/// Audio samples to be played.
/// Interleaved f32 samples normalized to [-1.0, 1.0].
pub type AudioSamples = Arc<Vec<f32>>;

/// Commands for controlling audio output.
#[derive(Debug)]
pub enum AudioOutputCommand {
    /// Play audio samples.
    Play(AudioSamples),

    /// Pause playback.
    Pause,

    /// Resume playback.
    Resume,

    /// Stop playback and clear buffer.
    Stop,

    /// Clear buffer without changing pause state.
    /// Used during seek to discard old audio without interrupting playback.
    ClearBuffer,

    /// Set volume (0.0–1.5, perceptually scaled).
    /// A quadratic curve is applied: actual = slider², so:
    /// - 50% slider → 25% actual (-12 dB)
    /// - 100% slider → 100% actual (0 dB)
    /// - 150% slider → 225% actual (+7 dB)
    SetVolume(super::Volume),

    /// Set mute state.
    SetMuted(bool),
}

/// Shared state between audio thread and main thread.
struct SharedState {
    /// Current volume (stored as u32 bits of f32 for atomic access).
    volume_bits: AtomicU32,

    /// Mute state.
    muted: AtomicBool,

    /// Pause state.
    paused: AtomicBool,
}

impl SharedState {
    fn new(initial_volume: f32) -> Self {
        Self {
            volume_bits: AtomicU32::new(initial_volume.to_bits()),
            muted: AtomicBool::new(false),
            paused: AtomicBool::new(false),
        }
    }

    fn volume(&self) -> f32 {
        f32::from_bits(self.volume_bits.load(Ordering::Relaxed))
    }

    fn set_volume(&self, volume: f32) {
        self.volume_bits.store(volume.to_bits(), Ordering::Relaxed);
    }

    fn is_muted(&self) -> bool {
        self.muted.load(Ordering::Relaxed)
    }

    fn set_muted(&self, muted: bool) {
        self.muted.store(muted, Ordering::Relaxed);
    }

    fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::Relaxed);
    }
}

/// Audio output stream manager.
///
/// Handles audio playback through the system's default audio device.
pub struct AudioOutput {
    /// Channel for sending commands to the audio thread.
    command_tx: mpsc::UnboundedSender<AudioOutputCommand>,

    /// Shared state for volume/mute control.
    shared_state: Arc<SharedState>,

    /// Sample rate of the output device.
    sample_rate: u32,

    /// Number of channels of the output device.
    channels: u16,

    /// The audio stream (kept alive to maintain playback).
    _stream: cpal::Stream,
}

impl AudioOutput {
    /// Creates a new audio output stream.
    ///
    /// Returns the configured sample rate and channel count that the caller
    /// should use for resampling audio to match the output device.
    ///
    /// # Errors
    ///
    /// Returns an error if no audio output device is found, if the device
    /// configuration cannot be retrieved, or if the audio stream fails to start.
    pub fn new(initial_volume: f32) -> Result<Self> {
        // Get the default audio host and device
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| Error::Io("No audio output device found".to_string()))?;

        // Get supported config
        let supported_config = device
            .default_output_config()
            .map_err(|e| Error::Io(format!("Failed to get audio config: {e}")))?;

        let sample_rate = supported_config.sample_rate();
        let channels = supported_config.channels();

        // Create shared state
        let shared_state = Arc::new(SharedState::new(initial_volume));
        let shared_state_clone = Arc::clone(&shared_state);

        // Create command channel
        let (command_tx, mut command_rx) = mpsc::unbounded_channel::<AudioOutputCommand>();

        // Audio buffer for samples to play
        // Limited to ~0.5 seconds to prevent unbounded growth
        let max_buffer_size = (sample_rate as usize) * (channels as usize); // 0.5 seconds at stereo
        let buffer: Arc<std::sync::Mutex<Vec<f32>>> =
            Arc::new(std::sync::Mutex::new(Vec::with_capacity(max_buffer_size)));
        let buffer_clone = Arc::clone(&buffer);

        // Spawn a task to process commands
        let buffer_for_task = Arc::clone(&buffer);
        let shared_for_task = Arc::clone(&shared_state);
        let max_buffer_for_task = max_buffer_size;
        tokio::spawn(async move {
            while let Some(cmd) = command_rx.recv().await {
                match cmd {
                    AudioOutputCommand::Play(samples) => {
                        if let Ok(mut buf) = buffer_for_task.lock() {
                            // Only add samples if buffer has room (backpressure)
                            // This prevents unbounded memory growth
                            let available_space = max_buffer_for_task.saturating_sub(buf.len());
                            if available_space >= samples.len() {
                                buf.extend_from_slice(&samples);
                            } else if available_space > 0 {
                                // Add partial samples if some space available
                                buf.extend_from_slice(&samples[..available_space]);
                            }
                            // If buffer is full, drop samples (better than OOM)
                        }
                    }
                    AudioOutputCommand::Pause => {
                        shared_for_task.set_paused(true);
                    }
                    AudioOutputCommand::Resume => {
                        shared_for_task.set_paused(false);
                    }
                    AudioOutputCommand::Stop => {
                        if let Ok(mut buf) = buffer_for_task.lock() {
                            buf.clear();
                        }
                        shared_for_task.set_paused(true);
                    }
                    AudioOutputCommand::ClearBuffer => {
                        // Clear buffer without changing pause state
                        // Used during seek to discard old audio
                        if let Ok(mut buf) = buffer_for_task.lock() {
                            buf.clear();
                        }
                    }
                    AudioOutputCommand::SetVolume(volume) => {
                        // Volume type guarantees valid range, no clamp needed
                        shared_for_task.set_volume(volume.value());
                    }
                    AudioOutputCommand::SetMuted(muted) => {
                        shared_for_task.set_muted(muted);
                    }
                }
            }
        });

        // Build audio stream based on sample format
        let stream = match supported_config.sample_format() {
            cpal::SampleFormat::F32 => Self::build_stream::<f32>(
                &device,
                &supported_config.into(),
                buffer_clone,
                shared_state_clone,
            )?,
            cpal::SampleFormat::I16 => Self::build_stream::<i16>(
                &device,
                &supported_config.into(),
                buffer_clone,
                shared_state_clone,
            )?,
            cpal::SampleFormat::U16 => Self::build_stream::<u16>(
                &device,
                &supported_config.into(),
                buffer_clone,
                shared_state_clone,
            )?,
            _ => return Err(Error::Io("Unsupported audio sample format".to_string())),
        };

        // Start the stream
        stream
            .play()
            .map_err(|e| Error::Io(format!("Failed to start audio stream: {e}")))?;

        Ok(Self {
            command_tx,
            shared_state,
            sample_rate,
            channels,
            _stream: stream,
        })
    }

    /// Builds an audio output stream for a specific sample format.
    fn build_stream<T: cpal::SizedSample + cpal::FromSample<f32>>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        buffer: Arc<std::sync::Mutex<Vec<f32>>>,
        shared_state: Arc<SharedState>,
    ) -> Result<cpal::Stream> {
        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    let volume = shared_state.volume();
                    let muted = shared_state.is_muted();
                    let paused = shared_state.is_paused();

                    if muted || paused {
                        // Output silence
                        for sample in data.iter_mut() {
                            *sample = T::from_sample(0.0f32);
                        }
                        return;
                    }

                    // Get samples from buffer
                    let Ok(mut buf) = buffer.lock() else {
                        // Mutex poisoned, output silence
                        for sample in data.iter_mut() {
                            *sample = T::from_sample(0.0f32);
                        }
                        return;
                    };

                    // Apply perceptual volume curve (quadratic) for natural-feeling control.
                    // Human hearing is logarithmic, so a linear slider feels wrong.
                    // Squaring the volume makes the slider perceptually linear:
                    // - 50% slider → 25% actual (-12 dB, sounds like "half")
                    // - 100% slider → 100% actual (0 dB, unchanged)
                    // - 150% slider → 225% actual (+7 dB, clearly louder)
                    let perceptual_volume = volume * volume;

                    for (i, sample) in data.iter_mut().enumerate() {
                        if i < buf.len() {
                            // Apply volume and clamp to safe range.
                            // Clamping to slightly below 1.0 prevents i16 overflow
                            // (dasp's from_sample overflows at exactly 1.0 for i16).
                            // Values > 1.0 represent amplification and will be clipped.
                            let amplified = (buf[i] * perceptual_volume).clamp(-1.0, 0.999_999_9);
                            *sample = T::from_sample(amplified);
                        } else {
                            *sample = T::from_sample(0.0f32);
                        }
                    }

                    // Remove consumed samples
                    let consumed = data.len().min(buf.len());
                    buf.drain(..consumed);
                },
                |err| {
                    eprintln!("Audio output error: {err}");
                },
                None,
            )
            .map_err(|e| Error::Io(format!("Failed to build audio stream: {e}")))?;

        Ok(stream)
    }

    /// Sends a command to the audio output.
    ///
    /// # Errors
    ///
    /// Returns an error if the audio output channel is closed.
    pub fn send_command(&self, command: AudioOutputCommand) -> Result<()> {
        self.command_tx
            .send(command)
            .map_err(|_| Error::Io("Audio output channel closed".into()))
    }

    /// Queues audio samples for playback.
    ///
    /// # Errors
    ///
    /// Returns an error if the audio output channel is closed.
    pub fn play(&self, samples: AudioSamples) -> Result<()> {
        self.send_command(AudioOutputCommand::Play(samples))
    }

    /// Pauses audio playback.
    ///
    /// # Errors
    ///
    /// Returns an error if the audio output channel is closed.
    pub fn pause(&self) -> Result<()> {
        self.send_command(AudioOutputCommand::Pause)
    }

    /// Resumes audio playback.
    ///
    /// # Errors
    ///
    /// Returns an error if the audio output channel is closed.
    pub fn resume(&self) -> Result<()> {
        self.send_command(AudioOutputCommand::Resume)
    }

    /// Stops playback and clears the buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the audio output channel is closed.
    pub fn stop(&self) -> Result<()> {
        self.send_command(AudioOutputCommand::Stop)
    }

    /// Clears the audio buffer without changing pause state.
    /// Used during seek to discard old audio without interrupting playback.
    ///
    /// # Errors
    ///
    /// Returns an error if the audio output channel is closed.
    pub fn clear_buffer(&self) -> Result<()> {
        self.send_command(AudioOutputCommand::ClearBuffer)
    }

    /// Sets the volume.
    ///
    /// # Errors
    ///
    /// Returns an error if the audio output channel is closed.
    pub fn set_volume(&self, volume: super::Volume) -> Result<()> {
        self.send_command(AudioOutputCommand::SetVolume(volume))
    }

    /// Sets the mute state.
    ///
    /// # Errors
    ///
    /// Returns an error if the audio output channel is closed.
    pub fn set_muted(&self, muted: bool) -> Result<()> {
        self.send_command(AudioOutputCommand::SetMuted(muted))
    }

    /// Returns the current volume.
    #[must_use]
    pub fn volume(&self) -> f32 {
        self.shared_state.volume()
    }

    /// Returns whether audio is muted.
    #[must_use]
    pub fn is_muted(&self) -> bool {
        self.shared_state.is_muted()
    }

    /// Returns the output sample rate.
    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Returns the number of output channels.
    #[must_use]
    pub fn channels(&self) -> u16 {
        self.channels
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_player::Volume;

    #[test]
    fn shared_state_volume_operations() {
        let state = SharedState::new(0.8);
        assert!((state.volume() - 0.8).abs() < 0.001);

        state.set_volume(0.5);
        assert!((state.volume() - 0.5).abs() < 0.001);

        state.set_volume(1.5); // Should still store as-is (clamping done elsewhere)
        assert!((state.volume() - 1.5).abs() < 0.001);
    }

    #[test]
    fn shared_state_mute_operations() {
        let state = SharedState::new(1.0);
        assert!(!state.is_muted());

        state.set_muted(true);
        assert!(state.is_muted());

        state.set_muted(false);
        assert!(!state.is_muted());
    }

    #[test]
    fn shared_state_pause_operations() {
        let state = SharedState::new(1.0);
        assert!(!state.is_paused());

        state.set_paused(true);
        assert!(state.is_paused());

        state.set_paused(false);
        assert!(!state.is_paused());
    }

    #[test]
    fn audio_output_command_debug() {
        let cmd = AudioOutputCommand::SetVolume(Volume::new(0.5));
        let debug_str = format!("{cmd:?}");
        assert!(debug_str.contains("SetVolume"));
    }

    // Note: Tests that create AudioOutput require actual audio hardware
    // and are better suited for integration tests or manual testing.
    // The following test is marked as ignored by default.
    #[tokio::test]
    #[ignore = "requires audio hardware"]
    async fn audio_output_can_be_created() {
        let result = AudioOutput::new(0.8);
        // This may fail on CI without audio hardware, so we just check it doesn't panic
        if let Ok(output) = result {
            assert!((output.volume() - 0.8).abs() < 0.001);
            assert!(!output.is_muted());
            assert!(output.sample_rate() > 0);
            assert!(output.channels() > 0);
        }
    }
}
