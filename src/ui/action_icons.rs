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
use iced::widget::image::{Handle, Image};

// =============================================================================
// Video Playback Actions
// =============================================================================

/// Icons for video playback controls.
pub mod video {
    use super::*;

    /// Play video.
    pub fn play() -> Image<Handle> {
        icons::play()
    }

    /// Pause video.
    pub fn pause() -> Image<Handle> {
        icons::pause()
    }

    /// Step forward one frame.
    pub fn step_forward() -> Image<Handle> {
        icons::triangle_bar_right()
    }

    /// Step backward one frame.
    pub fn step_backward() -> Image<Handle> {
        icons::triangle_bar_left()
    }

    /// Capture current frame as image.
    pub fn capture_frame() -> Image<Handle> {
        icons::camera()
    }

    /// Toggle loop playback.
    pub fn toggle_loop() -> Image<Handle> {
        icons::loop_icon()
    }

    /// Volume control (unmuted state).
    pub fn volume() -> Image<Handle> {
        icons::volume()
    }

    /// Volume muted state.
    pub fn volume_muted() -> Image<Handle> {
        icons::volume_mute()
    }

    /// More options / overflow menu.
    pub fn more_options() -> Image<Handle> {
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
    pub fn rotate_left() -> Image<Handle> {
        icons::rotate_left()
    }

    /// Rotate image 90° clockwise.
    pub fn rotate_right() -> Image<Handle> {
        icons::rotate_right()
    }

    /// Flip image horizontally.
    pub fn flip_horizontal() -> Image<Handle> {
        icons::flip_horizontal()
    }

    /// Flip image vertically.
    pub fn flip_vertical() -> Image<Handle> {
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
    pub fn zoom_in() -> Image<Handle> {
        icons::zoom_in()
    }

    /// Zoom out.
    pub fn zoom_out() -> Image<Handle> {
        icons::zoom_out()
    }

    /// Reset zoom to original size.
    pub fn zoom_reset() -> Image<Handle> {
        icons::refresh()
    }

    /// Fit image to window (enabled state).
    pub fn fit_to_window() -> Image<Handle> {
        icons::compress()
    }

    /// Fit image to window (disabled state / expand).
    pub fn expand() -> Image<Handle> {
        icons::expand()
    }

    /// Enter/exit fullscreen mode.
    pub fn fullscreen() -> Image<Handle> {
        icons::fullscreen()
    }

    /// Delete current media file.
    pub fn delete() -> Image<Handle> {
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
    pub fn menu() -> Image<Handle> {
        icons::hamburger()
    }

    /// Open settings.
    pub fn settings() -> Image<Handle> {
        icons::cog()
    }

    /// Open help.
    pub fn help() -> Image<Handle> {
        icons::help()
    }

    /// Open about screen.
    pub fn about() -> Image<Handle> {
        icons::info()
    }

    /// Close / dismiss.
    pub fn close() -> Image<Handle> {
        icons::cross()
    }

    /// Edit action (e.g., edit metadata).
    /// Returns dark icon for light theme, light icon for dark theme.
    pub fn edit(is_dark_theme: bool) -> Image<Handle> {
        if is_dark_theme {
            icons::light::pencil()
        } else {
            icons::pencil()
        }
    }

    /// Collapse a left-side panel (chevron points left).
    /// Returns dark icon for light theme, light icon for dark theme.
    pub fn collapse_left_panel(is_dark_theme: bool) -> Image<Handle> {
        if is_dark_theme {
            icons::light::chevron_double_left()
        } else {
            icons::chevron_double_left()
        }
    }

    /// Expand a left-side panel (chevron points right).
    /// Returns dark icon for light theme, light icon for dark theme.
    pub fn expand_left_panel(is_dark_theme: bool) -> Image<Handle> {
        if is_dark_theme {
            icons::light::chevron_double_right()
        } else {
            icons::chevron_double_right()
        }
    }

    /// Collapse a right-side panel (chevron points right).
    /// Returns dark icon for light theme, light icon for dark theme.
    pub fn collapse_right_panel(is_dark_theme: bool) -> Image<Handle> {
        if is_dark_theme {
            icons::light::chevron_double_right()
        } else {
            icons::chevron_double_right()
        }
    }

    /// Expand a right-side panel (chevron points left).
    /// Returns dark icon for light theme, light icon for dark theme.
    pub fn expand_right_panel(is_dark_theme: bool) -> Image<Handle> {
        if is_dark_theme {
            icons::light::chevron_double_left()
        } else {
            icons::chevron_double_left()
        }
    }
}

// =============================================================================
// Notification Severity Icons
// =============================================================================

/// Icons for notification severities.
pub mod notification {
    use super::*;

    /// Success notification.
    pub fn success() -> Image<Handle> {
        icons::checkmark()
    }

    /// Warning notification.
    pub fn warning() -> Image<Handle> {
        icons::warning()
    }

    /// Error notification.
    pub fn error() -> Image<Handle> {
        icons::warning()
    }

    /// Info notification.
    pub fn info() -> Image<Handle> {
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
    pub fn viewer() -> Image<Handle> {
        icons::image()
    }

    /// Video playback section.
    pub fn video() -> Image<Handle> {
        icons::video_camera()
    }

    /// Frame capture section.
    pub fn capture() -> Image<Handle> {
        icons::camera()
    }

    /// Image editor section.
    pub fn editor() -> Image<Handle> {
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
        // Test both theme variants
        let _ = navigation::collapse_left_panel(false);
        let _ = navigation::collapse_left_panel(true);
        let _ = navigation::expand_left_panel(false);
        let _ = navigation::expand_left_panel(true);
        let _ = navigation::collapse_right_panel(false);
        let _ = navigation::collapse_right_panel(true);
        let _ = navigation::expand_right_panel(false);
        let _ = navigation::expand_right_panel(true);
        let _ = navigation::edit(false);
        let _ = navigation::edit(true);
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
