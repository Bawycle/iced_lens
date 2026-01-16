// SPDX-License-Identifier: MPL-2.0
//! Video playback sub-component wrapping `VideoPlayer` state machine.

use crate::diagnostics::DiagnosticsHandle;
use crate::media::VideoData;
use crate::video_player::subscription::PlaybackMessage;
use crate::video_player::{PlaybackState, VideoPlayer, Volume};
use iced::widget::image::Handle as ImageHandle;

/// Video playback sub-component state.
pub struct State {
    /// The underlying video player (if video is loaded).
    player: Option<VideoPlayer>,
    /// Current frame to display.
    current_frame: Option<ImageHandle>,
    /// Whether playback is active (subscription running).
    playback_active: bool,
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("has_player", &self.player.is_some())
            .field("has_frame", &self.current_frame.is_some())
            .field("playback_active", &self.playback_active)
            .finish()
    }
}

impl Clone for State {
    fn clone(&self) -> Self {
        // VideoPlayer cannot be cloned due to command sender.
        // Clone only the frame and active flag.
        Self {
            player: None,
            current_frame: self.current_frame.clone(),
            playback_active: false,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            player: None,
            current_frame: None,
            playback_active: false,
        }
    }
}

/// Messages for the video playback sub-component.
#[derive(Debug, Clone)]
pub enum Message {
    /// Initialize video player with video data.
    Initialize(VideoData),
    /// Clear video player state.
    Clear,
    /// Playback event from subscription.
    PlaybackEvent(PlaybackMessage),
    /// Play/resume playback.
    Play,
    /// Pause playback.
    Pause,
    /// Toggle play/pause.
    TogglePlayback,
    /// Stop playback (return to beginning).
    Stop,
    /// Seek to position (0.0-1.0 relative).
    SeekRelative(f32),
    /// Seek to absolute position in seconds.
    SeekAbsolute(f64),
    /// Step forward one frame.
    StepForward,
    /// Step backward one frame.
    StepBackward,
    /// Toggle loop mode.
    ToggleLoop,
    /// Set volume.
    SetVolume(Volume),
    /// Set mute state.
    SetMuted(bool),
    /// Increase playback speed.
    IncreaseSpeed,
    /// Decrease playback speed.
    DecreaseSpeed,
    /// Set diagnostics handle.
    SetDiagnostics(DiagnosticsHandle),
}

/// Effects produced by video playback changes.
#[derive(Debug, Clone)]
pub enum Effect {
    /// No effect.
    None,
    /// Video player initialized - start subscription.
    PlayerInitialized,
    /// Video player cleared.
    PlayerCleared,
    /// Frame updated - view needs refresh.
    FrameUpdated,
    /// Playback state changed.
    StateChanged,
    /// End of stream reached.
    EndOfStream { loop_enabled: bool },
    /// Playback error occurred.
    PlaybackError { message: String },
    /// Capture current frame for editing.
    CaptureFrame {
        frame: crate::media::frame_export::ExportableFrame,
    },
    /// Speed changed - show notification.
    SpeedChanged { speed: f64 },
}

