// SPDX-License-Identifier: MPL-2.0
//! Video playback newtypes.
//!
//! This module provides type-safe wrappers for video playback values,
//! ensuring they are always within valid ranges.

use std::time::Duration;

// =============================================================================
// Volume
// =============================================================================

/// Volume bounds (0.0 to 1.5, where 1.0 = 100%).
pub mod volume_bounds {
    /// Minimum volume level.
    pub const MIN: f32 = 0.0;
    /// Maximum volume level (1.5 = 150% amplification).
    pub const MAX: f32 = 1.5;
    /// Default volume level.
    pub const DEFAULT: f32 = 0.8;
    /// Volume adjustment step per key press (5%).
    pub const STEP: f32 = 0.05;
}

/// Volume level, guaranteed to be within valid range (0.0–1.5).
///
/// Values above 1.0 represent amplification (up to 150%).
/// This newtype enforces validity at the type level, making it impossible
/// to create an invalid volume value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Volume(f32);

impl Volume {
    /// Creates a new volume level, clamping to valid range.
    #[must_use]
    pub fn new(volume: f32) -> Self {
        Self(volume.clamp(volume_bounds::MIN, volume_bounds::MAX))
    }

    /// Returns the volume value as f32.
    #[must_use]
    pub fn value(self) -> f32 {
        self.0
    }

    /// Returns true if volume is effectively muted (below audible threshold).
    #[must_use]
    pub fn is_muted(self) -> bool {
        self.0 < 0.001
    }

    /// Increases volume by one step, clamping to maximum.
    #[must_use]
    pub fn increase(self) -> Self {
        Self::new(self.0 + volume_bounds::STEP)
    }

    /// Decreases volume by one step, clamping to minimum.
    #[must_use]
    pub fn decrease(self) -> Self {
        Self::new(self.0 - volume_bounds::STEP)
    }

    /// Returns true if this is the minimum volume.
    #[must_use]
    pub fn is_min(self) -> bool {
        self.0 <= volume_bounds::MIN
    }

    /// Returns true if this is the maximum volume.
    #[must_use]
    pub fn is_max(self) -> bool {
        self.0 >= volume_bounds::MAX
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self(volume_bounds::DEFAULT)
    }
}

// =============================================================================
// PlaybackSpeed
// =============================================================================

/// Playback speed bounds (0.1x to 8.0x).
pub mod speed_bounds {
    /// Minimum playback speed (0.1x = ten times slower).
    pub const MIN: f64 = 0.1;
    /// Maximum playback speed (8x = eight times faster).
    pub const MAX: f64 = 8.0;
    /// Default playback speed (1.0 = normal speed).
    pub const DEFAULT: f64 = 1.0;
    /// Speed threshold above which audio is automatically muted.
    pub const AUTO_MUTE_THRESHOLD: f64 = 2.0;
    /// Playback speed presets for cycling with J/L keys.
    pub const PRESETS: &[f64] = &[
        0.1, 0.15, 0.2, 0.25, 0.33, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 4.0, 8.0,
    ];
}

/// Playback speed value, guaranteed to be within valid range (0.1x - 8.0x).
///
/// This newtype enforces validity at the type level, making it impossible
/// to create an invalid playback speed value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaybackSpeed(f64);

impl PlaybackSpeed {
    /// Creates a new playback speed, clamping to valid range.
    #[must_use]
    pub fn new(speed: f64) -> Self {
        Self(speed.clamp(speed_bounds::MIN, speed_bounds::MAX))
    }

    /// Returns the speed value as f64.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }

    /// Returns true if audio should be auto-muted at this speed.
    #[must_use]
    pub fn should_auto_mute(self) -> bool {
        self.0 > speed_bounds::AUTO_MUTE_THRESHOLD
    }

    /// Returns the next higher preset speed, or self if at maximum.
    #[must_use]
    pub fn increase(self) -> Self {
        let next = speed_bounds::PRESETS
            .iter()
            .find(|&&s| s > self.0 + 0.001)
            .copied()
            .unwrap_or(self.0);
        Self(next)
    }

    /// Returns the next lower preset speed, or self if at minimum.
    #[must_use]
    pub fn decrease(self) -> Self {
        let prev = speed_bounds::PRESETS
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
        (self.0 - speed_bounds::MIN).abs() < 0.001
    }

    /// Returns true if this is the maximum speed.
    #[must_use]
    pub fn is_max(self) -> bool {
        (self.0 - speed_bounds::MAX).abs() < 0.001
    }
}

impl Default for PlaybackSpeed {
    fn default() -> Self {
        Self(speed_bounds::DEFAULT)
    }
}

// =============================================================================
// KeyboardSeekStep
// =============================================================================

/// Keyboard seek step bounds (0.5 to 30.0 seconds).
pub mod seek_step_bounds {
    /// Minimum keyboard seek step in seconds.
    pub const MIN: f64 = 0.5;
    /// Maximum keyboard seek step in seconds.
    pub const MAX: f64 = 30.0;
    /// Default keyboard seek step in seconds.
    pub const DEFAULT: f64 = 2.0;
}

