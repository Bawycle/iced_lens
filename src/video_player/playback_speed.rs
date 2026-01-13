// SPDX-License-Identifier: MPL-2.0
//! Playback speed domain type for video playback.
//!
//! This module provides a type-safe wrapper for playback speed values,
//! ensuring they are always within the valid range (0.1x - 8.0x).

use crate::config::{
    MAX_PLAYBACK_SPEED, MIN_PLAYBACK_SPEED, PLAYBACK_SPEED_AUTO_MUTE_THRESHOLD,
    PLAYBACK_SPEED_PRESETS,
};

/// Playback speed value, guaranteed to be within valid range (0.1x - 8.0x).
///
/// This newtype enforces validity at the type level, making it impossible
/// to create an invalid playback speed value.
///
/// # Example
///
/// ```
/// use iced_lens::video_player::PlaybackSpeed;
///
/// let speed = PlaybackSpeed::new(2.0);
/// assert_eq!(speed.value(), 2.0);
///
/// // Values outside range are clamped
/// let too_fast = PlaybackSpeed::new(100.0);
/// assert_eq!(too_fast.value(), 8.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaybackSpeed(f64);

impl PlaybackSpeed {
    /// Creates a new playback speed, clamping to valid range.
    #[must_use]
    pub fn new(speed: f64) -> Self {
        Self(speed.clamp(MIN_PLAYBACK_SPEED, MAX_PLAYBACK_SPEED))
    }

    /// Returns the speed value as f64.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }

    /// Returns true if audio should be auto-muted at this speed.
    #[must_use]
    pub fn should_auto_mute(self) -> bool {
        self.0 > PLAYBACK_SPEED_AUTO_MUTE_THRESHOLD
    }

    /// Returns the next higher preset speed, or self if at maximum.
    #[must_use]
    pub fn increase(self) -> Self {
        let next = PLAYBACK_SPEED_PRESETS
            .iter()
            .find(|&&s| s > self.0 + 0.001)
            .copied()
            .unwrap_or(self.0);
        Self(next)
    }

    /// Returns the next lower preset speed, or self if at minimum.
    #[must_use]
    pub fn decrease(self) -> Self {
        let prev = PLAYBACK_SPEED_PRESETS
            .iter()
            .rev()
            .find(|&&s| s < self.0 - 0.001)
            .copied()
            .unwrap_or(self.0);
        Self(prev)
    }

    /// Returns true if this is the minimum speed.
    #[must_use]
    pub fn is_min(self) -> bool {
        (self.0 - MIN_PLAYBACK_SPEED).abs() < 0.001
    }

    /// Returns true if this is the maximum speed.
    #[must_use]
    pub fn is_max(self) -> bool {
        (self.0 - MAX_PLAYBACK_SPEED).abs() < 0.001
    }
}

impl Default for PlaybackSpeed {
    fn default() -> Self {
        Self(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::assert_abs_diff_eq;

    #[test]
    fn new_clamps_to_valid_range() {
        assert_abs_diff_eq!(PlaybackSpeed::new(0.01).value(), MIN_PLAYBACK_SPEED);
        assert_abs_diff_eq!(PlaybackSpeed::new(100.0).value(), MAX_PLAYBACK_SPEED);
        assert_abs_diff_eq!(PlaybackSpeed::new(2.0).value(), 2.0);
    }

    #[test]
    fn default_is_normal_speed() {
        assert_abs_diff_eq!(PlaybackSpeed::default().value(), 1.0);
    }

    #[test]
    fn should_auto_mute_above_threshold() {
        assert!(!PlaybackSpeed::new(1.0).should_auto_mute());
        assert!(!PlaybackSpeed::new(2.0).should_auto_mute());
        assert!(PlaybackSpeed::new(2.5).should_auto_mute());
        assert!(PlaybackSpeed::new(4.0).should_auto_mute());
    }

    #[test]
    fn increase_cycles_through_presets() {
        let speed = PlaybackSpeed::new(1.0);
        let faster = speed.increase();
        assert!(faster.value() > 1.0);

        // At max, stays at max
        let max_speed = PlaybackSpeed::new(MAX_PLAYBACK_SPEED);
        assert_abs_diff_eq!(max_speed.increase().value(), MAX_PLAYBACK_SPEED);
    }

    #[test]
    fn decrease_cycles_through_presets() {
        let speed = PlaybackSpeed::new(1.0);
        let slower = speed.decrease();
        assert!(slower.value() < 1.0);

        // At min, stays at min
        let min_speed = PlaybackSpeed::new(MIN_PLAYBACK_SPEED);
        assert_abs_diff_eq!(min_speed.decrease().value(), MIN_PLAYBACK_SPEED);
    }

    #[test]
    fn is_min_and_is_max() {
        assert!(PlaybackSpeed::new(MIN_PLAYBACK_SPEED).is_min());
        assert!(!PlaybackSpeed::new(1.0).is_min());

        assert!(PlaybackSpeed::new(MAX_PLAYBACK_SPEED).is_max());
        assert!(!PlaybackSpeed::new(1.0).is_max());
    }
}
