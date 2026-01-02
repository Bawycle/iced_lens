// SPDX-License-Identifier: MPL-2.0
//! Overlay timeout domain type for fullscreen UI.
//!
//! This module provides a type-safe wrapper for the fullscreen overlay
//! auto-hide timeout duration in seconds.

use crate::config::{
    DEFAULT_OVERLAY_TIMEOUT_SECS, MAX_OVERLAY_TIMEOUT_SECS, MIN_OVERLAY_TIMEOUT_SECS,
};

/// Overlay timeout in seconds for fullscreen mode.
///
/// This newtype enforces validity at the type level, ensuring the value
/// is always within the valid range (1–30 seconds).
///
/// # Example
///
/// ```
/// use iced_lens::ui::state::OverlayTimeout;
///
/// let timeout = OverlayTimeout::new(5);
/// assert_eq!(timeout.value(), 5);
///
/// // Values outside range are clamped
/// let too_high = OverlayTimeout::new(100);
/// assert_eq!(too_high.value(), 30); // Clamped to max
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverlayTimeout(u32);

impl OverlayTimeout {
    /// Creates a new overlay timeout value, clamping to valid range.
    #[must_use]
    pub fn new(value: u32) -> Self {
        Self(value.clamp(MIN_OVERLAY_TIMEOUT_SECS, MAX_OVERLAY_TIMEOUT_SECS))
    }

    /// Returns the value as u32.
    #[must_use]
    pub fn value(self) -> u32 {
        self.0
    }

    /// Returns the timeout as a Duration.
    #[must_use]
    pub fn as_duration(self) -> std::time::Duration {
        std::time::Duration::from_secs(u64::from(self.0))
    }

    /// Returns true if this is the minimum value.
    #[must_use]
    pub fn is_min(self) -> bool {
        self.0 <= MIN_OVERLAY_TIMEOUT_SECS
    }

    /// Returns true if this is the maximum value.
    #[must_use]
    pub fn is_max(self) -> bool {
        self.0 >= MAX_OVERLAY_TIMEOUT_SECS
    }
}

impl Default for OverlayTimeout {
    fn default() -> Self {
        Self(DEFAULT_OVERLAY_TIMEOUT_SECS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clamps_to_valid_range() {
        assert_eq!(OverlayTimeout::new(0).value(), MIN_OVERLAY_TIMEOUT_SECS);
        assert_eq!(OverlayTimeout::new(100).value(), MAX_OVERLAY_TIMEOUT_SECS);
    }

    #[test]
    fn new_accepts_valid_values() {
        assert_eq!(OverlayTimeout::new(1).value(), 1);
        assert_eq!(OverlayTimeout::new(15).value(), 15);
        assert_eq!(OverlayTimeout::new(30).value(), 30);
    }

    #[test]
    fn default_returns_expected_value() {
        assert_eq!(
            OverlayTimeout::default().value(),
            DEFAULT_OVERLAY_TIMEOUT_SECS
        );
    }

    #[test]
    fn is_min_detects_minimum() {
        assert!(OverlayTimeout::new(1).is_min());
        assert!(!OverlayTimeout::new(15).is_min());
    }

    #[test]
    fn is_max_detects_maximum() {
        assert!(OverlayTimeout::new(30).is_max());
        assert!(!OverlayTimeout::new(15).is_max());
    }

    #[test]
    fn as_duration_converts_correctly() {
        let timeout = OverlayTimeout::new(5);
        assert_eq!(timeout.as_duration(), std::time::Duration::from_secs(5));
    }

    #[test]
    fn equality_works() {
        assert_eq!(OverlayTimeout::new(5), OverlayTimeout::new(5));
        assert_ne!(OverlayTimeout::new(5), OverlayTimeout::new(10));
    }
}
