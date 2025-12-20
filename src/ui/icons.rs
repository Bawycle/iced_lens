// SPDX-License-Identifier: MPL-2.0
//! Centralized icon module for PNG icons.
//!
//! PNG format ensures consistent cross-platform rendering (no SVG interpretation
//! differences on Windows). Icons are embedded at compile time via `include_bytes!`
//! and handles are cached using `OnceLock` for optimal performance.
//!
//! # Module Structure
//!
//! - **`icons::*`** - Dark icons (black) from `assets/icons/png/dark/` for light theme
//! - **`icons::light::*`** - Light icons (white) from `assets/icons/png/light/` for dark theme
//! - **`icons::overlay::*`** - Light icons for HUD/overlays on dark backgrounds
//!
//! # Usage
//!
//! ```ignore
//! use crate::ui::icons;
//!
//! // For light theme (default)
//! let play_button = button(icons::play());
//!
//! // For dark theme
//! let play_button = button(icons::light::chevron_double_right());
//! ```
//!
//! For theme-aware icons, use [`action_icons`](super::action_icons) which
//! automatically selects the correct variant based on theme.
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

define_icon!(
    play,
    "../../assets/icons/png/dark/play.png",
    "Play icon: triangle pointing right."
);
define_icon!(
    pause,
    "../../assets/icons/png/dark/pause.png",
    "Pause icon: two vertical bars."
);
define_icon!(
    volume,
    "../../assets/icons/png/dark/volume.png",
    "Volume icon: speaker with sound waves."
);
define_icon!(
    volume_mute,
    "../../assets/icons/png/dark/volume_mute.png",
    "Volume mute icon: speaker with X (crossed out)."
);
define_icon!(
    loop_icon,
    "../../assets/icons/png/dark/loop.png",
    "Loop icon: circular arrows indicating repeat."
);
define_icon!(
    camera,
    "../../assets/icons/png/dark/camera.png",
    "Camera icon: for frame capture/screenshot."
);
define_icon!(
    triangle_bar_right,
    "../../assets/icons/png/dark/triangle_bar_right.png",
    "Triangle with bar on right: play/skip next shape."
);
define_icon!(
    triangle_bar_left,
    "../../assets/icons/png/dark/triangle_bar_left.png",
    "Triangle with bar on left: skip previous shape."
);
define_icon!(
    ellipsis_horizontal,
    "../../assets/icons/png/dark/ellipsis_horizontal.png",
    "Horizontal ellipsis: three dots in a row."
);

// =============================================================================
// Status & Feedback Icons
// =============================================================================

define_icon!(
    warning,
    "../../assets/icons/png/dark/warning.png",
    "Warning icon: triangle with exclamation mark."
);
define_icon!(
    checkmark,
    "../../assets/icons/png/dark/checkmark.png",
    "Checkmark icon: check/tick mark for success."
);
define_icon!(
    cross,
    "../../assets/icons/png/dark/cross.png",
    "Cross icon: X mark shape."
);

// =============================================================================
// Zoom & View Icons
// =============================================================================

define_icon!(
    zoom_in,
    "../../assets/icons/png/dark/zoom_in.png",
    "Zoom in icon: magnifying glass with plus."
);
define_icon!(
    zoom_out,
    "../../assets/icons/png/dark/zoom_out.png",
    "Zoom out icon: magnifying glass with minus."
);
define_icon!(
    refresh,
    "../../assets/icons/png/dark/refresh.png",
    "Refresh icon: circular arrow (used for reset zoom)."
);
define_icon!(
    expand,
    "../../assets/icons/png/dark/expand.png",
    "Expand icon: arrows pointing outward (fit-to-window off)."
);
define_icon!(
    compress,
    "../../assets/icons/png/dark/compress.png",
    "Compress icon: arrows pointing inward (fit-to-window on)."
);
define_icon!(
    fullscreen,
    "../../assets/icons/png/dark/fullscreen.png",
    "Fullscreen icon: four corners expanding."
);

// =============================================================================
// Action Icons
// =============================================================================

define_icon!(
    trash,
    "../../assets/icons/png/dark/trash.png",
    "Trash icon: garbage bin (used for delete)."
);
define_icon!(
    pencil,
    "../../assets/icons/png/dark/pencil.png",
    "Pencil icon: for edit actions."
);

// =============================================================================
// Transform Icons (Editor)
// =============================================================================

define_icon!(
    rotate_left,
    "../../assets/icons/png/dark/rotate_left.png",
    "Rotate left icon: counter-clockwise arrow."
);
define_icon!(
    rotate_right,
    "../../assets/icons/png/dark/rotate_right.png",
    "Rotate right icon: clockwise arrow."
);
define_icon!(
    flip_horizontal,
    "../../assets/icons/png/dark/flip_horizontal.png",
    "Flip horizontal icon: mirror left-right."
);
define_icon!(
    flip_vertical,
    "../../assets/icons/png/dark/flip_vertical.png",
    "Flip vertical icon: mirror top-bottom."
);

// =============================================================================
// Navigation Icons
// =============================================================================

