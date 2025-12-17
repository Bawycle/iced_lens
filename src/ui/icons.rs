// SPDX-License-Identifier: MPL-2.0
//! Centralized icon module for PNG icons.
//!
//! All UI icons are loaded from `assets/icons/png/` and exposed as functions
//! returning [`iced::widget::Image`] widgets. PNG format ensures consistent
//! cross-platform rendering (no SVG interpretation differences on Windows).
//!
//! Icons are embedded in the binary at compile time via `include_bytes!`
//! and handles are cached using `OnceLock` for optimal performance.
//!
//! # Usage
//!
//! ```ignore
//! use crate::ui::icons;
//!
//! let play_button = button(icons::play())
//!     .on_press(Message::Play);
//! ```
//!
//! # Naming Convention
//!
//! Icons use generic visual names describing the icon's appearance,
//! not the action context (e.g., `trash` not `delete_image`).

use iced::widget::image::{Handle, Image};
use iced::Length;
use std::sync::OnceLock;

// =============================================================================
// Macro for icon definition with cached handle
// =============================================================================

/// Macro to define an icon function with a cached handle.
/// The handle is created once on first access and reused thereafter.
macro_rules! define_icon {
    ($name:ident, $path:literal, $doc:literal) => {
        #[doc = $doc]
        pub fn $name() -> Image<Handle> {
            static HANDLE: OnceLock<Handle> = OnceLock::new();
            static DATA: &[u8] = include_bytes!($path);
            let handle = HANDLE.get_or_init(|| Handle::from_bytes(DATA));
            Image::new(handle.clone())
        }
    };
}

// =============================================================================
// Video Playback Icons
// =============================================================================

define_icon!(play, "../../assets/icons/png/play.png", "Play icon: triangle pointing right.");
define_icon!(pause, "../../assets/icons/png/pause.png", "Pause icon: two vertical bars.");
define_icon!(volume, "../../assets/icons/png/volume.png", "Volume icon: speaker with sound waves.");
define_icon!(volume_mute, "../../assets/icons/png/volume_mute.png", "Volume mute icon: speaker with X (crossed out).");
define_icon!(loop_icon, "../../assets/icons/png/loop.png", "Loop icon: circular arrows indicating repeat.");
define_icon!(camera, "../../assets/icons/png/camera.png", "Camera icon: for frame capture/screenshot.");
define_icon!(triangle_bar_right, "../../assets/icons/png/triangle_bar_right.png", "Triangle with bar on right: play/skip next shape.");
define_icon!(triangle_bar_left, "../../assets/icons/png/triangle_bar_left.png", "Triangle with bar on left: skip previous shape.");
define_icon!(ellipsis_horizontal, "../../assets/icons/png/ellipsis_horizontal.png", "Horizontal ellipsis: three dots in a row.");

// =============================================================================
// Status & Feedback Icons
// =============================================================================

define_icon!(warning, "../../assets/icons/png/warning.png", "Warning icon: triangle with exclamation mark.");
define_icon!(checkmark, "../../assets/icons/png/checkmark.png", "Checkmark icon: check/tick mark for success.");
define_icon!(cross, "../../assets/icons/png/cross.png", "Cross icon: X mark shape.");

// =============================================================================
// Zoom & View Icons
// =============================================================================

define_icon!(zoom_in, "../../assets/icons/png/zoom_in.png", "Zoom in icon: magnifying glass with plus.");
define_icon!(zoom_out, "../../assets/icons/png/zoom_out.png", "Zoom out icon: magnifying glass with minus.");
define_icon!(refresh, "../../assets/icons/png/refresh.png", "Refresh icon: circular arrow (used for reset zoom).");
define_icon!(expand, "../../assets/icons/png/expand.png", "Expand icon: arrows pointing outward (fit-to-window off).");
define_icon!(compress, "../../assets/icons/png/compress.png", "Compress icon: arrows pointing inward (fit-to-window on).");
define_icon!(fullscreen, "../../assets/icons/png/fullscreen.png", "Fullscreen icon: four corners expanding.");

