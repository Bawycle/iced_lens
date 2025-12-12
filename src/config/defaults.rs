// SPDX-License-Identifier: MPL-2.0
//! Centralized default values for all configuration constants.
//!
//! This module serves as the single source of truth for default values
//! used across the application. Constants are organized by category.
//!
//! # Categories
//!
//! - **Zoom**: Zoom percentage and step bounds
//! - **Overlay**: Fullscreen overlay auto-hide timeout
//! - **Volume**: Audio playback volume settings
//! - **Frame Cache**: Video frame caching for seek performance

// ==========================================================================
// Zoom Defaults
// ==========================================================================

/// Default zoom level when opening an image (100% = original size).
pub const DEFAULT_ZOOM_PERCENT: f32 = 100.0;

/// Minimum allowed zoom percentage.
pub const MIN_ZOOM_PERCENT: f32 = 10.0;

/// Maximum allowed zoom percentage.
pub const MAX_ZOOM_PERCENT: f32 = 800.0;

/// Default zoom step for zoom in/out operations.
pub const DEFAULT_ZOOM_STEP_PERCENT: f32 = 10.0;

/// Minimum allowed zoom step percentage.
pub const MIN_ZOOM_STEP_PERCENT: f32 = 1.0;

/// Maximum allowed zoom step percentage.
pub const MAX_ZOOM_STEP_PERCENT: f32 = 200.0;

// ==========================================================================
// Overlay/Timeout Defaults
// ==========================================================================

/// Default auto-hide timeout for fullscreen overlays (in seconds).
pub const DEFAULT_OVERLAY_TIMEOUT_SECS: u32 = 3;

/// Minimum overlay timeout (in seconds).
pub const MIN_OVERLAY_TIMEOUT_SECS: u32 = 1;

/// Maximum overlay timeout (in seconds).
pub const MAX_OVERLAY_TIMEOUT_SECS: u32 = 30;

// ==========================================================================
// Volume Defaults
// ==========================================================================

/// Default video playback volume (0.0 to 1.0).
pub const DEFAULT_VOLUME: f32 = 0.8;

/// Minimum volume level.
pub const MIN_VOLUME: f32 = 0.0;

/// Maximum volume level.
pub const MAX_VOLUME: f32 = 1.0;

/// Volume adjustment step per key press (5%).
pub const VOLUME_STEP: f32 = 0.05;

/// Target loudness for audio normalization (LUFS).
/// EBU R128 standard uses -23 LUFS, but -16 LUFS is common for streaming.
pub const DEFAULT_NORMALIZATION_TARGET_LUFS: f32 = -16.0;

// ==========================================================================
// Frame Cache Defaults
// ==========================================================================

/// Default frame cache size in megabytes for video seek optimization.
pub const DEFAULT_FRAME_CACHE_MB: u32 = 64;

/// Minimum frame cache size in megabytes.
pub const MIN_FRAME_CACHE_MB: u32 = 16;

/// Maximum frame cache size in megabytes.
pub const MAX_FRAME_CACHE_MB: u32 = 512;

/// Default frame history size in megabytes for backward frame stepping.
pub const DEFAULT_FRAME_HISTORY_MB: u32 = 128;

/// Minimum frame history size in megabytes.
pub const MIN_FRAME_HISTORY_MB: u32 = 32;

/// Maximum frame history size in megabytes.
pub const MAX_FRAME_HISTORY_MB: u32 = 512;

// ==========================================================================
// Compile-time Validation
// ==========================================================================

