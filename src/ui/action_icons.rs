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
//! │         icons (visual primitives)       │  ← Raw PNG assets
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Module Structure
//!
//! Each domain module provides dark icons by default (for help screens, sidebars).
//! Toolbar sub-modules provide light icons for button backgrounds:
//!
//! - **`video::*`** - Dark icons for video playback actions
//! - **`video::toolbar::*`** - Light icons for video toolbar buttons
//! - **`viewer::*`** - Dark icons for viewer actions
//! - **`viewer::toolbar::*`** - Light icons for viewer toolbar buttons
//! - **`editor::*`** - Dark icons for image editor actions
//! - **`navigation::*`** - Navigation icons (light for buttons, theme-aware for panels)
//! - **`notification::*`** - Icons for toast notifications
//! - **`sections::*`** - Icons for help screen section headers
//!
//! # Usage
//!
//! ```ignore
//! use crate::ui::action_icons;
//!
//! // In toolbar buttons (light icons for dark button backgrounds)
//! let btn = button(action_icons::video::toolbar::play());
//! let btn = button(action_icons::viewer::toolbar::zoom_in());
//!
//! // In help screen or sidebars (dark icons)
//! let icon = action_icons::editor::rotate_left();
//! let icon = action_icons::video::capture_frame();
//! ```
//!
//! # Naming Convention
//!
//! Functions are named by **what action they represent**, not what they look like.
//! The underlying visual icon can change without affecting call sites.

use super::icons;

// =============================================================================
// Video Playback Actions
// =============================================================================

/// Icons for video playback controls.
pub mod video {
    use super::icons;
    use iced::widget::image::{Handle, Image};

    /// Play video.
    #[must_use]
    pub fn play() -> Image<Handle> {
        icons::play()
    }

    /// Pause video.
    #[must_use]
    pub fn pause() -> Image<Handle> {
        icons::pause()
    }

    /// Step forward one frame.
    #[must_use]
    pub fn step_forward() -> Image<Handle> {
        icons::triangle_bar_right()
    }

    /// Step backward one frame.
    #[must_use]
    pub fn step_backward() -> Image<Handle> {
        icons::triangle_bar_left()
    }

    /// Capture current frame as image.
    #[must_use]
    pub fn capture_frame() -> Image<Handle> {
        icons::camera()
    }

    /// Toggle loop playback.
    #[must_use]
    pub fn toggle_loop() -> Image<Handle> {
        icons::loop_icon()
    }

    /// Volume control (unmuted state).
    #[must_use]
    pub fn volume() -> Image<Handle> {
        icons::volume()
    }

    /// Volume muted state.
    #[must_use]
    pub fn volume_muted() -> Image<Handle> {
        icons::volume_mute()
    }

    /// More options / overflow menu.
    #[must_use]
    pub fn more_options() -> Image<Handle> {
        icons::ellipsis_horizontal()
    }

    /// Decrease playback speed.
    #[must_use]
    pub fn speed_down() -> Image<Handle> {
        icons::triangle_minus()
    }

    /// Increase playback speed.
    #[must_use]
    pub fn speed_up() -> Image<Handle> {
        icons::triangle_plus()
    }

    /// Light icon variants for toolbar buttons.
    pub mod toolbar {
        use super::icons;
        use iced::widget::image::{Handle, Image};

        /// Play video (light icon for toolbar).
        #[must_use]
        pub fn play() -> Image<Handle> {
            icons::light::play()
        }

        /// Pause video (light icon for toolbar).
        #[must_use]
        pub fn pause() -> Image<Handle> {
            icons::light::pause()
        }

        /// Step forward one frame (light icon for toolbar).
        #[must_use]
        pub fn step_forward() -> Image<Handle> {
            icons::light::triangle_bar_right()
        }

        /// Step backward one frame (light icon for toolbar).
        #[must_use]
        pub fn step_backward() -> Image<Handle> {
            icons::light::triangle_bar_left()
        }

        /// Capture current frame (light icon for toolbar).
        #[must_use]
        pub fn capture_frame() -> Image<Handle> {
            icons::light::camera()
        }

        /// Toggle loop playback (light icon for toolbar).
        #[must_use]
        pub fn toggle_loop() -> Image<Handle> {
            icons::light::loop_icon()
        }