// =============================================================================
// Action Icons
// =============================================================================

define_icon!(trash, "../../assets/icons/png/trash.png", "Trash icon: garbage bin (used for delete).");

// =============================================================================
// Transform Icons (Editor)
// =============================================================================

define_icon!(rotate_left, "../../assets/icons/png/rotate_left.png", "Rotate left icon: counter-clockwise arrow.");
define_icon!(rotate_right, "../../assets/icons/png/rotate_right.png", "Rotate right icon: clockwise arrow.");
define_icon!(flip_horizontal, "../../assets/icons/png/flip_horizontal.png", "Flip horizontal icon: mirror left-right.");
define_icon!(flip_vertical, "../../assets/icons/png/flip_vertical.png", "Flip vertical icon: mirror top-bottom.");

// =============================================================================
// Navigation Icons
// =============================================================================

define_icon!(hamburger, "../../assets/icons/png/hamburger.png", "Hamburger menu icon: three horizontal lines.");
define_icon!(help, "../../assets/icons/png/help.png", "Help icon: question mark in circle.");
define_icon!(info, "../../assets/icons/png/info.png", "Info icon: letter 'i' in circle.");

// =============================================================================
// Settings Section Icons
// =============================================================================

define_icon!(globe, "../../assets/icons/png/globe.png", "Globe icon: world/international (for general settings).");
define_icon!(image, "../../assets/icons/png/image.png", "Image icon: picture frame (for display settings).");
define_icon!(cog, "../../assets/icons/png/cog.png", "Cog icon: gear/settings.");

// =============================================================================
// HUD Indicator Icons
// =============================================================================

define_icon!(crosshair, "../../assets/icons/png/crosshair.png", "Crosshair icon: position indicator.");
define_icon!(magnifier, "../../assets/icons/png/magnifier.png", "Magnifier icon: simple magnifying glass (for HUD zoom indicator).");
define_icon!(video_camera, "../../assets/icons/png/video_camera.png", "Video camera icon: camcorder without audio.");
define_icon!(video_camera_audio, "../../assets/icons/png/video_camera_audio.png", "Video camera with audio icon: camcorder with sound wave.");

// =============================================================================
// Helper Functions
// =============================================================================

/// Creates an icon with specified dimensions.
///
/// This is a convenience wrapper for setting both width and height.
pub fn sized(icon: Image<Handle>, size: f32) -> Image<Handle> {
    icon.width(Length::Fixed(size)).height(Length::Fixed(size))
}

/// Creates an icon that fills its container.
pub fn fill(icon: Image<Handle>) -> Image<Handle> {
    icon.width(Length::Fill).height(Length::Fill)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_icons_load_successfully() {
        // These calls verify that all include_bytes! paths are valid
        let _ = play();
        let _ = pause();
        let _ = volume();
        let _ = volume_mute();
        let _ = loop_icon();
        let _ = zoom_in();
        let _ = zoom_out();
        let _ = refresh();
        let _ = expand();
        let _ = compress();
        let _ = fullscreen();
        let _ = trash();
        let _ = rotate_left();
        let _ = rotate_right();
        let _ = flip_horizontal();
        let _ = flip_vertical();
        let _ = crosshair();
        let _ = magnifier();
        let _ = video_camera();
        let _ = video_camera_audio();
        let _ = globe();
        let _ = image();
        let _ = cog();
        let _ = camera();
        let _ = triangle_bar_right();
        let _ = triangle_bar_left();
        let _ = ellipsis_horizontal();
        let _ = hamburger();
        let _ = help();
        let _ = info();
        let _ = warning();
        let _ = checkmark();
        let _ = cross();
    }

    #[test]
    fn sized_helper_works() {
        let icon = sized(play(), 32.0);
        // Just verify it compiles and returns an Image
        let _ = icon;
    }

    #[test]
    fn fill_helper_works() {
        let icon = fill(pause());
        let _ = icon;
    }
}
