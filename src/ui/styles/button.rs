// SPDX-License-Identifier: MPL-2.0
//! Centralized button styles.

use crate::ui::design_tokens::{
    opacity,
    palette::{self, BLACK, WHITE},
    radius, shadow,
};
use iced::widget::button;
use iced::{Background, Border, Color, Theme};

/// Style pour bouton primaire (action principale).
pub fn primary(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active | button::Status::Pressed => button::Style {
            background: Some(Background::Color(palette::PRIMARY_500)),
            text_color: WHITE,
            border: Border {
                color: palette::PRIMARY_600,
                width: 1.0,
                radius: radius::SM.into(),
            },
            shadow: shadow::SM,
            snap: true,
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(palette::PRIMARY_400)),
            text_color: WHITE,
            border: Border {
                color: palette::PRIMARY_500,
                width: 1.0,
                radius: radius::SM.into(),
            },
            shadow: shadow::MD,
            snap: true,
        },
        _ => button::Style::default(),
    }
}

/// Style pour boutons overlay (navigation, play, etc.).
pub fn overlay(
    text_color: Color,
    alpha_normal: f32,
    alpha_hover: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let alpha = match status {
            button::Status::Hovered => alpha_hover,
            button::Status::Pressed => opacity::OVERLAY_PRESSED,
            _ => alpha_normal,
        };

        button::Style {
            background: Some(Background::Color(Color { a: alpha, ..BLACK })),
            text_color,
            border: Border::default(),
            shadow: shadow::MD,
            snap: true,
        }
    }
}

/// Style pour bouton désactivé (grayed out, non-interactif).
///
/// Very faded appearance with low contrast to clearly indicate non-interactivity.
/// Adapts to Light/Dark theme.
pub fn disabled() -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme: &Theme, _status: button::Status| {
        let is_light = matches!(theme, Theme::Light);

        // Low contrast appearance, clearly non-interactive
        let (bg_color, text_color, border_color) = if is_light {
            (
                Color::from_rgba(0.9, 0.9, 0.9, 0.5), // Semi-transparent light gray
                palette::GRAY_400,                    // Faded text
                Color::from_rgba(0.7, 0.7, 0.7, 0.3), // Very subtle border
            )
        } else {
            (
                Color::from_rgba(0.25, 0.25, 0.25, 0.7), // More visible dark gray
                palette::GRAY_400,                       // Faded but readable text
                Color::from_rgba(0.35, 0.35, 0.35, 0.5), // Subtle but visible border
            )
        };

        button::Style {
            background: Some(Background::Color(bg_color)),
            text_color,
            border: Border {
                color: border_color,
                width: 1.0,
                radius: radius::SM.into(),
            },
            shadow: shadow::NONE,
            snap: true,
        }
    }
}

/// Style for selected state in button groups (radio buttons, segmented controls).
///
/// Use this when one option is selected among multiple mutually exclusive choices.
/// For on/off toggle buttons, use `toggle_active` instead.
pub fn selected(theme: &Theme, status: button::Status) -> button::Style {
    let is_light = matches!(theme, Theme::Light);

    match status {
        button::Status::Active | button::Status::Pressed => button::Style {
            background: Some(Background::Color(palette::PRIMARY_500)),
            text_color: WHITE,
            border: Border {
                color: palette::PRIMARY_600,
                width: 1.0,
                radius: radius::SM.into(),
            },
            shadow: shadow::SM,
            snap: true,
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(palette::PRIMARY_400)),
            text_color: WHITE,
            border: Border {
                color: palette::PRIMARY_500,
                width: 1.0,
                radius: radius::SM.into(),
            },
            shadow: shadow::MD,
            snap: true,
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(if is_light {
                palette::GRAY_200
            } else {
                palette::GRAY_700
            })),
            text_color: palette::GRAY_400,
            border: Border {
                color: palette::GRAY_400,
                width: 1.0,
                radius: radius::SM.into(),
            },
            shadow: shadow::NONE,
            snap: true,
        },
    }
}

/// Style for unselected state in button groups (radio buttons, segmented controls).
///
/// Use this for non-selected options in mutually exclusive choice groups.
/// Has clear visual distinction from disabled buttons: solid border, full contrast text.
/// For on/off toggle buttons in inactive state, use the default button style.
pub fn unselected(theme: &Theme, status: button::Status) -> button::Style {
    let is_light = matches!(theme, Theme::Light);

    // Clear, interactive appearance with solid border (distinct from faded disabled style)
    let (bg_color, text_color, border_color) = if is_light {
        (
            palette::GRAY_100,
            palette::GRAY_900, // Full contrast text
            palette::GRAY_400, // Solid visible border
        )
    } else {
        (
            palette::GRAY_700,
            WHITE, // Full contrast text
            palette::GRAY_400,
        )
    };

    match status {
        button::Status::Active | button::Status::Pressed => button::Style {
            background: Some(Background::Color(bg_color)),
            text_color,
            border: Border {
                color: border_color,
                width: 1.0,
                radius: radius::SM.into(),
            },
            shadow: shadow::NONE,
            snap: true,
        },
        button::Status::Hovered => {
            let hover_bg = if is_light {
                palette::GRAY_200
            } else {
                Color::from_rgb(0.35, 0.35, 0.35)
            };
            button::Style {
                background: Some(Background::Color(hover_bg)),
                text_color,
                border: Border {
                    color: palette::PRIMARY_500,
                    width: 1.0,
                    radius: radius::SM.into(),
                },
                shadow: shadow::SM,
                snap: true,
            }
        }
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(if is_light {
                palette::GRAY_100
            } else {
                palette::GRAY_700
            })),
            text_color: palette::GRAY_400,
            border: Border {
                color: palette::GRAY_400,
                width: 1.0,
                radius: radius::SM.into(),
            },
            shadow: shadow::NONE,
            snap: true,
        },
    }
}

/// Style pour bouton play overlay vidéo.
pub fn video_play_overlay() -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let alpha = match status {
            button::Status::Hovered => opacity::OVERLAY_HOVER,
            button::Status::Pressed => opacity::OVERLAY_STRONG,
            _ => opacity::OVERLAY_MEDIUM,
        };

        button::Style {
            background: Some(Background::Color(Color { a: alpha, ..BLACK })),
            text_color: WHITE,
            border: Border {
                radius: radius::FULL.into(),
                ..Default::default()
            },
            shadow: shadow::LG,
            snap: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_button_uses_brand_colors() {
        let theme = Theme::Dark;
        let style = primary(&theme, button::Status::Active);

        if let Some(Background::Color(bg)) = style.background {
            assert_eq!(bg, palette::PRIMARY_500);
        } else {
            panic!("Expected background color");
        }
    }

    #[test]
    fn overlay_button_alpha_changes_on_hover() {
        let theme = Theme::Dark;
        let style_fn = overlay(WHITE, 0.5, 0.8);

        let normal = style_fn(&theme, button::Status::Active);
        let hover = style_fn(&theme, button::Status::Hovered);

        // Extract alpha values (would need helper)
        // This is a simplified test
        assert_ne!(normal.background, hover.background);
    }
}
