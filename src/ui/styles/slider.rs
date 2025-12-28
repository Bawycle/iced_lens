// SPDX-License-Identifier: MPL-2.0
//! Slider-specific style definitions.
//!
//! Provides consistent styling for sliders across the application.

use crate::ui::design_tokens::{opacity, palette};
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
                Color {
                    a: 0.6,
                    ..palette::GRAY_100
                },
                Color {
                    a: opacity::DISABLED_DARK,
                    ..palette::DISABLED_LIGHT_BG
                },
                Color {
                    a: opacity::DISABLED_BORDER,
                    ..palette::GRAY_200
                },
            )
        } else {
            (
                Color {
                    a: 0.6,
                    ..palette::GRAY_700
                },
                Color {
                    a: opacity::DISABLED_DARK,
                    ..palette::DISABLED_DARK_BORDER
                },
                Color {
                    a: opacity::OVERLAY_MEDIUM,
                    ..palette::GRAY_400
                },
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
