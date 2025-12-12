// SPDX-License-Identifier: MPL-2.0
//! Playback state machine for video player.
//!
//! Manages the lifecycle of video playback with clear state transitions:
//! - Stopped: No playback, showing thumbnail
//! - Playing: Actively decoding and displaying frames
//! - Paused: Playback paused at current position
//! - Seeking: Jumping to a specific timestamp
//! - Buffering: Waiting for frames to be decoded
//! - Error: Playback failed, showing error state

use super::subscription::DecoderCommandSender;
use super::sync::{SharedSyncClock, SyncClock};
use super::DecoderCommand;
use crate::error::Result;
use crate::media::VideoData;
use std::sync::Arc;

/// Playback state machine.
///
/// This enum represents all possible states of the video player,
/// ensuring type-safe state transitions via pattern matching.
#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackState {
    /// Video is stopped, showing thumbnail.
    /// Initial state after loading.
    Stopped,

    /// Video is currently playing.
    /// Contains current playback position in seconds.
    Playing { position_secs: f64 },

    /// Video is paused at a specific position.
    /// User can resume from this position.
    Paused { position_secs: f64 },

    /// Video is seeking to a new position.
    /// Contains target position in seconds and whether to resume playing after seek.
    Seeking {
        target_secs: f64,
        resume_playing: bool,
    },

    /// Video is buffering (loading frames).
    /// Contains current position being buffered.
    Buffering { position_secs: f64 },

    /// Playback error occurred.
    /// Contains error message for display.
    Error { message: String },
}

impl PlaybackState {
    /// Returns the current playback position in seconds, if available.
    pub fn position(&self) -> Option<f64> {
        match self {
            Self::Stopped => Some(0.0),
            Self::Playing { position_secs } => Some(*position_secs),
            Self::Paused { position_secs } => Some(*position_secs),
            Self::Seeking { target_secs, .. } => Some(*target_secs),
            Self::Buffering { position_secs } => Some(*position_secs),
            Self::Error { .. } => None,
        }
    }

    /// Returns true if the video is currently playing.
    pub fn is_playing(&self) -> bool {
        matches!(self, Self::Playing { .. })
    }

    /// Returns true if the video is paused.
    pub fn is_paused(&self) -> bool {
        matches!(self, Self::Paused { .. })
    }

    /// Returns true if the video is in an error state.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    /// Returns the error message if in error state.
    pub fn error_message(&self) -> Option<&str> {
        match self {
            Self::Error { message } => Some(message),
            _ => None,
        }
    }

    /// Returns true if the video is playing or will resume playing after a seek/buffer.
    ///
    /// This is useful for determining keyboard shortcut behavior:
    /// - Arrow keys should seek while video is "actively playing" (including during seek)
    /// - Arrow keys should navigate when video is paused or stopped
    pub fn is_playing_or_will_resume(&self) -> bool {
        match self {
            Self::Playing { .. } => true,
            Self::Seeking { resume_playing, .. } => *resume_playing,
            Self::Buffering { .. } => true, // Buffering implies playback will continue
            _ => false,
        }
    }

    /// Returns true if the video is effectively paused (paused or seeking without resume).
    ///
    /// This is useful for frame-by-frame stepping which should work when:
    /// - Video is paused
    /// - Video is seeking but will stay paused after seek completes
    pub fn is_effectively_paused(&self) -> bool {
        match self {
            Self::Paused { .. } => true,
            Self::Seeking { resume_playing, .. } => !*resume_playing,
            _ => false,
        }
    }

    /// Returns the effective position for operations like frame stepping.
    ///
    /// Returns the current position when paused, or the target position when seeking.
    pub fn effective_position(&self) -> Option<f64> {
        match self {
            Self::Paused { position_secs } => Some(*position_secs),
            Self::Seeking { target_secs, .. } => Some(*target_secs),
            Self::Playing { position_secs } => Some(*position_secs),
            Self::Buffering { position_secs } => Some(*position_secs),
            _ => None,
        }
    }
}

/// Video player that manages playback state and frame delivery.
pub struct VideoPlayer {
    /// Current playback state.
    state: PlaybackState,

    /// Video metadata and thumbnail.
    video_data: VideoData,

    /// Whether the video should loop when it reaches the end.
    loop_enabled: bool,

    /// Command sender to control the decoder (provided by subscription).
    command_sender: Option<DecoderCommandSender>,

    /// Sync clock for audio/video synchronization.
    /// Shared with audio output to track playback position.
    sync_clock: SharedSyncClock,

    /// Current position in frame history (1-indexed).
    /// 0 = not in stepping mode, 1 = first frame, 2+ = can step backward.
    /// Reset to 0 on play/seek/stop, incremented on step_frame,
    /// decremented on step_backward.
    history_position: usize,