impl State {
    /// Handle a video playback message.
    #[allow(clippy::too_many_lines)]
    pub fn handle(&mut self, msg: Message) -> Effect {
        match msg {
            Message::Initialize(video_data) => {
                match VideoPlayer::new(&video_data) {
                    Ok(player) => {
                        // Set thumbnail as initial frame
                        let thumbnail = video_data.thumbnail.handle.clone();
                        self.current_frame = Some(thumbnail);
                        self.player = Some(player);
                        self.playback_active = false;
                        Effect::PlayerInitialized
                    }
                    Err(e) => Effect::PlaybackError {
                        message: e.to_string(),
                    },
                }
            }
            Message::Clear => {
                self.player = None;
                self.current_frame = None;
                self.playback_active = false;
                Effect::PlayerCleared
            }
            Message::PlaybackEvent(event) => self.handle_playback_event(event),
            Message::Play => {
                if let Some(player) = &mut self.player {
                    player.play();
                    Effect::StateChanged
                } else {
                    Effect::None
                }
            }
            Message::Pause => {
                if let Some(player) = &mut self.player {
                    player.pause();
                    Effect::StateChanged
                } else {
                    Effect::None
                }
            }
            Message::TogglePlayback => {
                if let Some(player) = &mut self.player {
                    if player.state().is_playing() {
                        player.pause();
                    } else {
                        player.play();
                    }
                    Effect::StateChanged
                } else {
                    Effect::None
                }
            }
            Message::Stop => {
                if let Some(player) = &mut self.player {
                    player.stop();
                    // Reset to thumbnail
                    if let Some(ref player) = self.player {
                        let thumbnail = player.video_data().thumbnail.handle.clone();
                        self.current_frame = Some(thumbnail);
                    }
                    Effect::StateChanged
                } else {
                    Effect::None
                }
            }
            Message::SeekRelative(ratio) => {
                if let Some(player) = &mut self.player {
                    let duration = player.video_data().duration_secs;
                    let target = f64::from(ratio) * duration;
                    player.seek(target);
                    Effect::StateChanged
                } else {
                    Effect::None
                }
            }
            Message::SeekAbsolute(target_secs) => {
                if let Some(player) = &mut self.player {
                    player.seek(target_secs);
                    Effect::StateChanged
                } else {
                    Effect::None
                }
            }
            Message::StepForward => {
                if let Some(player) = &mut self.player {
                    player.step_frame();
                    Effect::StateChanged
                } else {
                    Effect::None
                }
            }
            Message::StepBackward => {
                if let Some(player) = &mut self.player {
                    player.step_backward();
                    Effect::StateChanged
                } else {
                    Effect::None
                }
            }
            Message::ToggleLoop => {
                if let Some(player) = &mut self.player {
                    player.set_loop(!player.is_loop_enabled());
                    Effect::StateChanged
                } else {
                    Effect::None
                }
            }
            Message::SetVolume(volume) => {
                if let Some(player) = &self.player {
                    player.set_volume(volume);
                }
                Effect::None
            }
            Message::SetMuted(muted) => {
                if let Some(player) = &self.player {
                    player.set_muted(muted);
                }
                Effect::None
            }
            Message::IncreaseSpeed => {
                if let Some(player) = &mut self.player {
                    let speed = player.increase_playback_speed();
                    Effect::SpeedChanged { speed }
                } else {
                    Effect::None
                }
            }
            Message::DecreaseSpeed => {
                if let Some(player) = &mut self.player {
                    let speed = player.decrease_playback_speed();
                    Effect::SpeedChanged { speed }
                } else {
                    Effect::None
                }
            }
            Message::SetDiagnostics(handle) => {
                if let Some(player) = &mut self.player {
                    player.set_diagnostics(handle);
                }
                Effect::None
            }
        }
    }

    /// Handle playback events from the video subscription.
    fn handle_playback_event(&mut self, event: PlaybackMessage) -> Effect {
        let Some(player) = &mut self.player else {
            return Effect::None;
        };

        match event {
            PlaybackMessage::Started(sender) => {
                player.set_command_sender(sender);
                self.playback_active = true;
                Effect::StateChanged
            }
            PlaybackMessage::FrameReady {
                rgba_data,
                width,
                height,
                pts_secs,
            } => {
                // Convert raw RGBA data to Iced image handle
                let handle = ImageHandle::from_rgba(width, height, rgba_data.to_vec());
                self.current_frame = Some(handle);
                player.update_position(pts_secs);
                Effect::FrameUpdated
            }
            PlaybackMessage::AudioPts(pts_secs) => {
                player.update_audio_pts(pts_secs);
                Effect::None
            }
            PlaybackMessage::Buffering => {
                // Buffering doesn't carry position - use current position
                let position = player.state().position().unwrap_or(0.0);
                player.set_buffering(position);
                Effect::StateChanged
            }
            PlaybackMessage::EndOfStream => {
                player.set_at_end_of_stream();
                let loop_enabled = player.is_loop_enabled();

                if loop_enabled {
                    player.seek(0.0);
                    player.play();
                } else {
                    // Pause at end
                    let duration = player.video_data().duration_secs;
                    player.pause_at(duration);
                }

                Effect::EndOfStream { loop_enabled }
            }
            PlaybackMessage::HistoryExhausted => {
                player.reset_history_position();
                Effect::None
            }
            PlaybackMessage::Error(error) => {
                player.set_error(error.clone());
                Effect::PlaybackError { message: error }
            }
        }
    }

    /// Get the video player reference.
    #[must_use]
    pub fn player(&self) -> Option<&VideoPlayer> {
        self.player.as_ref()
    }

    /// Get the video player mutably.
    pub fn player_mut(&mut self) -> Option<&mut VideoPlayer> {
        self.player.as_mut()
    }

    /// Get the current frame to display.
    #[must_use]
    pub fn current_frame(&self) -> Option<&ImageHandle> {
        self.current_frame.as_ref()
    }

    /// Check if a video is loaded.
    #[must_use]
    pub fn has_video(&self) -> bool {
        self.player.is_some()
    }

    /// Check if playback is active.
    #[must_use]
    pub fn is_playback_active(&self) -> bool {
        self.playback_active
    }

    /// Get the current playback state.
    #[must_use]
    pub fn playback_state(&self) -> Option<&PlaybackState> {
        self.player.as_ref().map(VideoPlayer::state)
    }

    /// Check if video is currently playing.
    #[must_use]
    pub fn is_playing(&self) -> bool {
        self.player
            .as_ref()
            .is_some_and(|p| p.state().is_playing())
    }

