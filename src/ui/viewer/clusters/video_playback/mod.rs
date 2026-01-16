// SPDX-License-Identifier: MPL-2.0
//! Video playback cluster - video player state, settings, and seek management.
//!
//! This cluster groups all video playback related state:
//! - Video player instance and session management
//! - Playback settings (volume, mute, loop, autoplay)
//! - Seek preview and keyboard seek debouncing
//! - Video-specific fit-to-window setting
//!
//! Note: `VideoShader` (rendering widget) remains in `component.rs` as it's UI-specific.

use crate::media::VideoData;
use crate::video_player::{KeyboardSeekStep, VideoPlayer, Volume};
use std::path::PathBuf;
use std::time::Instant;

/// Video playback cluster state.
///
/// Encapsulates all video playback state including the player, settings,
/// and seek management. Cross-cutting interactions (e.g., seek affecting
/// preview position) are handled within the cluster.
#[allow(clippy::struct_excessive_bools)] // Cluster groups related settings
pub struct State {
    /// The video player instance (created when a video is loaded).
    player: Option<VideoPlayer>,

    /// Path to the current video file.
    current_path: Option<PathBuf>,

    /// Session ID for subscription management.
    /// Incremented each time playback starts to ensure unique subscription IDs.
    session_id: u64,

    // ═══════════════════════════════════════════════════════════════════════
    // PLAYBACK SETTINGS (persisted)
    // ═══════════════════════════════════════════════════════════════════════
    /// Video volume level (0.0 to 1.5).
    volume: f32,

    /// Whether video audio is muted.
    muted: bool,

    /// Whether video playback should loop.
    loop_enabled: bool,

    /// Whether videos should auto-play when loaded.
    autoplay: bool,

    /// Fit-to-window setting for videos (separate from images, not persisted).
    fit_to_window: bool,

    /// Keyboard seek step (arrow keys during video playback).
    keyboard_seek_step: KeyboardSeekStep,

    // ═══════════════════════════════════════════════════════════════════════
    // SEEK STATE
    // ═══════════════════════════════════════════════════════════════════════
    /// Preview position for seek slider in seconds.
    /// Set during slider drag, cleared on release or when frame arrives.
    seek_preview_position: Option<f64>,

    /// Last time a keyboard seek was triggered (for debouncing).
    last_keyboard_seek: Option<Instant>,

    // ═══════════════════════════════════════════════════════════════════════
    // UI STATE
    // ═══════════════════════════════════════════════════════════════════════
    /// Whether the overflow menu (advanced video controls) is open.
    overflow_menu_open: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            player: None,
            current_path: None,
            session_id: 0,
            volume: crate::config::DEFAULT_VOLUME,
            muted: false,
            loop_enabled: false,
            autoplay: false,
            fit_to_window: true, // Videos always fit-to-window by default
            keyboard_seek_step: KeyboardSeekStep::default(),
            seek_preview_position: None,
            last_keyboard_seek: None,
            overflow_menu_open: false,
        }
    }
}

impl Clone for State {
    fn clone(&self) -> Self {
        // VideoPlayer is not Clone, so we can't clone the player
        Self {
            player: None, // Cannot clone VideoPlayer
            current_path: self.current_path.clone(),
            session_id: self.session_id,
            volume: self.volume,
            muted: self.muted,
            loop_enabled: self.loop_enabled,
            autoplay: self.autoplay,
            fit_to_window: self.fit_to_window,
            keyboard_seek_step: self.keyboard_seek_step,
            seek_preview_position: self.seek_preview_position,
            last_keyboard_seek: self.last_keyboard_seek,
            overflow_menu_open: self.overflow_menu_open,
        }
    }
}

/// Messages for the video playback cluster.
#[derive(Debug, Clone)]
pub enum Message {
    // ═══════════════════════════════════════════════════════════════════════
    // PLAYBACK CONTROL
    // ═══════════════════════════════════════════════════════════════════════
    /// Toggle play/pause.
    TogglePlayback,
    /// Play the video.
    Play,
    /// Pause the video.
    Pause,
    /// Stop the video.
    Stop,

    // ═══════════════════════════════════════════════════════════════════════
    // SEEK
    // ═══════════════════════════════════════════════════════════════════════
    /// Preview seek position (during slider drag).
    SeekPreview(f64),
    /// Commit seek to preview position.
    SeekCommit,
    /// Seek relative to current position.
    SeekRelative(f64),
    /// Clear seek preview (when frame arrives near target).
    ClearSeekPreview,

