// SPDX-License-Identifier: MPL-2.0
//! Shared UI color helpers and overlay styles for the viewer and editor.

use crate::config::BackgroundTheme;
use crate::ui::design_tokens::{
    opacity,
    palette::{self, BLACK, GRAY_100, GRAY_900, WHITE},
};
use iced::widget::container;
use iced::{Color, Theme};

/// Background color used by the viewer toolbar/header.
pub fn viewer_toolbar_background() -> Color {
    GRAY_900
}

/// Flat color used when the viewer/editor background theme is set to "Light".
pub fn viewer_light_surface_color() -> Color {
    GRAY_100
}

/// Flat color used when the viewer/editor background theme is set to "Dark".
pub fn viewer_dark_surface_color() -> Color {
    GRAY_900
}

/// Standard color for error text.
pub fn error_text_color() -> Color {
    palette::ERROR_500
}

/// Standard color for error icons and accents.
pub fn error_color() -> Color {
    palette::ERROR_500
}

/// Standard color for success text.
pub fn success_text_color() -> Color {
    palette::SUCCESS_500
}

/// Standard color for muted/secondary text.
pub fn muted_text_color() -> Color {
    palette::GRAY_400
}

// ============================================================================
// Fullscreen Overlay Styles
// ============================================================================
// Shared colors and styling for overlay elements (navigation arrows, HUD, counters)
// in fullscreen mode, ensuring consistent visual appearance across all overlays.

/// Dark text color for navigation arrows on light backgrounds.
pub fn overlay_arrow_dark_color() -> Color {
    GRAY_900
}
/// White text color for navigation arrows on dark backgrounds.
pub fn overlay_arrow_light_color() -> Color {
    WHITE
}

/// Style for the editor canvas background.
pub fn editor_canvas_style(background_color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(background_color)),
        ..Default::default()
    }
}

// ============================================================================
// Editor Overlay Styles
// ============================================================================

/// Color for the resize overlay preview rectangle and text.
pub fn resize_overlay_color() -> Color {
    palette::INFO_500
}

/// Color of the darkened overlay outside the crop area.
pub fn crop_overlay_outside_color() -> Color {
    Color {
        a: opacity::OVERLAY_MEDIUM,
        ..BLACK
    }
}

/// Color of the rule-of-thirds grid in the crop overlay.
pub fn crop_overlay_grid_color() -> Color {
    Color {
        a: opacity::OVERLAY_MEDIUM,
        ..WHITE
    }
}

/// Fill color for crop resize handles.
pub fn crop_overlay_handle_color() -> Color {
    WHITE
}

/// Border color for crop resize handles.
pub fn crop_overlay_handle_border_color() -> Color {
    BLACK
}

/// Returns `true` if the configured background theme expects a checkerboard surface.
pub fn is_checkerboard(theme: BackgroundTheme) -> bool {
    matches!(theme, BackgroundTheme::Checkerboard)
}