        /// Volume control (light icon for toolbar).
        #[must_use]
        pub fn volume() -> Image<Handle> {
            icons::light::volume()
        }

        /// Volume muted (light icon for toolbar).
        #[must_use]
        pub fn volume_muted() -> Image<Handle> {
            icons::light::volume_mute()
        }

        /// More options (light icon for toolbar).
        #[must_use]
        pub fn more_options() -> Image<Handle> {
            icons::light::ellipsis_horizontal()
        }

        /// Decrease playback speed (light icon for toolbar).
        #[must_use]
        pub fn speed_down() -> Image<Handle> {
            icons::light::triangle_minus()
        }

        /// Increase playback speed (light icon for toolbar).
        #[must_use]
        pub fn speed_up() -> Image<Handle> {
            icons::light::triangle_plus()
        }
    }
}

// =============================================================================
// Image Editor Actions
// =============================================================================

/// Icons for image editor tools.
pub mod editor {
    use super::icons;
    use iced::widget::image::{Handle, Image};

    /// Rotate image 90° counter-clockwise.
    #[must_use]
    pub fn rotate_left() -> Image<Handle> {
        icons::rotate_left()
    }

    /// Rotate image 90° clockwise.
    #[must_use]
    pub fn rotate_right() -> Image<Handle> {
        icons::rotate_right()
    }

    /// Flip image horizontally.
    #[must_use]
    pub fn flip_horizontal() -> Image<Handle> {
        icons::flip_horizontal()
    }

    /// Flip image vertically.
    #[must_use]
    pub fn flip_vertical() -> Image<Handle> {
        icons::flip_vertical()
    }

    /// Navigate to previous image in editor.
    #[must_use]
    pub fn navigate_previous() -> Image<Handle> {
        icons::chevron_left()
    }

    /// Navigate to next image in editor.
    #[must_use]
    pub fn navigate_next() -> Image<Handle> {
        icons::chevron_right()
    }
}

// =============================================================================
// Viewer Actions
// =============================================================================

/// Icons for image/video viewer controls.
pub mod viewer {
    use super::icons;
    use iced::widget::image::{Handle, Image};

    /// Zoom in.
    #[must_use]
    pub fn zoom_in() -> Image<Handle> {
        icons::zoom_in()
    }

    /// Zoom out.
    #[must_use]
    pub fn zoom_out() -> Image<Handle> {
        icons::zoom_out()
    }

    /// Reset zoom to original size.
    #[must_use]
    pub fn zoom_reset() -> Image<Handle> {
        icons::refresh()
    }

    /// Fit image to window (enabled state).
    #[must_use]
    pub fn fit_to_window() -> Image<Handle> {
        icons::compress()
    }

    /// Fit image to window (disabled state / expand).
    #[must_use]
    pub fn expand() -> Image<Handle> {
        icons::expand()
    }

    /// Enter/exit fullscreen mode.
    #[must_use]
    pub fn fullscreen() -> Image<Handle> {
        icons::fullscreen()
    }

    /// Delete current media file.
    #[must_use]
    pub fn delete() -> Image<Handle> {
        icons::trash()
    }

    /// Light icon variants for toolbar buttons.
    pub mod toolbar {
        use super::icons;
        use iced::widget::image::{Handle, Image};

        /// Zoom in (light icon for toolbar).
        #[must_use]
        pub fn zoom_in() -> Image<Handle> {
            icons::light::zoom_in()
        }

        /// Zoom out (light icon for toolbar).
        #[must_use]
        pub fn zoom_out() -> Image<Handle> {
            icons::light::zoom_out()
        }

        /// Reset zoom (light icon for toolbar).
        #[must_use]
        pub fn zoom_reset() -> Image<Handle> {
            icons::light::refresh()
        }

        /// Fit to window (light icon for toolbar).
        #[must_use]
        pub fn fit_to_window() -> Image<Handle> {
            icons::light::compress()
        }

        /// Expand (light icon for toolbar).
        #[must_use]
        pub fn expand() -> Image<Handle> {
            icons::light::expand()
        }

        /// Fullscreen (light icon for toolbar).
        #[must_use]
        pub fn fullscreen() -> Image<Handle> {
            icons::light::fullscreen()
        }

        /// Delete (light icon for toolbar).
        #[must_use]
        pub fn delete() -> Image<Handle> {
            icons::light::trash()
        }
    }
}