    // ═══════════════════════════════════════════════════════════════════════
    // STEPPING
    // ═══════════════════════════════════════════════════════════════════════
    /// Step forward one frame (only when paused).
    StepForward,
    /// Step backward one frame (only when paused).
    StepBackward,

    // ═══════════════════════════════════════════════════════════════════════
    // SETTINGS
    // ═══════════════════════════════════════════════════════════════════════
    /// Set volume level.
    SetVolume(Volume),
    /// Toggle mute.
    ToggleMute,
    /// Toggle loop.
    ToggleLoop,
    /// Set autoplay.
    SetAutoplay(bool),
    /// Set fit-to-window.
    SetFitToWindow(bool),
    /// Set keyboard seek step.
    SetKeyboardSeekStep(KeyboardSeekStep),

    // ═══════════════════════════════════════════════════════════════════════
    // SPEED
    // ═══════════════════════════════════════════════════════════════════════
    /// Increase playback speed.
    IncreaseSpeed,
    /// Decrease playback speed.
    DecreaseSpeed,

    // ═══════════════════════════════════════════════════════════════════════
    // UI
    // ═══════════════════════════════════════════════════════════════════════
    /// Toggle overflow menu visibility.
    ToggleOverflowMenu,
    /// Close overflow menu.
    CloseOverflowMenu,

    // ═══════════════════════════════════════════════════════════════════════
    // LIFECYCLE
    // ═══════════════════════════════════════════════════════════════════════
    /// Reset for new media (clears video state).
    ResetForNewMedia,
}

/// Effects produced by video playback operations.
#[derive(Debug, Clone)]
pub enum Effect {
    /// No effect.
    None,
    /// Preferences should be persisted.
    PersistPreferences,
    /// Player was created and needs command sender from subscription.
    PlayerCreated,
    /// Playback started - session ID changed.
    SessionChanged,
}

/// Seek debounce duration in milliseconds.
const SEEK_DEBOUNCE_MS: u64 = 200;

impl State {
    /// Create a new video player from video data.
    ///
    /// Returns `Ok(())` if the player was created successfully.
    /// The caller should handle subscription setup after calling this.
    ///
    /// # Errors
    ///
    /// Returns an error if the video player fails to initialize (e.g., invalid video data).
    pub fn create_player(
        &mut self,
        video_data: &VideoData,
        media_path: Option<PathBuf>,
        diagnostics: Option<crate::diagnostics::DiagnosticsHandle>,
    ) -> Result<(), crate::error::Error> {
        let mut player = VideoPlayer::new(video_data)?;

        // Pass diagnostics handle to the player for state event logging
        if let Some(handle) = diagnostics {
            player.set_diagnostics(handle);
        }

        self.player = Some(player);
        self.current_path = media_path;
        self.session_id = self.session_id.wrapping_add(1);

        Ok(())
    }

