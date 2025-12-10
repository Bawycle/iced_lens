// SPDX-License-Identifier: MPL-2.0
//! Centralized icon module for SVG icons.
//!
//! All UI icons are loaded from `assets/icons/` and exposed as functions
//! returning [`iced::widget::Svg`] widgets. This ensures consistent icon
//! management across the application.
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

use iced::widget::svg::{Handle, Svg};
use iced::Length;

// =============================================================================
// Video Playback Icons
// =============================================================================

/// Play icon: triangle pointing right.
pub fn play<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/play.svg");
    Svg::new(Handle::from_memory(DATA))
}

/// Pause icon: two vertical bars.
pub fn pause<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/pause.svg");
    Svg::new(Handle::from_memory(DATA))
}

/// Volume icon: speaker with sound waves.
pub fn volume<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/volume.svg");
    Svg::new(Handle::from_memory(DATA))
}

/// Volume mute icon: speaker with X (crossed out).
pub fn volume_mute<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/volume_mute.svg");
    Svg::new(Handle::from_memory(DATA))
}

/// Loop icon: circular arrows indicating repeat.
pub fn loop_icon<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/loop.svg");
    Svg::new(Handle::from_memory(DATA))
}

// =============================================================================
// Status & Feedback Icons
// =============================================================================

/// Warning icon: triangle with exclamation mark.
pub fn warning<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/warning.svg");
    Svg::new(Handle::from_memory(DATA))
}

// =============================================================================
// Zoom & View Icons
// =============================================================================

/// Zoom in icon: magnifying glass with plus.
pub fn zoom_in<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/zoom_in.svg");
    Svg::new(Handle::from_memory(DATA))
}

/// Zoom out icon: magnifying glass with minus.
pub fn zoom_out<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/zoom_out.svg");
    Svg::new(Handle::from_memory(DATA))
}

/// Refresh icon: circular arrow (used for reset zoom).
pub fn refresh<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/refresh.svg");
    Svg::new(Handle::from_memory(DATA))
}

/// Expand icon: arrows pointing outward (fit-to-window off).
pub fn expand<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/expand.svg");
    Svg::new(Handle::from_memory(DATA))
}

/// Compress icon: arrows pointing inward (fit-to-window on).
pub fn compress<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/compress.svg");
    Svg::new(Handle::from_memory(DATA))
}

/// Fullscreen icon: four corners expanding.
pub fn fullscreen<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/fullscreen.svg");
    Svg::new(Handle::from_memory(DATA))
}

// =============================================================================
// Action Icons
// =============================================================================

/// Trash icon: garbage bin (used for delete).
pub fn trash<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/trash.svg");
    Svg::new(Handle::from_memory(DATA))
}

// =============================================================================
// Transform Icons (Editor)
// =============================================================================

/// Rotate left icon: counter-clockwise arrow.
pub fn rotate_left<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/rotate_left.svg");
    Svg::new(Handle::from_memory(DATA))
}

/// Rotate right icon: clockwise arrow.
pub fn rotate_right<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/rotate_right.svg");
    Svg::new(Handle::from_memory(DATA))
}

/// Flip horizontal icon: mirror left-right.
pub fn flip_horizontal<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/flip_horizontal.svg");
    Svg::new(Handle::from_memory(DATA))
}

/// Flip vertical icon: mirror top-bottom.
pub fn flip_vertical<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/flip_vertical.svg");
    Svg::new(Handle::from_memory(DATA))
}

// =============================================================================
// HUD Indicator Icons
// =============================================================================

/// Crosshair icon: position indicator.
pub fn crosshair<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/crosshair.svg");
    Svg::new(Handle::from_memory(DATA))
}

/// Magnifier icon: simple magnifying glass (for HUD zoom indicator).
pub fn magnifier<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/magnifier.svg");
    Svg::new(Handle::from_memory(DATA))
}

/// Video camera icon: camcorder without audio.
pub fn video_camera<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/video_camera.svg");
    Svg::new(Handle::from_memory(DATA))
}

/// Video camera with audio icon: camcorder with sound wave.
pub fn video_camera_audio<'a>() -> Svg<'a> {
    static DATA: &[u8] = include_bytes!("../../assets/icons/video_camera_audio.svg");
    Svg::new(Handle::from_memory(DATA))
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Creates an icon with specified dimensions.
///
/// This is a convenience wrapper for setting both width and height.
pub fn sized<'a>(icon: Svg<'a>, size: f32) -> Svg<'a> {
    icon.width(Length::Fixed(size)).height(Length::Fixed(size))
}

/// Creates an icon that fills its container.
pub fn fill<'a>(icon: Svg<'a>) -> Svg<'a> {
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
    }

    #[test]
    fn sized_helper_works() {
        let icon = sized(play(), 32.0);
        // Just verify it compiles and returns an Svg
        let _ = icon;
    }

    #[test]
    fn fill_helper_works() {
        let icon = fill(pause());
        let _ = icon;
    }
}
