// SPDX-License-Identifier: MPL-2.0
//! Container styles.

use crate::ui::design_tokens::{opacity, radius};
use iced::widget::container;
use iced::{Background, Border, Color, Theme};

/// Generic panel surface used for settings and editor side panels.
///
/// The color is derived from the active Iced `Theme` background, with a slight
/// opacity, so panels stay readable in both light and dark modes without
/// hard-coding colors.
pub fn panel(theme: &Theme) -> container::Style {
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
            radius: radius::LG.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