    /// Check if video is paused.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.player.as_ref().is_some_and(|p| p.state().is_paused())
    }

    /// Get the current playback position in seconds.
    #[must_use]
    pub fn position(&self) -> Option<f64> {
        self.player.as_ref().and_then(|p| p.state().position())
    }

    /// Get the video duration in seconds.
    #[must_use]
    pub fn duration(&self) -> Option<f64> {
        self.player.as_ref().map(|p| p.video_data().duration_secs)
    }

    /// Get the current playback speed.
    #[must_use]
    pub fn playback_speed(&self) -> f64 {
        self.player
            .as_ref()
            .map_or(1.0, VideoPlayer::playback_speed)
    }

    /// Check if loop is enabled.
    #[must_use]
    pub fn is_loop_enabled(&self) -> bool {
        self.player
            .as_ref()
            .is_some_and(VideoPlayer::is_loop_enabled)
    }

    /// Check if audio is available.
    #[must_use]
    pub fn has_audio(&self) -> bool {
        self.player.as_ref().is_some_and(VideoPlayer::has_audio)
    }

    /// Check if forward stepping is available.
    #[must_use]
    pub fn can_step_forward(&self) -> bool {
        self.player
            .as_ref()
            .is_some_and(VideoPlayer::can_step_forward)
    }

    /// Check if backward stepping is available.
    #[must_use]
    pub fn can_step_backward(&self) -> bool {
        self.player
            .as_ref()
            .is_some_and(VideoPlayer::can_step_backward)
    }

    /// Get the sync clock for A/V synchronization.
    #[must_use]
    pub fn sync_clock(&self) -> Option<crate::video_player::SharedSyncClock> {
        self.player.as_ref().map(VideoPlayer::sync_clock)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::ImageData;

    fn sample_video_data() -> VideoData {
        let pixels = vec![0_u8; 100 * 100 * 4];
        VideoData {
            thumbnail: ImageData::from_rgba(100, 100, pixels),
            width: 1920,
            height: 1080,
            duration_secs: 120.0,
            fps: 30.0,
            has_audio: true,
        }
    }

    #[test]
    fn default_state_has_no_video() {
        let state = State::default();
        assert!(!state.has_video());
        assert!(state.current_frame().is_none());
        assert!(!state.is_playback_active());
    }

    #[test]
    fn initialize_creates_player() {
        let mut state = State::default();
        let video = sample_video_data();

        let effect = state.handle(Message::Initialize(video));

        assert!(matches!(effect, Effect::PlayerInitialized));
        assert!(state.has_video());
        assert!(state.current_frame().is_some()); // Thumbnail
    }

    #[test]
    fn clear_removes_player() {
        let mut state = State::default();
        state.handle(Message::Initialize(sample_video_data()));
        assert!(state.has_video());

        let effect = state.handle(Message::Clear);

        assert!(matches!(effect, Effect::PlayerCleared));
        assert!(!state.has_video());
        assert!(state.current_frame().is_none());
    }

    #[test]
    fn play_without_player_returns_none() {
        let mut state = State::default();
        let effect = state.handle(Message::Play);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn toggle_playback_works() {
        let mut state = State::default();
        state.handle(Message::Initialize(sample_video_data()));

        // Initially stopped, toggle should start playing
        let effect = state.handle(Message::TogglePlayback);
        assert!(matches!(effect, Effect::StateChanged));
        assert!(state.is_playing());

        // Toggle again should pause
        let effect = state.handle(Message::TogglePlayback);
        assert!(matches!(effect, Effect::StateChanged));
        assert!(state.is_paused());
    }

    #[test]
    fn seek_relative_calculates_correct_position() {
        let mut state = State::default();
        state.handle(Message::Initialize(sample_video_data()));

        // Seek to 50%
        let effect = state.handle(Message::SeekRelative(0.5));
        assert!(matches!(effect, Effect::StateChanged));

        // Position should be at seek target
        if let Some(pos) = state.position() {
            assert!((pos - 60.0).abs() < 0.1); // 50% of 120s = 60s
        }
    }

    #[test]
    fn toggle_loop_changes_state() {
        let mut state = State::default();
        state.handle(Message::Initialize(sample_video_data()));
        assert!(!state.is_loop_enabled());

        state.handle(Message::ToggleLoop);
        assert!(state.is_loop_enabled());

        state.handle(Message::ToggleLoop);
        assert!(!state.is_loop_enabled());
    }

    #[test]
    fn speed_change_returns_new_speed() {
        let mut state = State::default();
        state.handle(Message::Initialize(sample_video_data()));

        let effect = state.handle(Message::IncreaseSpeed);
        if let Effect::SpeedChanged { speed } = effect {
            assert!(speed > 1.0);
        } else {
            panic!("Expected SpeedChanged effect");
        }
    }

    #[test]
    fn playback_state_accessors_work() {
        let mut state = State::default();
        assert!(state.playback_state().is_none());
        assert!(state.duration().is_none());

        state.handle(Message::Initialize(sample_video_data()));

        assert!(state.playback_state().is_some());
        assert_eq!(state.duration(), Some(120.0));
        assert_eq!(state.playback_speed(), 1.0);
    }
}