// =============================================================================
// Navigation Actions
// =============================================================================

/// Icons for app navigation.
pub mod navigation {
    use super::icons;
    use crate::config::BackgroundTheme;
    use iced::widget::image::{Handle, Image};

    /// Open hamburger menu (light icon for navbar button).
    #[must_use]
    pub fn menu() -> Image<Handle> {
        icons::light::hamburger()
    }

    /// Open settings.
    #[must_use]
    pub fn settings() -> Image<Handle> {
        icons::cog()
    }

    /// Open help.
    #[must_use]
    pub fn help() -> Image<Handle> {
        icons::help()
    }

    /// Open about screen.
    #[must_use]
    pub fn about() -> Image<Handle> {
        icons::info()
    }

    /// Close / dismiss.
    #[must_use]
    pub fn close() -> Image<Handle> {
        icons::cross()
    }

    /// Edit action (e.g., edit metadata).
    /// Returns dark icon for light theme, light icon for dark theme.
    #[must_use]
    pub fn edit(is_dark_theme: bool) -> Image<Handle> {
        if is_dark_theme {
            icons::light::pencil()
        } else {
            icons::pencil()
        }
    }

    /// Collapse a left-side panel (chevron points left).
    /// Returns dark icon for light theme, light icon for dark theme.
    #[must_use]
    pub fn collapse_left_panel(is_dark_theme: bool) -> Image<Handle> {
        if is_dark_theme {
            icons::light::chevron_double_left()
        } else {
            icons::chevron_double_left()
        }
    }

    /// Expand a left-side panel (chevron points right).
    /// Returns dark icon for light theme, light icon for dark theme.
    #[must_use]
    pub fn expand_left_panel(is_dark_theme: bool) -> Image<Handle> {
        if is_dark_theme {
            icons::light::chevron_double_right()
        } else {
            icons::chevron_double_right()
        }
    }

    /// Collapse a right-side panel (chevron points right).
    /// Returns dark icon for light theme, light icon for dark theme.
    #[must_use]
    pub fn collapse_right_panel(is_dark_theme: bool) -> Image<Handle> {
        if is_dark_theme {
            icons::light::chevron_double_right()
        } else {
            icons::chevron_double_right()
        }
    }

    /// Expand a right-side panel (chevron points left).
    /// Returns dark icon for light theme, light icon for dark theme.
    #[must_use]
    pub fn expand_right_panel(is_dark_theme: bool) -> Image<Handle> {
        if is_dark_theme {
            icons::light::chevron_double_left()
        } else {
            icons::chevron_double_left()
        }
    }

    /// Navigate to previous media.
    /// Returns appropriate icon based on background theme:
    /// - Dark background: light icon for visibility
    /// - Light/Checkerboard: dark icon for visibility
    #[must_use]
    pub fn previous(background: BackgroundTheme) -> Image<Handle> {
        match background {
            BackgroundTheme::Dark => icons::overlay::chevron_left(),
            BackgroundTheme::Light | BackgroundTheme::Checkerboard => icons::chevron_left(),
        }
    }

    /// Navigate to next media.
    /// Returns appropriate icon based on background theme:
    /// - Dark background: light icon for visibility
    /// - Light/Checkerboard: dark icon for visibility
    #[must_use]
    pub fn next(background: BackgroundTheme) -> Image<Handle> {
        match background {
            BackgroundTheme::Dark => icons::overlay::chevron_right(),
            BackgroundTheme::Light | BackgroundTheme::Checkerboard => icons::chevron_right(),
        }
    }

    /// Loop indicator for wrap-around navigation at boundaries.
    /// Returns appropriate icon based on background theme:
    /// - Dark background: light icon for visibility
    /// - Light/Checkerboard: dark icon for visibility
    #[must_use]
    pub fn loop_indicator(background: BackgroundTheme) -> Image<Handle> {
        match background {
            BackgroundTheme::Dark => icons::overlay::loop_icon(),
            BackgroundTheme::Light | BackgroundTheme::Checkerboard => icons::loop_icon(),
        }
    }
}

// =============================================================================
// Notification Severity Icons
// =============================================================================

/// Icons for notification severities.
pub mod notification {
    use super::icons;
    use iced::widget::image::{Handle, Image};

    /// Success notification.
    #[must_use]
    pub fn success() -> Image<Handle> {
        icons::checkmark()
    }

