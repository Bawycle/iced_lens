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
        }
    }
}

/// Style pour bouton désactivé (grayed out, non-interactif).
pub fn disabled() -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, _status: button::Status| button::Style {
        background: Some(Background::Color(palette::GRAY_200)),
        text_color: palette::GRAY_400,
        border: Border {
            color: palette::GRAY_400,
            width: 1.0,
            radius: radius::SM.into(),
        },
        shadow: shadow::NONE,
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_button_uses_brand_colors() {
        let theme = Theme::default();
        let style = primary(&theme, button::Status::Active);

        if let Some(Background::Color(bg)) = style.background {
            assert_eq!(bg, palette::PRIMARY_500);
        } else {
            panic!("Expected background color");
        }
    }

    #[test]
    fn overlay_button_alpha_changes_on_hover() {
        let theme = Theme::default();
        let style_fn = overlay(WHITE, 0.5, 0.8);

        let normal = style_fn(&theme, button::Status::Active);
        let hover = style_fn(&theme, button::Status::Hovered);

        // Extract alpha values (would need helper)
        // This is a simplified test
        assert_ne!(normal.background, hover.background);
    }
}