define_icon!(
    hamburger,
    "../../assets/icons/png/dark/hamburger.png",
    "Hamburger menu icon: three horizontal lines."
);
define_icon!(
    help,
    "../../assets/icons/png/dark/help.png",
    "Help icon: question mark in circle."
);
define_icon!(
    info,
    "../../assets/icons/png/dark/info.png",
    "Info icon: letter 'i' in circle."
);
define_icon!(
    chevron_double_right,
    "../../assets/icons/png/dark/chevron_double_right.png",
    "Double chevron right icon: two chevrons pointing right (>>), used for sidebar collapse."
);
define_icon!(
    chevron_double_left,
    "../../assets/icons/png/dark/chevron_double_left.png",
    "Double chevron left icon: two chevrons pointing left (<<), used for sidebar expand."
);
define_icon!(
    chevron_right,
    "../../assets/icons/png/dark/chevron_right.png",
    "Single chevron right icon: chevron pointing right (>), used for navigation next."
);
define_icon!(
    chevron_left,
    "../../assets/icons/png/dark/chevron_left.png",
    "Single chevron left icon: chevron pointing left (<), used for navigation previous."
);

// =============================================================================
// Settings Section Icons
// =============================================================================

define_icon!(
    globe,
    "../../assets/icons/png/dark/globe.png",
    "Globe icon: world/international (for general settings)."
);
define_icon!(
    image,
    "../../assets/icons/png/dark/image.png",
    "Image icon: picture frame (for display settings)."
);
define_icon!(
    cog,
    "../../assets/icons/png/dark/cog.png",
    "Cog icon: gear/settings."
);

// =============================================================================
// HUD Indicator Icons
// =============================================================================

define_icon!(
    crosshair,
    "../../assets/icons/png/dark/crosshair.png",
    "Crosshair icon: position indicator."
);
define_icon!(
    magnifier,
    "../../assets/icons/png/dark/magnifier.png",
    "Magnifier icon: simple magnifying glass (for HUD zoom indicator)."
);
define_icon!(
    video_camera,
    "../../assets/icons/png/dark/video_camera.png",
    "Video camera icon: camcorder without audio."
);
define_icon!(
    video_camera_audio,
    "../../assets/icons/png/dark/video_camera_audio.png",
    "Video camera with audio icon: camcorder with sound wave."
);

// =============================================================================
// Light Icons (White variants for dark theme UI)
// =============================================================================

/// Light icon variants for use with dark theme (white icons on dark backgrounds).
pub mod light {
    use super::*;

    define_icon!(
        chevron_double_right,
        "../../assets/icons/png/light/chevron_double_right.png",
        "Double chevron right icon (white): for dark theme UI."
    );
    define_icon!(
        chevron_double_left,
        "../../assets/icons/png/light/chevron_double_left.png",
        "Double chevron left icon (white): for dark theme UI."
    );
    define_icon!(
        chevron_right,
        "../../assets/icons/png/light/chevron_right.png",
        "Single chevron right icon (white): for dark theme UI."
    );
    define_icon!(
        chevron_left,
        "../../assets/icons/png/light/chevron_left.png",
        "Single chevron left icon (white): for dark theme UI."
    );
    define_icon!(
        pencil,
        "../../assets/icons/png/light/pencil.png",
        "Pencil icon (white): for dark theme UI."
    );
}

// =============================================================================
// Overlay Icons (Light variants for dark backgrounds)
// =============================================================================

/// Light icon variants for use on dark backgrounds (overlays, HUD).
pub mod overlay {
    use super::*;

    define_icon!(
        play,
        "../../assets/icons/png/light/play.png",
        "Play icon (white): for dark overlay backgrounds."
    );
    define_icon!(
        pause,
        "../../assets/icons/png/light/pause.png",
        "Pause icon (white): for dark overlay backgrounds."
    );
    define_icon!(
        loop_icon,
        "../../assets/icons/png/light/loop.png",
        "Loop icon (white): for dark overlay backgrounds."
    );
    define_icon!(
        warning,
        "../../assets/icons/png/light/warning.png",
        "Warning icon (white): for dark overlay backgrounds."
    );
    define_icon!(
        crosshair,
        "../../assets/icons/png/light/crosshair.png",
        "Crosshair icon (white): for HUD on dark backgrounds."
    );
    define_icon!(
        magnifier,
        "../../assets/icons/png/light/magnifier.png",
        "Magnifier icon (white): for HUD on dark backgrounds."
    );
    define_icon!(
        video_camera,
        "../../assets/icons/png/light/video_camera.png",
        "Video camera icon (white): for HUD on dark backgrounds."
    );
    define_icon!(
        video_camera_audio,
        "../../assets/icons/png/light/video_camera_audio.png",
        "Video camera with audio icon (white): for HUD on dark backgrounds."
    );
    define_icon!(
        chevron_right,
        "../../assets/icons/png/light/chevron_right.png",
        "Single chevron right icon (white): for navigation next on dark backgrounds."
    );
    define_icon!(
        chevron_left,
        "../../assets/icons/png/light/chevron_left.png",
        "Single chevron left icon (white): for navigation previous on dark backgrounds."
    );
}

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
        let _ = pencil();
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
        let _ = chevron_double_right();
        let _ = chevron_double_left();
        let _ = chevron_right();
        let _ = chevron_left();
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

    #[test]
    fn light_icons_load_successfully() {
        let _ = light::chevron_double_right();
        let _ = light::chevron_double_left();
        let _ = light::chevron_right();
        let _ = light::chevron_left();
        let _ = light::pencil();
    }

    #[test]
    fn overlay_icons_load_successfully() {
        let _ = overlay::play();
        let _ = overlay::pause();
        let _ = overlay::loop_icon();
        let _ = overlay::warning();
        let _ = overlay::crosshair();
        let _ = overlay::magnifier();
        let _ = overlay::video_camera();
        let _ = overlay::video_camera_audio();
        let _ = overlay::chevron_right();
        let _ = overlay::chevron_left();
    }
}