    /// Whether we've reached the end of the video stream.
    /// Set to true when EndOfStream is received, reset to false on seek/play.
    at_end_of_stream: bool,
}

impl VideoPlayer {
    /// Creates a new video player for the given video data.
    ///
    /// The player starts in the Stopped state, showing the thumbnail.
    /// Command sender is set when subscription starts.
    pub fn new(video_data: &VideoData) -> Result<Self> {
        Ok(Self {
            state: PlaybackState::Stopped,
            video_data: video_data.clone(),
            loop_enabled: false,
            command_sender: None,
            sync_clock: Arc::new(SyncClock::new()),
            history_position: 0,
            at_end_of_stream: false,
        })
    }

    /// Returns a clone of the sync clock for sharing with audio output.
    pub fn sync_clock(&self) -> SharedSyncClock {
        Arc::clone(&self.sync_clock)
    }

    /// Sets the command sender for controlling the decoder.
    /// This is called when the subscription sends the Started message.
    pub fn set_command_sender(&mut self, sender: DecoderCommandSender) {
        self.command_sender = Some(sender);
    }

    /// Returns true if the player has a command sender (subscription is active).
    pub fn has_command_sender(&self) -> bool {
        self.command_sender.is_some()
    }

    /// Returns the current playback state.
    pub fn state(&self) -> &PlaybackState {
        &self.state
    }

    /// Returns the video metadata.
    pub fn video_data(&self) -> &VideoData {
        &self.video_data
    }

    /// Returns whether loop is enabled.
    pub fn is_loop_enabled(&self) -> bool {
        self.loop_enabled
    }

    /// Sets whether the video should loop.
    pub fn set_loop(&mut self, enabled: bool) {
        self.loop_enabled = enabled;
    }

    /// Returns whether the player is in stepping mode.
    ///
    /// Stepping mode is entered when step_frame() is called, and exited
    /// when play(), seek(), or stop() is called.
    pub fn is_in_stepping_mode(&self) -> bool {
        self.history_position > 0
    }

    /// Returns whether backward stepping is available.
    ///
    /// Backward stepping is available after at least 1 step forward,
    /// because the initial frame (shown before stepping) is also added to history.
    pub fn can_step_backward(&self) -> bool {
        self.history_position >= 1
    }

    /// Returns whether forward stepping is available.
    ///
    /// Forward stepping is available when the video is paused and
    /// we haven't reached the end of the stream (last frame).
    pub fn can_step_forward(&self) -> bool {
        self.state.is_effectively_paused() && !self.at_end_of_stream
    }

    /// Marks that the end of stream has been reached.
    ///
    /// Called when EndOfStream event is received from the decoder.
    pub fn set_at_end_of_stream(&mut self) {
        self.at_end_of_stream = true;
    }

    /// Starts or resumes playback.
    ///
    /// State transitions:
    /// - Stopped → Playing (from beginning)
    /// - Paused → Playing (from current position, or from beginning if at end)
    /// - Playing → No change (idempotent)
    ///
    /// Sends Play command to decoder if it exists.
    /// Also starts/resumes the sync clock for A/V synchronization.
    /// Exits stepping mode (clears frame history in decoder).
    pub fn play(&mut self) {
        let position = match &self.state {
            PlaybackState::Stopped => {
                self.state = PlaybackState::Playing { position_secs: 0.0 };
                0.0
            }
            PlaybackState::Paused { position_secs } => {
                // If paused at the end of the video, restart from beginning
                let at_end = (*position_secs - self.video_data.duration_secs).abs() < 0.1;
                if at_end {
                    // Seek to beginning and play - use seek_and_play to ensure playback starts
                    self.seek_and_play(0.0);
                    return;
                }
                let pos = *position_secs;
                self.state = PlaybackState::Playing { position_secs: pos };
                pos
            }
            PlaybackState::Playing { .. } => {
                // Already playing, no-op
                return;
            }
            _ => {
                // Other states (Seeking, Buffering, Error) - no transition
                return;
            }
        };

        // Exit stepping mode - reset history position
        self.history_position = 0;

        // Clear end-of-stream flag since we're resuming playback
        self.at_end_of_stream = false;

        // Start sync clock at the current position.
        // Always use start() instead of resume() to ensure the sync clock
        // is at the correct position, especially after frame stepping where
        // the video position has advanced but audio hasn't.
        self.sync_clock.start(position);

        // Send Play command to decoder via command sender with resume position
        if let Some(sender) = &self.command_sender {
            let resume_position = if position > 0.0 { Some(position) } else { None };
            let _ = sender.send(DecoderCommand::Play {
                resume_position_secs: resume_position,
            });
        }
    }

