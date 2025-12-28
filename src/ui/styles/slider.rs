// SPDX-License-Identifier: MPL-2.0
//! Slider-specific style definitions.
//!
//! Provides consistent styling for sliders across the application.

use crate::ui::design_tokens::palette;
use iced::widget::slider;
use iced::{Background, Border, Color, Theme};

/// Style for disabled slider (grayed out, non-interactive).
///
/// Very faded appearance with low contrast to clearly indicate non-interactivity.
/// Adapts to Light/Dark theme.
pub fn disabled() -> impl Fn(&Theme, slider::Status) -> slider::Style {
    move |theme: &Theme, _status: slider::Status| {
        let is_light = matches!(theme, Theme::Light);

        // Low contrast appearance, clearly non-interactive
        let (rail_bg, handle_bg, handle_border) = if is_light {
            (
                Color::from_rgba(0.85, 0.85, 0.85, 0.6), // Semi-transparent light gray
                Color::from_rgba(0.9, 0.9, 0.9, 0.7),    // Faded handle
                Color::from_rgba(0.7, 0.7, 0.7, 0.3),    // Very subtle border
            )
        } else {
            (
                Color::from_rgba(0.3, 0.3, 0.3, 0.6), // Semi-transparent dark gray
                Color::from_rgba(0.35, 0.35, 0.35, 0.7), // Faded handle
                Color::from_rgba(0.4, 0.4, 0.4, 0.5), // Subtle but visible border
            )
        };

        slider::Style {
            rail: slider::Rail {
                backgrounds: (Background::Color(rail_bg), Background::Color(rail_bg)),
                width: 4.0,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 2.0.into(),
                },
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 6.0 },
                background: Background::Color(handle_bg),
                border_width: 1.0,
                border_color: handle_border,
            },
        }
    }
}

/// Returns a text style for disabled volume percentage.
/// Matches the disabled slider appearance. Adapts to Light/Dark theme.
#[must_use]
pub fn disabled_text_style(theme: &Theme) -> iced::widget::text::Style {
    let is_light = matches!(theme, Theme::Light);
    // Use palette colors that provide appropriate contrast for each theme
    let color = if is_light {
        palette::GRAY_400 // Darker gray on light background
    } else {
        palette::GRAY_200 // Lighter gray on dark background
    };
    iced::widget::text::Style { color: Some(color) }
}
