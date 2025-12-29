// SPDX-License-Identifier: MPL-2.0
//! Frame cache size domain type for video playback.
//!
//! This module provides a type-safe wrapper for the video frame cache size
//! in megabytes.

use crate::config::{DEFAULT_FRAME_CACHE_MB, MAX_FRAME_CACHE_MB, MIN_FRAME_CACHE_MB};

/// Frame cache size in megabytes for video playback.
///
/// This newtype enforces validity at the type level, ensuring the value
/// is always within the valid range (16–512 MB).
///
/// # Example
///
/// ```
/// use iced_lens::video_player::FrameCacheMb;
///
/// let cache = FrameCacheMb::new(128);
/// assert_eq!(cache.value(), 128);
///
/// // Values outside range are clamped
/// let too_high = FrameCacheMb::new(1000);
/// assert_eq!(too_high.value(), 512); // Clamped to max
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameCacheMb(u32);

impl FrameCacheMb {
    /// Creates a new frame cache size, clamping to valid range.
    #[must_use] 
    pub fn new(value: u32) -> Self {
        Self(value.clamp(MIN_FRAME_CACHE_MB, MAX_FRAME_CACHE_MB))
    }

    /// Returns the value as u32.
    #[must_use] 
    pub fn value(self) -> u32 {
        self.0
    }

    /// Returns true if this is the minimum value.
    #[must_use] 
    pub fn is_min(self) -> bool {
        self.0 <= MIN_FRAME_CACHE_MB
    }

    /// Returns true if this is the maximum value.
    #[must_use] 
    pub fn is_max(self) -> bool {
        self.0 >= MAX_FRAME_CACHE_MB
    }
}

impl Default for FrameCacheMb {
    fn default() -> Self {
        Self(DEFAULT_FRAME_CACHE_MB)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clamps_to_valid_range() {
        assert_eq!(FrameCacheMb::new(0).value(), MIN_FRAME_CACHE_MB);
        assert_eq!(FrameCacheMb::new(1000).value(), MAX_FRAME_CACHE_MB);
    }

    #[test]
    fn new_accepts_valid_values() {
        assert_eq!(FrameCacheMb::new(16).value(), 16);
        assert_eq!(FrameCacheMb::new(128).value(), 128);
        assert_eq!(FrameCacheMb::new(512).value(), 512);
    }

    #[test]
    fn default_returns_expected_value() {
        assert_eq!(FrameCacheMb::default().value(), DEFAULT_FRAME_CACHE_MB);
    }

    #[test]
    fn is_min_detects_minimum() {
        assert!(FrameCacheMb::new(16).is_min());
        assert!(!FrameCacheMb::new(128).is_min());
    }

    #[test]
    fn is_max_detects_maximum() {
        assert!(FrameCacheMb::new(512).is_max());
        assert!(!FrameCacheMb::new(128).is_max());
    }

    #[test]
    fn equality_works() {
        assert_eq!(FrameCacheMb::new(64), FrameCacheMb::new(64));
        assert_ne!(FrameCacheMb::new(64), FrameCacheMb::new(128));
    }
}