const _: () = {
    // Zoom validation
    assert!(MIN_ZOOM_PERCENT > 0.0);
    assert!(MIN_ZOOM_PERCENT < DEFAULT_ZOOM_PERCENT);
    assert!(MAX_ZOOM_PERCENT > DEFAULT_ZOOM_PERCENT);
    assert!(MIN_ZOOM_STEP_PERCENT > 0.0);
    assert!(MAX_ZOOM_STEP_PERCENT > MIN_ZOOM_STEP_PERCENT);
    assert!(DEFAULT_ZOOM_STEP_PERCENT >= MIN_ZOOM_STEP_PERCENT);
    assert!(DEFAULT_ZOOM_STEP_PERCENT <= MAX_ZOOM_STEP_PERCENT);

    // Overlay timeout validation
    assert!(MIN_OVERLAY_TIMEOUT_SECS > 0);
    assert!(MAX_OVERLAY_TIMEOUT_SECS >= MIN_OVERLAY_TIMEOUT_SECS);
    assert!(DEFAULT_OVERLAY_TIMEOUT_SECS >= MIN_OVERLAY_TIMEOUT_SECS);
    assert!(DEFAULT_OVERLAY_TIMEOUT_SECS <= MAX_OVERLAY_TIMEOUT_SECS);

    // Frame cache validation
    assert!(MIN_FRAME_CACHE_MB > 0);
    assert!(MAX_FRAME_CACHE_MB >= MIN_FRAME_CACHE_MB);
    assert!(DEFAULT_FRAME_CACHE_MB >= MIN_FRAME_CACHE_MB);
    assert!(DEFAULT_FRAME_CACHE_MB <= MAX_FRAME_CACHE_MB);

    // Frame history validation
    assert!(MIN_FRAME_HISTORY_MB > 0);
    assert!(MAX_FRAME_HISTORY_MB >= MIN_FRAME_HISTORY_MB);
    assert!(DEFAULT_FRAME_HISTORY_MB >= MIN_FRAME_HISTORY_MB);
    assert!(DEFAULT_FRAME_HISTORY_MB <= MAX_FRAME_HISTORY_MB);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_defaults_are_valid() {
        assert_eq!(DEFAULT_ZOOM_PERCENT, 100.0);
        assert!(MIN_ZOOM_PERCENT < DEFAULT_ZOOM_PERCENT);
        assert!(MAX_ZOOM_PERCENT > DEFAULT_ZOOM_PERCENT);
    }

    #[test]
    fn zoom_step_defaults_are_valid() {
        assert_eq!(DEFAULT_ZOOM_STEP_PERCENT, 10.0);
        assert!(DEFAULT_ZOOM_STEP_PERCENT >= MIN_ZOOM_STEP_PERCENT);
        assert!(DEFAULT_ZOOM_STEP_PERCENT <= MAX_ZOOM_STEP_PERCENT);
    }

    #[test]
    fn volume_defaults_are_valid() {
        assert_eq!(DEFAULT_VOLUME, 0.8);
        assert!(DEFAULT_VOLUME >= MIN_VOLUME);
        assert!(DEFAULT_VOLUME <= MAX_VOLUME);
        assert!(VOLUME_STEP > 0.0);
    }

    #[test]
    fn overlay_timeout_defaults_are_valid() {
        assert_eq!(DEFAULT_OVERLAY_TIMEOUT_SECS, 3);
        assert!(DEFAULT_OVERLAY_TIMEOUT_SECS >= MIN_OVERLAY_TIMEOUT_SECS);
        assert!(DEFAULT_OVERLAY_TIMEOUT_SECS <= MAX_OVERLAY_TIMEOUT_SECS);
    }

    #[test]
    fn frame_cache_defaults_are_valid() {
        assert_eq!(DEFAULT_FRAME_CACHE_MB, 64);
        assert!(DEFAULT_FRAME_CACHE_MB >= MIN_FRAME_CACHE_MB);
        assert!(DEFAULT_FRAME_CACHE_MB <= MAX_FRAME_CACHE_MB);
    }

    #[test]
    fn frame_history_defaults_are_valid() {
        assert_eq!(DEFAULT_FRAME_HISTORY_MB, 128);
        assert!(DEFAULT_FRAME_HISTORY_MB >= MIN_FRAME_HISTORY_MB);
        assert!(DEFAULT_FRAME_HISTORY_MB <= MAX_FRAME_HISTORY_MB);
    }
}
