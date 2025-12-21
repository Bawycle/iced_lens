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
//! - **Playback Speed**: Video playback speed control

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

/// Default video playback volume (0.0 to 1.5, where 1.0 = 100%).
pub const DEFAULT_VOLUME: f32 = 0.8;

/// Minimum volume level.
pub const MIN_VOLUME: f32 = 0.0;

/// Maximum volume level (1.5 = 150% amplification).
pub const MAX_VOLUME: f32 = 1.5;

/// Volume adjustment step per key press (5%).
pub const VOLUME_STEP: f32 = 0.05;

/// Target loudness for audio normalization (LUFS).
/// EBU R128 standard uses -23 LUFS, but -16 LUFS is common for streaming.
pub const DEFAULT_NORMALIZATION_TARGET_LUFS: f32 = -16.0;

// ==========================================================================
// Video Seek Defaults
// ==========================================================================

/// Default keyboard seek step in seconds (arrow keys).
pub const DEFAULT_KEYBOARD_SEEK_STEP_SECS: f64 = 2.0;

/// Minimum keyboard seek step in seconds.
pub const MIN_KEYBOARD_SEEK_STEP_SECS: f64 = 0.5;

/// Maximum keyboard seek step in seconds.
pub const MAX_KEYBOARD_SEEK_STEP_SECS: f64 = 30.0;

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
// AI/Deblur Defaults
// ==========================================================================

/// Default URL for downloading the NAFNet ONNX deblurring model.
pub const DEFAULT_DEBLUR_MODEL_URL: &str =
    "https://huggingface.co/opencv/deblurring_nafnet/resolve/main/deblurring_nafnet_2025may.onnx";

// ==========================================================================
// Playback Speed Defaults
// ==========================================================================

/// Default playback speed (1.0 = normal speed).
pub const DEFAULT_PLAYBACK_SPEED: f64 = 1.0;

/// Minimum playback speed (0.1x = ten times slower).
pub const MIN_PLAYBACK_SPEED: f64 = 0.1;

/// Maximum playback speed (8x = eight times faster).
pub const MAX_PLAYBACK_SPEED: f64 = 8.0;

/// Playback speed presets for the speed control buttons.
/// Ordered from slowest to fastest. Users cycle through these with J and L keys.
pub const PLAYBACK_SPEED_PRESETS: &[f64] = &[
    0.1, 0.15, 0.2, 0.25, 0.33, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 4.0, 8.0,
];

/// Speed threshold above which audio is automatically muted.
/// At speeds > 2x, audio becomes distorted and unintelligible.
pub const PLAYBACK_SPEED_AUTO_MUTE_THRESHOLD: f64 = 2.0;

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

    // Keyboard seek step validation
    assert!(MIN_KEYBOARD_SEEK_STEP_SECS > 0.0);
    assert!(MAX_KEYBOARD_SEEK_STEP_SECS >= MIN_KEYBOARD_SEEK_STEP_SECS);
    assert!(DEFAULT_KEYBOARD_SEEK_STEP_SECS >= MIN_KEYBOARD_SEEK_STEP_SECS);
    assert!(DEFAULT_KEYBOARD_SEEK_STEP_SECS <= MAX_KEYBOARD_SEEK_STEP_SECS);

    // Playback speed validation
    assert!(MIN_PLAYBACK_SPEED > 0.0);
    assert!(MAX_PLAYBACK_SPEED > MIN_PLAYBACK_SPEED);
    assert!(DEFAULT_PLAYBACK_SPEED >= MIN_PLAYBACK_SPEED);
    assert!(DEFAULT_PLAYBACK_SPEED <= MAX_PLAYBACK_SPEED);
    assert!(PLAYBACK_SPEED_AUTO_MUTE_THRESHOLD > 1.0);
    assert!(PLAYBACK_SPEED_AUTO_MUTE_THRESHOLD <= MAX_PLAYBACK_SPEED);

    // Ensure presets array is not empty
    assert!(!PLAYBACK_SPEED_PRESETS.is_empty());

    // Validate presets are in ascending order and within bounds
    let mut i = 0;
    while i < PLAYBACK_SPEED_PRESETS.len() {
        // Each preset must be within valid range
        assert!(PLAYBACK_SPEED_PRESETS[i] >= MIN_PLAYBACK_SPEED);
        assert!(PLAYBACK_SPEED_PRESETS[i] <= MAX_PLAYBACK_SPEED);

        // Presets must be in ascending order (for cycling to work correctly)
        if i > 0 {
            assert!(PLAYBACK_SPEED_PRESETS[i] > PLAYBACK_SPEED_PRESETS[i - 1]);
        }
        i += 1;
    }

    // Ensure default speed (1.0) is in the presets
    let mut found_default = false;
    let mut j = 0;
    while j < PLAYBACK_SPEED_PRESETS.len() {
        // Use integer comparison to avoid floating point issues
        // 1.0 * 100 = 100, comparing integers
        if (PLAYBACK_SPEED_PRESETS[j] * 100.0) as i32 == (DEFAULT_PLAYBACK_SPEED * 100.0) as i32 {
            found_default = true;
        }
        j += 1;
    }
    assert!(found_default);
};
