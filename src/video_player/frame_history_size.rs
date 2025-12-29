// SPDX-License-Identifier: MPL-2.0
//! Frame history size domain type for video playback.
//!
//! This module provides a type-safe wrapper for the video frame history size
//! in megabytes, used for backward frame stepping.

use crate::config::{DEFAULT_FRAME_HISTORY_MB, MAX_FRAME_HISTORY_MB, MIN_FRAME_HISTORY_MB};

/// Frame history size in megabytes for backward video frame stepping.
///
/// This newtype enforces validity at the type level, ensuring the value
/// is always within the valid range (32–512 MB).
///
/// # Example
///
/// ```
/// use iced_lens::video_player::FrameHistoryMb;
///
/// let history = FrameHistoryMb::new(256);
/// assert_eq!(history.value(), 256);
///
/// // Values outside range are clamped
/// let too_high = FrameHistoryMb::new(1000);
/// assert_eq!(too_high.value(), 512); // Clamped to max
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameHistoryMb(u32);

impl FrameHistoryMb {
    /// Creates a new frame history size, clamping to valid range.
    #[must_use] 
    pub fn new(value: u32) -> Self {
        Self(value.clamp(MIN_FRAME_HISTORY_MB, MAX_FRAME_HISTORY_MB))
    }

    /// Returns the value as u32.
    #[must_use] 
    pub fn value(self) -> u32 {
        self.0
    }

    /// Returns true if this is the minimum value.
    #[must_use] 
    pub fn is_min(self) -> bool {
        self.0 <= MIN_FRAME_HISTORY_MB
    }

    /// Returns true if this is the maximum value.
    #[must_use] 
    pub fn is_max(self) -> bool {
        self.0 >= MAX_FRAME_HISTORY_MB
    }
}

impl Default for FrameHistoryMb {
    fn default() -> Self {
        Self(DEFAULT_FRAME_HISTORY_MB)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clamps_to_valid_range() {
        assert_eq!(FrameHistoryMb::new(0).value(), MIN_FRAME_HISTORY_MB);
        assert_eq!(FrameHistoryMb::new(1000).value(), MAX_FRAME_HISTORY_MB);
    }

    #[test]
    fn new_accepts_valid_values() {
        assert_eq!(FrameHistoryMb::new(32).value(), 32);
        assert_eq!(FrameHistoryMb::new(256).value(), 256);
        assert_eq!(FrameHistoryMb::new(512).value(), 512);
    }

    #[test]
    fn default_returns_expected_value() {
        assert_eq!(FrameHistoryMb::default().value(), DEFAULT_FRAME_HISTORY_MB);
    }

    #[test]
    fn is_min_detects_minimum() {
        assert!(FrameHistoryMb::new(32).is_min());
        assert!(!FrameHistoryMb::new(256).is_min());
    }

    #[test]
    fn is_max_detects_maximum() {
        assert!(FrameHistoryMb::new(512).is_max());
        assert!(!FrameHistoryMb::new(256).is_max());
    }

    #[test]
    fn equality_works() {
        assert_eq!(FrameHistoryMb::new(128), FrameHistoryMb::new(128));
        assert_ne!(FrameHistoryMb::new(128), FrameHistoryMb::new(256));
    }
}
