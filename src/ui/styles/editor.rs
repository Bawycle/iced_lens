// SPDX-License-Identifier: MPL-2.0
//! Centralized styles for editor surfaces and panels.

use crate::ui::design_tokens::{opacity, radius};
use iced::widget::container;
use iced::{Background, Border, Color, Theme};

/// Brightness adjustment for panel backgrounds.
/// Dark themes get slightly lighter, light themes get slightly darker
/// to create visual separation from the main background.
const BRIGHTNESS_ADJUST_DARK: f32 = 0.10;
const BRIGHTNESS_ADJUST_LIGHT: f32 = 0.06;
const LUMINANCE_THRESHOLD: f32 = 1.5;

/// Style for the main editor toolbar.
///
/// Uses the current Iced `Theme` extended palette so the toolbar follows
/// the global theme mode (light/dark) while staying visually subtle.
pub fn toolbar(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    let base = palette.background.base.color;

    container::Style {
        background: Some(Background::Color(Color::from_rgba(
            base.r,
            base.g,
            base.b,
            opacity::SURFACE,
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
    let (r, g, b) = if luminance < LUMINANCE_THRESHOLD {
        // Dark theme: brighten slightly
        (
            (base.r + BRIGHTNESS_ADJUST_DARK).min(1.0),
            (base.g + BRIGHTNESS_ADJUST_DARK).min(1.0),
            (base.b + BRIGHTNESS_ADJUST_DARK).min(1.0),
        )
    } else {
        // Light theme: darken slightly
        (
            (base.r - BRIGHTNESS_ADJUST_LIGHT).max(0.0),
            (base.g - BRIGHTNESS_ADJUST_LIGHT).max(0.0),
            (base.b - BRIGHTNESS_ADJUST_LIGHT).max(0.0),
        )
    };

    container::Style {
        background: Some(Background::Color(Color::from_rgba(
            r,
            g,
            b,
            opacity::SURFACE,
        ))),
        border: Border {
            radius: radius::LG.into(),
            width: 0.0,
            ..Default::default()
        },
        ..Default::default()
    }
}
