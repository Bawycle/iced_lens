// SPDX-License-Identifier: MPL-2.0
//! Video playback state machine.
//!
//! This module defines the playback states for video players.

/// Represents the current playback state of a video.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackState {
    /// Video is stopped (at beginning or end).
    #[default]
    Stopped,
    /// Video is currently playing.
    Playing,
    /// Video is paused at current position.
    Paused,
    /// Video is seeking to a new position.
    Seeking,
}

impl PlaybackState {
    /// Returns true if the video is currently playing.
    #[must_use]
    pub fn is_playing(self) -> bool {
        matches!(self, Self::Playing)
    }

    /// Returns true if the video is paused.
    #[must_use]
    pub fn is_paused(self) -> bool {
        matches!(self, Self::Paused)
    }

    /// Returns true if the video is stopped.
    #[must_use]
    pub fn is_stopped(self) -> bool {
        matches!(self, Self::Stopped)
    }

    /// Returns true if the video is seeking.
    #[must_use]
    pub fn is_seeking(self) -> bool {
        matches!(self, Self::Seeking)
    }

    /// Returns true if the video is in an active state (playing or seeking).
    #[must_use]
    pub fn is_active(self) -> bool {
        matches!(self, Self::Playing | Self::Seeking)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_stopped() {
        assert_eq!(PlaybackState::default(), PlaybackState::Stopped);
    }

    #[test]
    fn test_state_checks() {
        assert!(PlaybackState::Playing.is_playing());
        assert!(!PlaybackState::Paused.is_playing());

        assert!(PlaybackState::Paused.is_paused());
        assert!(!PlaybackState::Playing.is_paused());

        assert!(PlaybackState::Stopped.is_stopped());
        assert!(!PlaybackState::Playing.is_stopped());

        assert!(PlaybackState::Seeking.is_seeking());
        assert!(!PlaybackState::Playing.is_seeking());
    }

    #[test]
    fn test_is_active() {
        assert!(PlaybackState::Playing.is_active());
        assert!(PlaybackState::Seeking.is_active());
        assert!(!PlaybackState::Paused.is_active());
        assert!(!PlaybackState::Stopped.is_active());
    }
}
