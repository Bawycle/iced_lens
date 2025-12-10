// SPDX-License-Identifier: MPL-2.0
//! Centralized styles for editor surfaces and panels.

use crate::ui::design_tokens::radius;
use iced::widget::container;
use iced::{Background, Border, Color, Theme};

/// Style for the main editor toolbar.
///
/// Uses the current Iced `Theme` extended palette so the toolbar follows
/// the global theme mode (light/dark) while staying visually subtle.
pub fn toolbar(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    let base = palette.background.base.color;

    container::Style {
        background: Some(Background::Color(Color::from_rgba(
            base.r, base.g, base.b, 0.95,
        ))),
        border: Border {
            width: 0.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Style for settings-like panels in the editor sidebar.
/// Matches the settings panel surface so other panels can stay visually in sync.
pub fn settings_panel(theme: &Theme) -> container::Style {
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
            radius: radius::LG.into(),
            width: 0.0,
            ..Default::default()
        },
        ..Default::default()
    }
}
