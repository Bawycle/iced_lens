// SPDX-License-Identifier: MPL-2.0
//! Volume domain type for audio playback.
//!
//! This module provides a type-safe wrapper for volume values,
//! ensuring they are always within the valid range (0.0–1.5, where 1.0 = 100%).

use crate::config::{DEFAULT_VOLUME, MAX_VOLUME, MIN_VOLUME, VOLUME_STEP};

/// Volume level, guaranteed to be within valid range (0.0–1.5).
///
/// Values above 1.0 represent amplification (up to 150%).
/// This newtype enforces validity at the type level, making it impossible
/// to create an invalid volume value.
///
/// # Example
///
/// ```
/// use iced_lens::video_player::Volume;
///
/// let vol = Volume::new(0.5);
/// assert_eq!(vol.value(), 0.5);
///
/// // Values outside range are clamped
/// let too_loud = Volume::new(2.0);
/// assert_eq!(too_loud.value(), 1.5); // Clamped to max (150%)
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Volume(f32);

impl Volume {
    /// Creates a new volume level, clamping to valid range.
    #[must_use]
    pub fn new(volume: f32) -> Self {
        Self(volume.clamp(MIN_VOLUME, MAX_VOLUME))
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
        Self::new(self.0 + VOLUME_STEP)
    }

    /// Decreases volume by one step, clamping to minimum.
    #[must_use]
    pub fn decrease(self) -> Self {
        Self::new(self.0 - VOLUME_STEP)
    }

    /// Returns true if this is the minimum volume.
    #[must_use]
    pub fn is_min(self) -> bool {
        self.0 <= MIN_VOLUME
    }

    /// Returns true if this is the maximum volume.
    #[must_use]
    pub fn is_max(self) -> bool {
        self.0 >= MAX_VOLUME
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self(DEFAULT_VOLUME)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::assert_abs_diff_eq;

    #[test]
    fn new_clamps_to_valid_range() {
        assert_abs_diff_eq!(Volume::new(-0.5).value(), MIN_VOLUME);
        assert_abs_diff_eq!(Volume::new(1.5).value(), MAX_VOLUME);
        assert_abs_diff_eq!(Volume::new(0.5).value(), 0.5);
    }

    #[test]
    fn default_is_expected_volume() {
        assert_abs_diff_eq!(Volume::default().value(), DEFAULT_VOLUME);
    }

    #[test]
    fn is_muted_detects_zero_volume() {
        assert!(Volume::new(0.0).is_muted());
        assert!(Volume::new(0.0005).is_muted());
        assert!(!Volume::new(0.01).is_muted());
        assert!(!Volume::new(0.5).is_muted());
    }

    #[test]
    fn increase_adds_step() {
        let vol = Volume::new(0.5);
        let louder = vol.increase();
        assert_abs_diff_eq!(louder.value(), 0.5 + VOLUME_STEP, epsilon = 0.001);

        // At max, stays at max
        let max_vol = Volume::new(MAX_VOLUME);
        assert_abs_diff_eq!(max_vol.increase().value(), MAX_VOLUME);
    }

    #[test]
    fn decrease_subtracts_step() {
        let vol = Volume::new(0.5);
        let quieter = vol.decrease();
        assert_abs_diff_eq!(quieter.value(), 0.5 - VOLUME_STEP, epsilon = 0.001);

        // At min, stays at min
        let min_vol = Volume::new(MIN_VOLUME);
        assert_abs_diff_eq!(min_vol.decrease().value(), MIN_VOLUME);
    }

    #[test]
    fn is_min_and_is_max() {
        assert!(Volume::new(MIN_VOLUME).is_min());
        assert!(!Volume::new(0.5).is_min());

        assert!(Volume::new(MAX_VOLUME).is_max());
        assert!(!Volume::new(0.5).is_max());
    }
}