    /// Warning notification.
    #[must_use]
    pub fn warning() -> Image<Handle> {
        icons::warning()
    }

    /// Error notification.
    #[must_use]
    pub fn error() -> Image<Handle> {
        icons::warning()
    }

    /// Info notification.
    #[must_use]
    pub fn info() -> Image<Handle> {
        icons::info()
    }
}

// =============================================================================
// Help Section Icons
// =============================================================================

/// Icons for help screen sections.
pub mod sections {
    use super::icons;
    use iced::widget::image::{Handle, Image};

    /// Image/video viewer section.
    #[must_use]
    pub fn viewer() -> Image<Handle> {
        icons::image()
    }

    /// Video playback section.
    #[must_use]
    pub fn video() -> Image<Handle> {
        icons::video_camera()
    }

    /// Frame capture section.
    #[must_use]
    pub fn capture() -> Image<Handle> {
        icons::camera()
    }

    /// Image editor section.
    #[must_use]
    pub fn editor() -> Image<Handle> {
        icons::rotate_right()
    }

    /// Metadata editing section.
    #[must_use]
    pub fn metadata() -> Image<Handle> {
        icons::info()
    }
}

// =============================================================================
// Expand/Collapse Indicators
// =============================================================================

/// Icons for expand/collapse indicators in collapsible sections.
pub mod collapse {
    use super::icons;
    use iced::widget::image::{Handle, Image};

    /// Section is expanded (chevron pointing down).
    #[must_use]
    pub fn expanded(is_dark_theme: bool) -> Image<Handle> {
        if is_dark_theme {
            icons::light::chevron_down()
        } else {
            icons::chevron_down()
        }
    }

    /// Section is collapsed (chevron pointing right).
    #[must_use]
    pub fn collapsed(is_dark_theme: bool) -> Image<Handle> {
        if is_dark_theme {
            icons::light::chevron_right()
        } else {
            icons::chevron_right()
        }
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
    use crate::config::BackgroundTheme;

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
        let _ = video::speed_down();
        let _ = video::speed_up();
    }

    #[test]
    fn editor_icons_load() {
        let _ = editor::rotate_left();
        let _ = editor::rotate_right();
        let _ = editor::flip_horizontal();
        let _ = editor::flip_vertical();
        let _ = editor::navigate_previous();
        let _ = editor::navigate_next();
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
        // Test media navigation icons with all background themes
        let _ = navigation::previous(BackgroundTheme::Light);
        let _ = navigation::previous(BackgroundTheme::Dark);
        let _ = navigation::previous(BackgroundTheme::Checkerboard);
        let _ = navigation::next(BackgroundTheme::Light);
        let _ = navigation::next(BackgroundTheme::Dark);
        let _ = navigation::next(BackgroundTheme::Checkerboard);
        let _ = navigation::loop_indicator(BackgroundTheme::Light);
        let _ = navigation::loop_indicator(BackgroundTheme::Dark);
        let _ = navigation::loop_indicator(BackgroundTheme::Checkerboard);
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
        let _ = sections::metadata();
    }

    #[test]
    fn collapse_icons_load() {
        let _ = collapse::expanded(false);
        let _ = collapse::expanded(true);
        let _ = collapse::collapsed(false);
        let _ = collapse::collapsed(true);
    }

    #[test]
    fn video_toolbar_icons_load() {
        let _ = video::toolbar::play();
        let _ = video::toolbar::pause();
        let _ = video::toolbar::step_forward();
        let _ = video::toolbar::step_backward();
        let _ = video::toolbar::capture_frame();
        let _ = video::toolbar::toggle_loop();
        let _ = video::toolbar::volume();
        let _ = video::toolbar::volume_muted();
        let _ = video::toolbar::more_options();
        let _ = video::toolbar::speed_down();
        let _ = video::toolbar::speed_up();
    }

    #[test]
    fn viewer_toolbar_icons_load() {
        let _ = viewer::toolbar::zoom_in();
        let _ = viewer::toolbar::zoom_out();
        let _ = viewer::toolbar::zoom_reset();
        let _ = viewer::toolbar::fit_to_window();
        let _ = viewer::toolbar::expand();
        let _ = viewer::toolbar::fullscreen();
        let _ = viewer::toolbar::delete();
    }
}