/// Keyboard seek step in seconds for video navigation.
///
/// This newtype enforces validity at the type level, ensuring the value
/// is always within the valid range (0.5–30.0 seconds).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyboardSeekStep(f64);

impl KeyboardSeekStep {
    /// Creates a new keyboard seek step value, clamping to valid range.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(seek_step_bounds::MIN, seek_step_bounds::MAX))
    }

    /// Returns the value as f64.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }

    /// Returns the step as a Duration.
    #[must_use]
    pub fn as_duration(self) -> Duration {
        Duration::from_secs_f64(self.0)
    }

    /// Returns true if this is the minimum value.
    #[must_use]
    pub fn is_min(self) -> bool {
        self.0 <= seek_step_bounds::MIN
    }

    /// Returns true if this is the maximum value.
    #[must_use]
    pub fn is_max(self) -> bool {
        self.0 >= seek_step_bounds::MAX
    }
}

impl Default for KeyboardSeekStep {
    fn default() -> Self {
        Self(seek_step_bounds::DEFAULT)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Volume tests
    // -------------------------------------------------------------------------

    #[test]
    fn volume_clamps_to_valid_range() {
        assert!((Volume::new(-0.5).value() - volume_bounds::MIN).abs() < f32::EPSILON);
        assert!((Volume::new(2.0).value() - volume_bounds::MAX).abs() < f32::EPSILON);
        assert!((Volume::new(0.5).value() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn volume_default_is_expected() {
        assert!((Volume::default().value() - volume_bounds::DEFAULT).abs() < f32::EPSILON);
    }

    #[test]
    fn volume_is_muted_detects_zero() {
        assert!(Volume::new(0.0).is_muted());
        assert!(Volume::new(0.0005).is_muted());
        assert!(!Volume::new(0.01).is_muted());
    }

    #[test]
    fn volume_increase_and_decrease() {
        let vol = Volume::new(0.5);
        let louder = vol.increase();
        assert!(louder.value() > vol.value());

        let quieter = vol.decrease();
        assert!(quieter.value() < vol.value());
    }

    #[test]
    fn volume_min_max_checks() {
        assert!(Volume::new(volume_bounds::MIN).is_min());
        assert!(Volume::new(volume_bounds::MAX).is_max());
        assert!(!Volume::new(0.5).is_min());
        assert!(!Volume::new(0.5).is_max());
    }

    // -------------------------------------------------------------------------
    // PlaybackSpeed tests
    // -------------------------------------------------------------------------

    #[test]
    fn speed_clamps_to_valid_range() {
        assert!((PlaybackSpeed::new(0.01).value() - speed_bounds::MIN).abs() < 0.001);
        assert!((PlaybackSpeed::new(100.0).value() - speed_bounds::MAX).abs() < 0.001);
        assert!((PlaybackSpeed::new(2.0).value() - 2.0).abs() < 0.001);
    }

    #[test]
    fn speed_default_is_normal() {
        assert!((PlaybackSpeed::default().value() - speed_bounds::DEFAULT).abs() < 0.001);
    }

    #[test]
    fn speed_auto_mute_above_threshold() {
        assert!(!PlaybackSpeed::new(1.0).should_auto_mute());
        assert!(!PlaybackSpeed::new(2.0).should_auto_mute());
        assert!(PlaybackSpeed::new(2.5).should_auto_mute());
    }

    #[test]
    fn speed_increase_and_decrease() {
        let speed = PlaybackSpeed::new(1.0);
        let faster = speed.increase();
        assert!(faster.value() > speed.value());

        let slower = speed.decrease();
        assert!(slower.value() < speed.value());
    }

    #[test]
    fn speed_min_max_checks() {
        assert!(PlaybackSpeed::new(speed_bounds::MIN).is_min());
        assert!(PlaybackSpeed::new(speed_bounds::MAX).is_max());
        assert!(!PlaybackSpeed::new(1.0).is_min());
        assert!(!PlaybackSpeed::new(1.0).is_max());
    }

    // -------------------------------------------------------------------------
    // KeyboardSeekStep tests
    // -------------------------------------------------------------------------

    #[test]
    fn seek_step_clamps_to_valid_range() {
        assert!((KeyboardSeekStep::new(0.0).value() - seek_step_bounds::MIN).abs() < 0.001);
        assert!((KeyboardSeekStep::new(100.0).value() - seek_step_bounds::MAX).abs() < 0.001);
        assert!((KeyboardSeekStep::new(5.0).value() - 5.0).abs() < 0.001);
    }

    #[test]
    fn seek_step_default_is_expected() {
        assert!(
            (KeyboardSeekStep::default().value() - seek_step_bounds::DEFAULT).abs() < 0.001
        );
    }

    #[test]
    fn seek_step_min_max_checks() {
        assert!(KeyboardSeekStep::new(seek_step_bounds::MIN).is_min());
        assert!(KeyboardSeekStep::new(seek_step_bounds::MAX).is_max());
        assert!(!KeyboardSeekStep::new(5.0).is_min());
        assert!(!KeyboardSeekStep::new(5.0).is_max());
    }

    #[test]
    fn seek_step_as_duration() {
        let step = KeyboardSeekStep::new(2.5);
        assert_eq!(step.as_duration(), Duration::from_secs_f64(2.5));
    }
}
