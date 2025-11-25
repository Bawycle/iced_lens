// SPDX-License-Identifier: MPL-2.0
//! Shared UI color helpers to keep major surfaces visually consistent.

use iced::widget::container;
use iced::{Background, Border, Color, Theme};

/// Background color used by the viewer toolbar/header.
pub fn viewer_toolbar_background() -> Color {
    Color::from_rgb8(32, 33, 36)
}

/// Match the settings panel surface styling so other panels can stay in sync.
pub fn settings_panel_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    let base = palette.background.base.color;
    let luminance = base.r + base.g + base.b;
    let (r, g, b) = if luminance < 1.5 {
        (
            (base.r + 0.10).min(1.0),
            (base.g + 0.10).min(1.0),
            (base.b + 0.10).min(1.0),
        )
    } else {
        (
            (base.r - 0.06).max(0.0),
            (base.g - 0.06).max(0.0),
            (base.b - 0.06).max(0.0),
        )
    };

    container::Style {
        background: Some(Background::Color(Color::from_rgba(r, g, b, 0.95))),
        border: Border {
            radius: 12.0.into(),
            width: 0.0,
            ..Default::default()
        },
        ..Default::default()
    }
}
