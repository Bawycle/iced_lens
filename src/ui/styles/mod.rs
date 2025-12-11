// SPDX-License-Identifier: MPL-2.0
//! Styles centralisés pour tous les composants UI.

pub mod button;
pub mod container;
pub mod editor;
pub mod overlay;

// Re-exports pour backward compatibility
pub use button::{overlay as button_overlay, primary as button_primary};

use iced::widget::svg;
use iced::Theme;

/// Style for SVG icons that tints them with the theme's primary text color.
/// Useful for section header icons that should match the surrounding text.
pub fn tinted_svg(theme: &Theme, _status: svg::Status) -> svg::Style {
    svg::Style {
        color: Some(theme.extended_palette().primary.strong.color),
    }
}