    /// Handle a cluster message.
    #[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
    pub fn handle(&mut self, msg: Message) -> Effect {
        match msg {
            // ═══════════════════════════════════════════════════════════════
            // PLAYBACK CONTROL
            // ═══════════════════════════════════════════════════════════════
            Message::TogglePlayback => {
                if let Some(player) = &mut self.player {
                    if player.state().is_playing_or_will_resume() {
                        player.pause();
                    } else {
                        self.seek_preview_position = None;
                        player.play();
                    }
                }
                Effect::None
            }
            Message::Play => {
                if let Some(player) = &mut self.player {
                    self.seek_preview_position = None;
                    player.play();
                }
                Effect::None
            }
            Message::Pause => {
                if let Some(player) = &mut self.player {
                    player.pause();
                }
                Effect::None
            }
            Message::Stop => {
                if let Some(player) = &mut self.player {
                    player.stop();
                }
                Effect::None
            }

            // ═══════════════════════════════════════════════════════════════
            // SEEK
            // ═══════════════════════════════════════════════════════════════
            Message::SeekPreview(position) => {
                self.seek_preview_position = Some(position);
                Effect::None
            }
            Message::SeekCommit => {
                if let Some(target_secs) = self.seek_preview_position {
                    if let Some(player) = &mut self.player {
                        player.seek(target_secs);
                    }
                }
                Effect::None
            }
            Message::SeekRelative(delta_secs) => {
                let now = Instant::now();
                let should_seek = match self.last_keyboard_seek {
                    Some(last) => {
                        now.duration_since(last).as_millis() >= u128::from(SEEK_DEBOUNCE_MS)
                    }
                    None => true,
                };

                if should_seek {
                    if let Some(player) = &mut self.player {
                        let base_position = self
                            .seek_preview_position
                            .or_else(|| player.state().position());

                        if let Some(current_pos) = base_position {
                            let duration = player.video_data().duration_secs;
                            let target_secs = (current_pos + delta_secs).max(0.0).min(duration);

                            self.seek_preview_position = Some(target_secs);
                            player.seek(target_secs);
                            self.last_keyboard_seek = Some(now);
                        }
                    }
                }
                Effect::None
            }
            Message::ClearSeekPreview => {
                self.seek_preview_position = None;
                Effect::None
            }

            // ═══════════════════════════════════════════════════════════════
            // STEPPING
            // ═══════════════════════════════════════════════════════════════
            Message::StepForward => {
                if let Some(player) = &mut self.player {
                    if player.state().is_paused() {
                        self.seek_preview_position = None;
                        player.step_frame();
                    }
                }
                Effect::None
            }
            Message::StepBackward => {
                if let Some(player) = &mut self.player {
                    if player.state().is_paused() {
                        self.seek_preview_position = None;
                        player.step_backward();
                    }
                }
                Effect::None
            }

            // ═══════════════════════════════════════════════════════════════
            // SETTINGS
            // ═══════════════════════════════════════════════════════════════
            Message::SetVolume(volume) => {
                self.volume = volume.value();
                if let Some(player) = &self.player {
                    player.set_volume(volume);
                }
                Effect::PersistPreferences
            }
            Message::ToggleMute => {
                self.muted = !self.muted;
                if let Some(player) = &self.player {
                    player.set_muted(self.muted);
                }
                Effect::PersistPreferences
            }
            Message::ToggleLoop => {
                self.loop_enabled = !self.loop_enabled;
                if let Some(player) = &mut self.player {
                    player.set_loop(self.loop_enabled);
                }
                Effect::PersistPreferences
            }
            Message::SetAutoplay(enabled) => {
                self.autoplay = enabled;
                Effect::None
            }
            Message::SetFitToWindow(enabled) => {
                self.fit_to_window = enabled;
                Effect::None
            }
            Message::SetKeyboardSeekStep(step) => {
                self.keyboard_seek_step = step;
                Effect::None
            }

            // ═══════════════════════════════════════════════════════════════
            // SPEED
            // ═══════════════════════════════════════════════════════════════
            Message::IncreaseSpeed => {
                if let Some(player) = &mut self.player {
                    player.increase_playback_speed();
                    let effective_muted = self.muted || player.is_speed_auto_muted();
                    player.set_muted(effective_muted);
                }
                Effect::None
            }
            Message::DecreaseSpeed => {
                if let Some(player) = &mut self.player {
                    player.decrease_playback_speed();
                    let effective_muted = self.muted || player.is_speed_auto_muted();
                    player.set_muted(effective_muted);
                }
                Effect::None
            }

            // ═══════════════════════════════════════════════════════════════
            // UI
            // ═══════════════════════════════════════════════════════════════
            Message::ToggleOverflowMenu => {
                self.overflow_menu_open = !self.overflow_menu_open;
                Effect::None
            }
            Message::CloseOverflowMenu => {
                self.overflow_menu_open = false;
                Effect::None
            }

            // ═══════════════════════════════════════════════════════════════
            // LIFECYCLE
            // ═══════════════════════════════════════════════════════════════
            Message::ResetForNewMedia => {
                // Stop and clear any existing player
                if let Some(player) = &mut self.player {
                    player.stop();
                }
                self.player = None;
                self.current_path = None;
                self.seek_preview_position = None;
                self.last_keyboard_seek = None;
                self.overflow_menu_open = false;
                // Reset fit-to-window to default for new media
                self.fit_to_window = true;
                // Keep other settings (volume, muted, loop, autoplay, keyboard_seek_step)
                // as they are user preferences
                Effect::None
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PLAYER ACCESS (for external handlers that need direct player access)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get a reference to the video player.
    #[must_use]
    pub fn player(&self) -> Option<&VideoPlayer> {
        self.player.as_ref()
    }

    /// Get a mutable reference to the video player.
    pub fn player_mut(&mut self) -> Option<&mut VideoPlayer> {
        self.player.as_mut()
    }

    /// Check if a video player exists.
    #[must_use]
    pub fn has_player(&self) -> bool {
        self.player.is_some()
    }

    /// Clear the video player.
    pub fn clear_player(&mut self) {
        if let Some(player) = &mut self.player {
            player.stop();
        }
        self.player = None;
        self.current_path = None;
    }

    /// Set the video player directly (used when creating from external code).
    pub fn set_player(&mut self, player: VideoPlayer, path: Option<PathBuf>) {
        self.player = Some(player);
        self.current_path = path;
        self.session_id = self.session_id.wrapping_add(1);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ACCESSORS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get the current video path.
    #[must_use]
    pub fn current_path(&self) -> Option<&PathBuf> {
        self.current_path.as_ref()
    }

    /// Get the playback session ID.
    #[must_use]
    pub fn session_id(&self) -> u64 {
        self.session_id
    }

    /// Increment session ID (used when starting new subscription).
    pub fn increment_session_id(&mut self) {
        self.session_id = self.session_id.wrapping_add(1);
    }

    /// Get the volume level.
    #[must_use]
    pub fn volume(&self) -> f32 {
        self.volume
    }

    /// Set volume directly (without going through message).
    pub fn set_volume_raw(&mut self, volume: f32) {
        self.volume = volume.clamp(crate::config::MIN_VOLUME, crate::config::MAX_VOLUME);
    }

    /// Check if muted.
    #[must_use]
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Set muted directly.
    pub fn set_muted_raw(&mut self, muted: bool) {
        self.muted = muted;
    }

    /// Check if loop is enabled.
    #[must_use]
    pub fn is_loop_enabled(&self) -> bool {
        self.loop_enabled
    }

    /// Set loop directly.
    pub fn set_loop_raw(&mut self, enabled: bool) {
        self.loop_enabled = enabled;
    }

    /// Check if autoplay is enabled.
    #[must_use]
    pub fn is_autoplay(&self) -> bool {
        self.autoplay
    }

    /// Check if fit-to-window is enabled.
    #[must_use]
    pub fn fit_to_window(&self) -> bool {
        self.fit_to_window
    }

    /// Get the keyboard seek step.
    #[must_use]
    pub fn keyboard_seek_step(&self) -> KeyboardSeekStep {
        self.keyboard_seek_step
    }

    /// Get the seek preview position.
    #[must_use]
    pub fn seek_preview_position(&self) -> Option<f64> {
        self.seek_preview_position
    }

    /// Set seek preview position directly.
    pub fn set_seek_preview_position(&mut self, position: Option<f64>) {
        self.seek_preview_position = position;
    }

    /// Check if overflow menu is open.
    #[must_use]
    pub fn is_overflow_menu_open(&self) -> bool {
        self.overflow_menu_open
    }

    /// Check if video is currently playing or will resume.
    #[must_use]
    pub fn is_playing_or_will_resume(&self) -> bool {
        self.player
            .as_ref()
            .is_some_and(|p| p.state().is_playing_or_will_resume())
    }

    /// Check if player has an active session (not stopped or error).
    #[must_use]
    pub fn has_active_session(&self) -> bool {
        self.player.as_ref().is_some_and(|p| {
            !matches!(
                p.state(),
                crate::video_player::PlaybackState::Stopped
                    | crate::video_player::PlaybackState::Error { .. }
            )
        })
    }

    /// Get current video position in seconds.
    #[must_use]
    pub fn position(&self) -> Option<f64> {
        self.player.as_ref().and_then(|p| p.state().position())
    }

    /// Get current playback speed.
    #[must_use]
    pub fn playback_speed(&self) -> Option<f64> {
        self.player.as_ref().map(VideoPlayer::playback_speed)
    }

    /// Check if seek preview should be cleared based on frame PTS.
    ///
    /// Returns true if the preview was cleared.
    pub fn maybe_clear_seek_preview(&mut self, pts_secs: f64) -> bool {
        if let Some(preview_secs) = self.seek_preview_position {
            let diff = (pts_secs - preview_secs).abs();
            if diff < 0.5 {
                self.seek_preview_position = None;
                return true;
            }
        }
        false
    }

    /// Apply current settings to player (called after subscription starts).
    pub fn apply_settings_to_player(&mut self) {
        if let Some(player) = &mut self.player {
            player.set_volume(Volume::new(self.volume));
            player.set_muted(self.muted);
            player.set_loop(self.loop_enabled);
        }
    }

    /// Handle end of stream based on loop setting.
    pub fn handle_end_of_stream(&mut self) {
        if let Some(player) = &mut self.player {
            player.set_at_end_of_stream();

            if self.loop_enabled {
                self.seek_preview_position = None;
                player.seek(0.0);
                player.play();
            } else {
                let duration = player.video_data().duration_secs;
                player.pause_at(duration);
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn default_state_has_expected_values() {
        let state = State::default();
        assert!(!state.has_player());
        assert!(state.current_path().is_none());
        assert_eq!(state.session_id(), 0);
        assert!((state.volume() - crate::config::DEFAULT_VOLUME).abs() < f32::EPSILON);
        assert!(!state.is_muted());
        assert!(!state.is_loop_enabled());
        assert!(!state.is_autoplay());
        assert!(state.fit_to_window());
        assert!(state.seek_preview_position().is_none());
        assert!(!state.is_overflow_menu_open());
    }

    #[test]
    fn toggle_mute_flips_state() {
        let mut state = State::default();
        assert!(!state.is_muted());

        state.handle(Message::ToggleMute);
        assert!(state.is_muted());

        state.handle(Message::ToggleMute);
        assert!(!state.is_muted());
    }

    #[test]
    fn toggle_loop_flips_state() {
        let mut state = State::default();
        assert!(!state.is_loop_enabled());

        state.handle(Message::ToggleLoop);
        assert!(state.is_loop_enabled());

        state.handle(Message::ToggleLoop);
        assert!(!state.is_loop_enabled());
    }

    #[test]
    fn set_volume_updates_value() {
        let mut state = State::default();
        let effect = state.handle(Message::SetVolume(Volume::new(0.75)));

        assert!((state.volume() - 0.75).abs() < f32::EPSILON);
        assert!(matches!(effect, Effect::PersistPreferences));
    }

    #[test]
    fn seek_preview_sets_position() {
        let mut state = State::default();
        assert!(state.seek_preview_position().is_none());

        state.handle(Message::SeekPreview(5.5));
        assert_eq!(state.seek_preview_position(), Some(5.5));

        state.handle(Message::ClearSeekPreview);
        assert!(state.seek_preview_position().is_none());
    }

    #[test]
    fn toggle_overflow_menu_flips_state() {
        let mut state = State::default();
        assert!(!state.is_overflow_menu_open());

        state.handle(Message::ToggleOverflowMenu);
        assert!(state.is_overflow_menu_open());

        state.handle(Message::ToggleOverflowMenu);
        assert!(!state.is_overflow_menu_open());
    }

    #[test]
    fn reset_for_new_media_clears_state() {
        let mut state = State::default();
        state.seek_preview_position = Some(10.0);
        state.overflow_menu_open = true;
        state.fit_to_window = false;

        state.handle(Message::ResetForNewMedia);

        assert!(state.seek_preview_position().is_none());
        assert!(!state.is_overflow_menu_open());
        assert!(state.fit_to_window()); // Reset to default
    }

    #[test]
    fn maybe_clear_seek_preview_clears_when_close() {
        let mut state = State::default();
        state.seek_preview_position = Some(10.0);

        // Frame far from target - don't clear
        assert!(!state.maybe_clear_seek_preview(5.0));
        assert!(state.seek_preview_position().is_some());

        // Frame close to target - clear
        assert!(state.maybe_clear_seek_preview(10.3));
        assert!(state.seek_preview_position().is_none());
    }

    #[test]
    fn set_autoplay_updates_value() {
        let mut state = State::default();
        assert!(!state.is_autoplay());

        state.handle(Message::SetAutoplay(true));
        assert!(state.is_autoplay());

        state.handle(Message::SetAutoplay(false));
        assert!(!state.is_autoplay());
    }

    #[test]
    fn set_fit_to_window_updates_value() {
        let mut state = State::default();
        assert!(state.fit_to_window());

        state.handle(Message::SetFitToWindow(false));
        assert!(!state.fit_to_window());

        state.handle(Message::SetFitToWindow(true));
        assert!(state.fit_to_window());
    }
}
