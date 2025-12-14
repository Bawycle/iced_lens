// SPDX-License-Identifier: MPL-2.0
//! Semantic action icons mapping.
//!
//! This module provides a semantic layer over [`icons`](super::icons), mapping
//! user actions to their visual icon representations. This separation allows
//! changing an action's icon in one place without modifying all usage sites.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │         Component / Help Screen         │  ← Uses semantic names
//! ├─────────────────────────────────────────┤
//! │         action_icons (this module)      │  ← Semantic → Visual mapping
//! ├─────────────────────────────────────────┤
//! │         icons (visual primitives)       │  ← Raw SVG assets
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use crate::ui::action_icons;
//!
//! // In UI components
//! let btn = button(action_icons::video::capture_frame());
//!
//! // In help screen
//! let icon = action_icons::editor::rotate_left();
//! ```
//!
//! # Naming Convention
//!
//! Functions are named by **what action they represent**, not what they look like.
//! The underlying visual icon can change without affecting call sites.

use super::icons;
use iced::widget::svg::Svg;

// =============================================================================
// Video Playback Actions
// =============================================================================

/// Icons for video playback controls.
pub mod video {
    use super::*;

    /// Play video.
    pub fn play<'a>() -> Svg<'a> {
        icons::play()
    }

    /// Pause video.
    pub fn pause<'a>() -> Svg<'a> {
        icons::pause()
    }

    /// Step forward one frame.
    pub fn step_forward<'a>() -> Svg<'a> {
        icons::triangle_bar_right()
    }

    /// Step backward one frame.
    pub fn step_backward<'a>() -> Svg<'a> {
        icons::triangle_bar_left()
    }

    /// Capture current frame as image.
    pub fn capture_frame<'a>() -> Svg<'a> {
        icons::camera()
    }

    /// Toggle loop playback.
    pub fn toggle_loop<'a>() -> Svg<'a> {
        icons::loop_icon()
    }

    /// Volume control (unmuted state).
    pub fn volume<'a>() -> Svg<'a> {
        icons::volume()
    }

    /// Volume muted state.
    pub fn volume_muted<'a>() -> Svg<'a> {
        icons::volume_mute()
    }

    /// More options / overflow menu.
    pub fn more_options<'a>() -> Svg<'a> {
        icons::ellipsis_horizontal()
    }
}

// =============================================================================
// Image Editor Actions
// =============================================================================

/// Icons for image editor tools.
pub mod editor {
    use super::*;

    /// Rotate image 90° counter-clockwise.
    pub fn rotate_left<'a>() -> Svg<'a> {
        icons::rotate_left()
    }

    /// Rotate image 90° clockwise.
    pub fn rotate_right<'a>() -> Svg<'a> {
        icons::rotate_right()
    }

    /// Flip image horizontally.
    pub fn flip_horizontal<'a>() -> Svg<'a> {
        icons::flip_horizontal()
    }

    /// Flip image vertically.
    pub fn flip_vertical<'a>() -> Svg<'a> {
        icons::flip_vertical()
    }
}

// =============================================================================
// Viewer Actions
// =============================================================================

/// Icons for image/video viewer controls.
pub mod viewer {
    use super::*;

    /// Zoom in.
    pub fn zoom_in<'a>() -> Svg<'a> {
        icons::zoom_in()
    }

    /// Zoom out.
    pub fn zoom_out<'a>() -> Svg<'a> {
        icons::zoom_out()
    }

    /// Reset zoom to original size.
    pub fn zoom_reset<'a>() -> Svg<'a> {
        icons::refresh()
    }

    /// Fit image to window (enabled state).
    pub fn fit_to_window<'a>() -> Svg<'a> {
        icons::compress()
    }

    /// Fit image to window (disabled state / expand).
    pub fn expand<'a>() -> Svg<'a> {
        icons::expand()
    }

    /// Enter/exit fullscreen mode.
    pub fn fullscreen<'a>() -> Svg<'a> {
        icons::fullscreen()
    }

    /// Delete current media file.
    pub fn delete<'a>() -> Svg<'a> {
        icons::trash()
    }
}

// =============================================================================
// Navigation Actions
// =============================================================================

/// Icons for app navigation.
pub mod navigation {
    use super::*;

    /// Open hamburger menu.
    pub fn menu<'a>() -> Svg<'a> {
        icons::hamburger()
    }

    /// Open settings.
    pub fn settings<'a>() -> Svg<'a> {
        icons::cog()
    }

    /// Open help.
    pub fn help<'a>() -> Svg<'a> {
        icons::help()
    }

    /// Open about screen.
    pub fn about<'a>() -> Svg<'a> {
        icons::info()
    }

    /// Close / dismiss.
    pub fn close<'a>() -> Svg<'a> {
        icons::cross()
    }
}

// =============================================================================
// Notification Severity Icons
// =============================================================================

/// Icons for notification severities.
pub mod notification {
    use super::*;

    /// Success notification.
    pub fn success<'a>() -> Svg<'a> {
        icons::checkmark()
    }

    /// Warning notification.
    pub fn warning<'a>() -> Svg<'a> {
        icons::warning()
    }

    /// Error notification.
    pub fn error<'a>() -> Svg<'a> {
        icons::warning()
    }

    /// Info notification.
    pub fn info<'a>() -> Svg<'a> {
        icons::info()
    }
}

// =============================================================================
// Help Section Icons
// =============================================================================

/// Icons for help screen sections.
pub mod sections {
    use super::*;

    /// Image/video viewer section.
    pub fn viewer<'a>() -> Svg<'a> {
        icons::image()
    }

    /// Video playback section.
    pub fn video<'a>() -> Svg<'a> {
        icons::video_camera()
    }

    /// Frame capture section.
    pub fn capture<'a>() -> Svg<'a> {
        icons::camera()
    }

    /// Image editor section.
    pub fn editor<'a>() -> Svg<'a> {
        icons::rotate_right()
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Re-export of [`icons::sized`] for convenience.
pub use icons::sized;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn video_icons_load() {
        let _ = video::play();
        let _ = video::pause();
        let _ = video::step_forward();
        let _ = video::step_backward();
        let _ = video::capture_frame();
        let _ = video::toggle_loop();
        let _ = video::volume();
        let _ = video::volume_muted();
        let _ = video::more_options();
    }

    #[test]
    fn editor_icons_load() {
        let _ = editor::rotate_left();
        let _ = editor::rotate_right();
        let _ = editor::flip_horizontal();
        let _ = editor::flip_vertical();
    }

    #[test]
    fn viewer_icons_load() {
        let _ = viewer::zoom_in();
        let _ = viewer::zoom_out();
        let _ = viewer::zoom_reset();
        let _ = viewer::fit_to_window();
        let _ = viewer::expand();
        let _ = viewer::fullscreen();
        let _ = viewer::delete();
    }

    #[test]
    fn navigation_icons_load() {
        let _ = navigation::menu();
        let _ = navigation::settings();
        let _ = navigation::help();
        let _ = navigation::about();
        let _ = navigation::close();
    }

    #[test]
    fn notification_icons_load() {
        let _ = notification::success();
        let _ = notification::warning();
        let _ = notification::error();
        let _ = notification::info();
    }

    #[test]
    fn section_icons_load() {
        let _ = sections::viewer();
        let _ = sections::video();
        let _ = sections::capture();
        let _ = sections::editor();
    }
}