    /// Pauses playback at the current position.
    ///
    /// State transitions:
    /// - Playing → Paused (at current position)
    /// - Paused → No change (idempotent)
    ///
    /// Sends Pause command to decoder via command sender.
    /// Also pauses the sync clock.
    pub fn pause(&mut self) {
        if let PlaybackState::Playing { position_secs } = &self.state {
            self.state = PlaybackState::Paused {
                position_secs: *position_secs,
            };

            // Pause sync clock
            self.sync_clock.pause();

            // Send Pause command to decoder via command sender
            if let Some(sender) = &self.command_sender {
                let _ = sender.send(DecoderCommand::Pause);
            }
        }
    }

    /// Stops playback and returns to the beginning.
    ///
    /// State transitions:
    /// - Any state → Stopped
    ///
    /// Sends Stop command to decoder via command sender.
    /// Also stops and resets the sync clock.
    /// Exits stepping mode.
    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;

        // Exit stepping mode - reset history position
        self.history_position = 0;

        // Stop sync clock
        self.sync_clock.stop();

        // Send Stop command to decoder via command sender
        if let Some(sender) = &self.command_sender {
            let _ = sender.send(DecoderCommand::Stop);
        }
    }

    /// Pauses playback at a specific position.
    ///
    /// Used when reaching end of video to pause at the end instead of stopping.
    /// Also pauses the sync clock.
    pub fn pause_at(&mut self, position_secs: f64) {
        self.state = PlaybackState::Paused { position_secs };

        // Pause sync clock
        self.sync_clock.pause();

        // Send Pause command to decoder via command sender
        if let Some(sender) = &self.command_sender {
            let _ = sender.send(DecoderCommand::Pause);
        }
    }

    /// Seeks to a specific position in the video.
    ///
    /// Position is clamped to [0, duration].
    /// After seek completes, playback continues if it was playing or will resume.
    ///
    /// Sends Seek command to decoder via command sender.
    /// Also updates the sync clock to the seek position.
    /// Exits stepping mode (clears frame history in decoder).
    pub fn seek(&mut self, target_secs: f64) {
        let clamped_target = target_secs.max(0.0).min(self.video_data.duration_secs);

        // Exit stepping mode - seek breaks frame continuity
        self.history_position = 0;

        // Clear end-of-stream flag since we're seeking to a new position
        self.at_end_of_stream = false;

        // Remember if we should resume playing after seek.
        // Use is_playing_or_will_resume() to handle chained seeks correctly:
        // if we're already seeking with resume_playing=true, preserve that intent.
        let should_resume = self.state.is_playing_or_will_resume();

        self.state = PlaybackState::Seeking {
            target_secs: clamped_target,
            resume_playing: should_resume,
        };

        // Update sync clock to seek position
        self.sync_clock.seek(clamped_target);

        // Send Seek command to decoder via command sender
        if let Some(sender) = &self.command_sender {
            let _ = sender.send(DecoderCommand::Seek {
                target_secs: clamped_target,
            });

            // If we should resume playing, send Play command after seek
            // No need to pass position since Seek already positioned the decoder
            if should_resume {
                let _ = sender.send(DecoderCommand::Play {
                    resume_position_secs: None,
                });
            }
        }
    }

    /// Seeks to a specific position and starts playback.
    ///
    /// Unlike `seek()`, this always resumes playback after the seek completes,
    /// regardless of the current state. Used when restarting from end of video.
    /// Exits stepping mode (clears frame history in decoder).
    pub fn seek_and_play(&mut self, target_secs: f64) {
        let clamped_target = target_secs.max(0.0).min(self.video_data.duration_secs);

        // Exit stepping mode - seek breaks frame continuity
        self.history_position = 0;

        // Clear end-of-stream flag since we're seeking to a new position
        self.at_end_of_stream = false;

        self.state = PlaybackState::Seeking {
            target_secs: clamped_target,
            resume_playing: true, // Always resume playing
        };

        // Update sync clock to seek position
        self.sync_clock.seek(clamped_target);

        // Send Seek command to decoder via command sender
        if let Some(sender) = &self.command_sender {
            let _ = sender.send(DecoderCommand::Seek {
                target_secs: clamped_target,
            });

            // Always send Play command after seek
            // No need to pass position since Seek already positioned the decoder
            let _ = sender.send(DecoderCommand::Play {
                resume_position_secs: None,
            });
        }
    }

    /// Updates playback position (called during playback or stepping).
    ///
    /// Should be called regularly by the playback loop to track progress.
    /// Updates position for Playing, Buffering, Seeking, and Paused (when stepping) states.
    pub fn update_position(&mut self, position_secs: f64) {
        match &self.state {
            PlaybackState::Playing { .. } => {
                self.state = PlaybackState::Playing { position_secs };
            }
            PlaybackState::Buffering { .. } => {
                // Keep buffering state but update position
                self.state = PlaybackState::Playing { position_secs };
            }
            PlaybackState::Seeking { resume_playing, .. } => {
                // Seek completed - transition to appropriate state
                if *resume_playing {
                    self.state = PlaybackState::Playing { position_secs };
                } else {
                    self.state = PlaybackState::Paused { position_secs };
                }
            }
            PlaybackState::Paused { .. } => {
                // Update position during frame stepping so that when we resume,
                // playback starts from the stepped position, not the original pause position.
                if self.is_in_stepping_mode() {
                    self.state = PlaybackState::Paused { position_secs };
                }
            }
            _ => {}
        }
    }

    /// Transitions to buffering state.
    pub fn set_buffering(&mut self, position_secs: f64) {
        self.state = PlaybackState::Buffering { position_secs };
    }

    /// Transitions to error state with the given message.
    pub fn set_error(&mut self, message: String) {
        self.state = PlaybackState::Error { message };
    }

    /// Completes seeking and transitions to appropriate state.
    ///
    /// If currently seeking, transitions to Playing or Paused based on resume_playing flag.
    pub fn complete_seek(&mut self) {
        if let PlaybackState::Seeking {
            target_secs,
            resume_playing,
        } = &self.state
        {
            if *resume_playing {
                self.state = PlaybackState::Playing {
                    position_secs: *target_secs,
                };
            } else {
                self.state = PlaybackState::Paused {
                    position_secs: *target_secs,
                };
            }
        }
    }

    /// Updates the sync clock with the current audio PTS.
    ///
    /// Called by audio playback to keep sync clock in sync with audio output.
    pub fn update_audio_pts(&self, pts_secs: f64) {
        self.sync_clock.update_audio_pts(pts_secs);
    }

    /// Returns the current synchronized playback time.
    ///
    /// This time is interpolated from the audio clock and can be used
    /// to determine if video frames should be displayed, skipped, or delayed.
    pub fn sync_time(&self) -> f64 {
        self.sync_clock.current_time_secs()
    }

    /// Sets the audio volume (0.0 to 1.0).
    ///
    /// Volume is sent to the audio decoder via the command sender.
    pub fn set_volume(&self, volume: f32) {
        if let Some(sender) = &self.command_sender {
            let _ = sender.set_volume(volume);
        }
    }

    /// Sets the mute state.
    ///
    /// Mute state is sent to the audio decoder via the command sender.
    pub fn set_muted(&self, muted: bool) {
        if let Some(sender) = &self.command_sender {
            let _ = sender.set_muted(muted);
        }
    }

    /// Returns true if audio is available for this video.
    pub fn has_audio(&self) -> bool {
        self.command_sender
            .as_ref()
            .map(|s| s.has_audio())
            .unwrap_or(false)
    }

    /// Steps forward one frame by decoding the next frame sequentially.
    ///
    /// This sends a StepFrame command to the decoder, which decodes the next
    /// frame in the video stream without seeking. This is the correct way to
    /// advance frame-by-frame since seek() only goes to keyframes.
    /// Increments history position (enables backward stepping after 2+ steps).
    pub fn step_frame(&mut self) {
        if !self.state.is_paused() {
            return;
        }

        // Increment history position - enables backward stepping after 2+ steps
        self.history_position += 1;

        // Send StepFrame command to decoder
        if let Some(sender) = &self.command_sender {
            let _ = sender.send(DecoderCommand::StepFrame);
        }
    }

    /// Steps forward one frame (only when paused).
    ///
    /// Deprecated: Use step_frame() instead which decodes sequentially.
    pub fn step_forward(&mut self) {
        self.step_frame();
    }

    /// Steps backward one frame by retrieving from frame history.
    ///
    /// This sends a StepBackward command to the decoder, which retrieves the
    /// previous frame from the frame history buffer. Frame history is only
    /// populated during stepping mode to save memory.
    /// Decrements history position (minimum 0 - can't go before initial frame).
    pub fn step_backward(&mut self) {
        if !self.state.is_paused() {
            return;
        }

        // Only step backward if we have history to go back to
        if self.history_position >= 1 {
            self.history_position -= 1;

            // Clear end-of-stream flag since we're stepping back from the end
            self.at_end_of_stream = false;

            // Send StepBackward command to decoder
            if let Some(sender) = &self.command_sender {
                let _ = sender.send(DecoderCommand::StepBackward);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::ImageData;

    fn sample_video_data() -> VideoData {
        VideoData {
            thumbnail: ImageData {
                handle: iced::widget::image::Handle::from_bytes(vec![]),
                width: 1920,
                height: 1080,
            },
            width: 1920,
            height: 1080,
            duration_secs: 120.0,
            fps: 30.0,
            has_audio: true,
        }
    }

    #[test]
    fn new_player_starts_in_stopped_state() {
        let video = sample_video_data();
        let player = VideoPlayer::new(&video).unwrap();

        assert_eq!(player.state(), &PlaybackState::Stopped);
        assert_eq!(player.state().position(), Some(0.0));
        assert!(!player.is_loop_enabled());
    }

    #[test]
    fn play_from_stopped_starts_at_beginning() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();

        assert!(player.state().is_playing());
        assert_eq!(player.state().position(), Some(0.0));
    }

    #[test]
    fn pause_from_playing_preserves_position() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();
        player.update_position(30.5);
        player.pause();

        assert!(player.state().is_paused());
        assert_eq!(player.state().position(), Some(30.5));
    }

    #[test]
    fn play_from_paused_resumes_at_current_position() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();
        player.update_position(45.0);
        player.pause();
        player.play();

        assert!(player.state().is_playing());
        assert_eq!(player.state().position(), Some(45.0));
    }

    #[test]
    fn stop_returns_to_beginning() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();
        player.update_position(60.0);
        player.stop();

        assert_eq!(player.state(), &PlaybackState::Stopped);
        assert_eq!(player.state().position(), Some(0.0));
    }

    #[test]
    fn seek_clamps_to_video_duration() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        // Seek beyond duration
        player.seek(200.0);
        assert_eq!(player.state().position(), Some(120.0));

        // Seek before start
        player.seek(-10.0);
        assert_eq!(player.state().position(), Some(0.0));

        // Seek to valid position
        player.seek(75.5);
        assert_eq!(player.state().position(), Some(75.5));
    }

    #[test]
    fn complete_seek_transitions_to_paused() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.seek(30.0);
        player.complete_seek();

        assert!(player.state().is_paused());
        assert_eq!(player.state().position(), Some(30.0));
    }

    #[test]
    fn set_loop_toggles_loop_state() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        assert!(!player.is_loop_enabled());

        player.set_loop(true);
        assert!(player.is_loop_enabled());

        player.set_loop(false);
        assert!(!player.is_loop_enabled());
    }

    #[test]
    fn error_state_clears_position() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();
        player.set_error("Decoding failed".to_string());

        assert!(player.state().is_error());
        assert_eq!(player.state().position(), None);
    }

    #[test]
    fn buffering_state_preserves_position() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();
        player.update_position(25.0);
        player.set_buffering(25.0);

        assert_eq!(
            player.state(),
            &PlaybackState::Buffering {
                position_secs: 25.0
            }
        );
        assert_eq!(player.state().position(), Some(25.0));
    }

    #[test]
    fn play_is_idempotent_when_already_playing() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();
        let state_before = player.state().clone();

        player.play();

        assert_eq!(player.state(), &state_before);
    }

    #[test]
    fn pause_is_idempotent_when_already_paused() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();
        player.update_position(10.0);
        player.pause();
        let state_before = player.state().clone();

        player.pause();

        assert_eq!(player.state(), &state_before);
    }

    #[test]
    fn is_playing_or_will_resume_reflects_playback_intent() {
        // Stopped: not playing
        assert!(!PlaybackState::Stopped.is_playing_or_will_resume());

        // Playing: yes
        assert!(PlaybackState::Playing {
            position_secs: 10.0
        }
        .is_playing_or_will_resume());

        // Paused: not playing
        assert!(!PlaybackState::Paused {
            position_secs: 10.0
        }
        .is_playing_or_will_resume());

        // Seeking with resume_playing=true: yes (will resume)
        assert!(PlaybackState::Seeking {
            target_secs: 20.0,
            resume_playing: true
        }
        .is_playing_or_will_resume());

        // Seeking with resume_playing=false: not playing
        assert!(!PlaybackState::Seeking {
            target_secs: 20.0,
            resume_playing: false
        }
        .is_playing_or_will_resume());

        // Buffering: yes (playback will continue)
        assert!(PlaybackState::Buffering {
            position_secs: 15.0
        }
        .is_playing_or_will_resume());

        // Error: not playing
        assert!(!PlaybackState::Error {
            message: "error".to_string()
        }
        .is_playing_or_will_resume());
    }

    #[test]
    fn chained_seeks_preserve_resume_playing_intent() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        // Start playing
        player.play();
        assert!(player.state().is_playing());

        // First seek while playing - should set resume_playing=true
        player.seek(30.0);
        assert!(matches!(
            player.state(),
            &PlaybackState::Seeking {
                resume_playing: true,
                ..
            }
        ));

        // Second seek while still seeking - should preserve resume_playing=true
        player.seek(45.0);
        assert!(matches!(
            player.state(),
            &PlaybackState::Seeking {
                resume_playing: true,
                ..
            }
        ));

        // Third seek - still preserve intent
        player.seek(60.0);
        assert!(matches!(
            player.state(),
            &PlaybackState::Seeking {
                resume_playing: true,
                ..
            }
        ));

        // When seek completes, should resume playing
        player.update_position(60.0);
        assert!(player.state().is_playing());
    }

    #[test]
    fn chained_seeks_from_paused_stay_paused() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        // Start paused (seek from stopped goes to paused)
        player.seek(10.0);
        player.update_position(10.0);
        assert!(player.state().is_paused());

        // First seek while paused - should set resume_playing=false
        player.seek(30.0);
        assert!(matches!(
            player.state(),
            &PlaybackState::Seeking {
                resume_playing: false,
                ..
            }
        ));

        // Second seek while still seeking - should preserve resume_playing=false
        player.seek(45.0);
        assert!(matches!(
            player.state(),
            &PlaybackState::Seeking {
                resume_playing: false,
                ..
            }
        ));

        // When seek completes, should stay paused
        player.update_position(45.0);
        assert!(player.state().is_paused());
    }

    #[test]
    fn sync_clock_starts_with_play() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        // Initially sync clock should not be playing
        assert!(!player.sync_clock().is_playing());

        // After play, sync clock should be playing
        player.play();
        assert!(player.sync_clock().is_playing());
    }

    #[test]
    fn sync_clock_pauses_with_player() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();
        assert!(player.sync_clock().is_playing());

        player.pause();
        assert!(!player.sync_clock().is_playing());
    }

    #[test]
    fn sync_clock_stops_with_player() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();
        player.stop();

        assert!(!player.sync_clock().is_playing());
        assert!(player.sync_clock().current_time_secs() < 0.001);
    }

    #[test]
    fn sync_clock_seek_updates_position() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.seek(45.0);
        // Sync clock should be updated to seek position
        let sync_time = player.sync_clock().current_time_secs();
        assert!((sync_time - 45.0).abs() < 0.1);
    }

    #[test]
    fn update_audio_pts_updates_sync_clock() {
        let video = sample_video_data();
        let player = VideoPlayer::new(&video).unwrap();

        player.update_audio_pts(30.5);
        let sync_time = player.sync_clock().current_time_secs();
        assert!((sync_time - 30.5).abs() < 0.001);
    }

    #[test]
    fn sync_time_returns_clock_time() {
        let video = sample_video_data();
        let player = VideoPlayer::new(&video).unwrap();

        player.update_audio_pts(25.0);
        assert!((player.sync_time() - 25.0).abs() < 0.001);
    }

    #[test]
    fn sync_clock_can_be_shared() {
        let video = sample_video_data();
        let player = VideoPlayer::new(&video).unwrap();

        let clock1 = player.sync_clock();
        let clock2 = player.sync_clock();

        // Update through one reference
        clock1.update_audio_pts(15.0);

        // Should be visible through other reference
        assert!((clock2.current_time_secs() - 15.0).abs() < 0.001);
    }

    #[test]
    fn play_from_end_restarts_at_beginning() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        // Simulate video reaching end and being paused there
        player.pause_at(video.duration_secs);
        assert!(player.state().is_paused());
        assert_eq!(player.state().position(), Some(120.0)); // duration_secs

        // Play should restart from beginning (seek_and_play to 0)
        player.play();

        // Should be in Seeking state with resume_playing=true, targeting 0
        assert!(matches!(
            player.state(),
            &PlaybackState::Seeking {
                target_secs,
                resume_playing: true,
            } if target_secs.abs() < 0.001
        ));
    }

    #[test]
    fn play_near_end_restarts_at_beginning() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        // Simulate being paused very close to end (within 0.1s tolerance)
        player.pause_at(video.duration_secs - 0.05);
        assert!(player.state().is_paused());

        // Play should restart from beginning
        player.play();

        // Should seek to beginning
        assert!(matches!(
            player.state(),
            &PlaybackState::Seeking {
                target_secs,
                resume_playing: true,
            } if target_secs.abs() < 0.001
        ));
    }

    #[test]
    fn play_not_at_end_resumes_normally() {
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        // Pause somewhere in the middle
        player.play();
        player.update_position(60.0);
        player.pause();
        assert!(player.state().is_paused());
        assert_eq!(player.state().position(), Some(60.0));

        // Play should resume from current position, not restart
        player.play();
        assert!(player.state().is_playing());
        assert_eq!(player.state().position(), Some(60.0));
    }

    // ========================================================================
    // A/V Sync Invariant Tests
    // These tests simulate UI action sequences that could cause A/V drift
    // and verify that position tracking remains consistent.
    // ========================================================================

    #[test]
    fn stepping_updates_paused_position() {
        // Simulates: pause → step_frame → frame arrives
        // Bug: position wasn't updated during stepping, causing drift on resume
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        // Play and pause at 10.0s
        player.play();
        player.update_position(10.0);
        player.pause();
        assert_eq!(player.state().position(), Some(10.0));

        // Step forward (enters stepping mode)
        player.step_frame();
        assert!(player.is_in_stepping_mode());

        // Frame arrives at 10.033s (one frame at 30fps)
        player.update_position(10.033);

        // Position MUST be updated (this was the bug)
        assert_eq!(player.state().position(), Some(10.033));
        assert!(player.state().is_paused());
    }

    #[test]
    fn stepping_multiple_frames_tracks_position() {
        // Simulates: pause → step → step → step (multiple frames)
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();
        player.update_position(20.0);
        player.pause();

        // Step through 5 frames (at 30fps, ~167ms)
        let frame_duration = 1.0 / 30.0;
        for i in 1..=5 {
            player.step_frame();
            let new_pos = 20.0 + (i as f64) * frame_duration;
            player.update_position(new_pos);
        }

        // Position should reflect all steps
        let expected = 20.0 + 5.0 * frame_duration;
        let actual = player.state().position().unwrap();
        assert!((actual - expected).abs() < 0.001);
    }

    #[test]
    fn stepping_then_play_uses_stepped_position() {
        // Simulates: pause → step → step → play
        // Verifies sync clock would start at correct position
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();
        player.update_position(15.0);
        player.pause();

        // Step forward a few frames
        player.step_frame();
        player.update_position(15.1);
        player.step_frame();
        player.update_position(15.2);

        // Resume playback
        player.play();

        // Must resume from stepped position, not original pause position
        assert!(player.state().is_playing());
        assert_eq!(player.state().position(), Some(15.2));

        // Verify sync clock is at the stepped position
        let sync_time = player.sync_time();
        assert!((sync_time - 15.2).abs() < 0.1);
    }

    #[test]
    fn stepping_backward_then_play_uses_correct_position() {
        // Simulates: pause → step → step → step_back → play
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();
        player.update_position(30.0);
        player.pause();

        // Step forward
        player.step_frame();
        player.update_position(30.033);
        player.step_frame();
        player.update_position(30.066);

        // Step backward (frame from history)
        player.step_backward();
        // Note: step_backward sends a command but doesn't directly update position
        // The decoder would send the previous frame back
        player.update_position(30.033);

        // Resume
        player.play();
        assert_eq!(player.state().position(), Some(30.033));
    }

    #[test]
    fn update_position_ignored_when_paused_not_stepping() {
        // Verifies that spurious frame updates don't change position when paused
        // (only stepping mode should update paused position)
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();
        player.update_position(25.0);
        player.pause();

        // Spurious update (not in stepping mode)
        assert!(!player.is_in_stepping_mode());
        player.update_position(99.0);

        // Position should NOT change
        assert_eq!(player.state().position(), Some(25.0));
    }

    #[test]
    fn seek_during_playback_sync_position() {
        // Simulates: playing → seek (slider drag) → frame arrives
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();
        player.update_position(10.0);

        // Seek to 45.0s (slider drag and release)
        player.seek(45.0);

        // Verify seeking state preserves resume intent
        assert!(matches!(
            player.state(),
            PlaybackState::Seeking { resume_playing: true, .. }
        ));

        // Frame arrives at seek target
        player.update_position(45.0);

        // Should be playing at new position
        assert!(player.state().is_playing());
        assert_eq!(player.state().position(), Some(45.0));
    }

    #[test]
    fn multiple_rapid_seeks_use_final_position() {
        // Simulates: playing → seek → seek → seek (rapid slider movements)
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();
        player.update_position(5.0);

        // Rapid seeks (user dragging slider)
        player.seek(20.0);
        player.seek(40.0);
        player.seek(60.0);

        // Final seek target
        assert_eq!(player.state().position(), Some(60.0));

        // Frame arrives
        player.update_position(60.0);
        assert!(player.state().is_playing());
        assert_eq!(player.state().position(), Some(60.0));
    }

    #[test]
    fn seek_while_paused_then_play() {
        // Simulates: pause → seek → play
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();
        player.update_position(10.0);
        player.pause();

        // Seek while paused
        player.seek(50.0);
        assert!(matches!(
            player.state(),
            PlaybackState::Seeking { resume_playing: false, .. }
        ));

        // Frame arrives (completes seek to paused)
        player.update_position(50.0);
        assert!(player.state().is_paused());
        assert_eq!(player.state().position(), Some(50.0));

        // Now play
        player.play();
        assert!(player.state().is_playing());
        assert_eq!(player.state().position(), Some(50.0));

        // Sync clock should be at 50.0
        let sync_time = player.sync_time();
        assert!((sync_time - 50.0).abs() < 0.1);
    }

    #[test]
    fn sync_clock_matches_position_after_play() {
        // Verifies sync clock is initialized to correct position on play()
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        // Test from stopped (position 0)
        player.play();
        assert!((player.sync_time() - 0.0).abs() < 0.1);

        // Test from paused
        player.update_position(35.0);
        player.pause();
        player.play();
        assert!((player.sync_time() - 35.0).abs() < 0.1);
    }

    #[test]
    fn sync_clock_matches_position_after_stepping_and_play() {
        // Critical test: verifies the A/V sync fix for stepping
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();
        player.update_position(10.0);
        player.pause();

        // Step forward significantly
        for i in 1..=30 {
            player.step_frame();
            player.update_position(10.0 + (i as f64) * 0.033);
        }

        let stepped_position = player.state().position().unwrap();
        assert!((stepped_position - 10.99).abs() < 0.1); // ~11 seconds after 30 frames

        // Resume playback
        player.play();

        // Sync clock MUST match the stepped position (this was the bug)
        let sync_time = player.sync_time();
        assert!(
            (sync_time - stepped_position).abs() < 0.1,
            "Sync clock {} doesn't match stepped position {}",
            sync_time,
            stepped_position
        );
    }

    // ========================================================================
    // Edge Case Tests - Loop, End-of-Stream, Complex Sequences
    // ========================================================================

    #[test]
    fn loop_restart_maintains_sync() {
        // Simulates: playing → end of stream → loop restart (seek(0) + play)
        // This is what the UI does when loop is enabled and video ends
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.set_loop(true);
        player.play();
        player.update_position(video.duration_secs);

        // Simulate loop restart (what component.rs does on EndOfStream with loop)
        player.seek(0.0);
        player.play();

        // After loop restart, should be seeking to 0 with resume_playing=true
        // (because play() after seek() transitions to Playing when seek completes)
        assert!(matches!(
            player.state(),
            PlaybackState::Seeking { target_secs, resume_playing: true } if *target_secs == 0.0
        ));

        // Frame arrives at position 0
        player.update_position(0.0);

        // Should be playing at 0
        assert!(player.state().is_playing());
        assert_eq!(player.state().position(), Some(0.0));

        // Sync clock must be at 0
        let sync_time = player.sync_time();
        assert!(
            sync_time.abs() < 0.1,
            "Sync clock {} should be near 0 after loop restart",
            sync_time
        );
    }

    #[test]
    fn seek_after_end_of_stream_resyncs() {
        // Simulates: playing → end of stream (paused at end) → seek back → play
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();
        player.update_position(video.duration_secs);

        // End of stream without loop - pause at end
        player.set_at_end_of_stream();
        player.pause_at(video.duration_secs);

        assert!(player.state().is_paused());
        assert_eq!(player.state().position(), Some(video.duration_secs));

        // User seeks back to middle
        player.seek(60.0);
        player.update_position(60.0);

        assert!(player.state().is_paused());
        assert_eq!(player.state().position(), Some(60.0));

        // User plays
        player.play();

        assert!(player.state().is_playing());
        assert_eq!(player.state().position(), Some(60.0));

        // Sync clock must be at 60.0
        let sync_time = player.sync_time();
        assert!(
            (sync_time - 60.0).abs() < 0.1,
            "Sync clock {} should be near 60.0 after seek from end",
            sync_time
        );
    }

    #[test]
    fn complex_sequence_pause_seek_step_play() {
        // Simulates a complex user interaction sequence:
        // playing → pause → seek → step → step → play
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        // Start playing at 10s
        player.play();
        player.update_position(10.0);

        // Pause
        player.pause();
        assert!(player.state().is_paused());
        assert_eq!(player.state().position(), Some(10.0));

        // Seek to 50s while paused
        player.seek(50.0);
        player.update_position(50.0);
        assert!(player.state().is_paused());
        assert_eq!(player.state().position(), Some(50.0));

        // Step forward twice
        player.step_frame();
        player.update_position(50.033);
        player.step_frame();
        player.update_position(50.066);

        assert!(player.state().is_paused());
        assert_eq!(player.state().position(), Some(50.066));

        // Resume playback
        player.play();

        // Must be playing at stepped position
        assert!(player.state().is_playing());
        assert_eq!(player.state().position(), Some(50.066));

        // Sync clock must match
        let sync_time = player.sync_time();
        assert!(
            (sync_time - 50.066).abs() < 0.1,
            "Sync clock {} should match stepped position 50.066",
            sync_time
        );
    }

    #[test]
    fn seek_to_zero_from_middle_resyncs() {
        // Edge case: seek to exactly 0.0 (beginning)
        let video = sample_video_data();
        let mut player = VideoPlayer::new(&video).unwrap();

        player.play();
        player.update_position(45.0);

        // Seek to beginning
        player.seek(0.0);

        // Frame arrives
        player.update_position(0.0);

        assert!(player.state().is_playing());
        assert_eq!(player.state().position(), Some(0.0));

        // Sync clock must be at 0
        let sync_time = player.sync_time();
        assert!(
            sync_time.abs() < 0.1,
            "Sync clock {} should be near 0",
            sync_time
        );
    }
}
