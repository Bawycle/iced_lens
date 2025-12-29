// SPDX-License-Identifier: MPL-2.0
//! Overlay styles for fullscreen controls, HUD, and position counters.

use crate::ui::design_tokens::{
    opacity,
    palette::{BLACK, WHITE},
};
use iced::widget::{container, svg};
use iced::{Background, Border, Color, Theme};

fn container_background() -> Color {
    Color {
        a: opacity::OVERLAY_STRONG,
        ..BLACK
    }
}

fn container_border() -> Color {
    Color {
        a: opacity::OVERLAY_SUBTLE,
        ..WHITE
    }
}

/// Generic style for overlay indicators like the HUD and position counter.
pub fn indicator(rad: f32) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| container::Style {
        background: Some(Background::Color(container_background())),
        text_color: Some(WHITE),
        border: Border {
            color: container_border(),
            width: 1.0,
            radius: rad.into(),
        },
        ..Default::default()
    }
}

/// Style for the overlay controls container in fullscreen mode.
#[must_use] 
pub fn controls_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(container_background())),
        text_color: Some(WHITE),
        ..Default::default()
    }
}

/// Style for loop/navigation SVG icons in overlays.
pub fn loop_icon(color: Color) -> impl Fn(&Theme, svg::Status) -> svg::Style {
    move |_theme: &Theme, _status: svg::Status| svg::Style { color: Some(color) }
}

/// Style for the central play SVG icon in video overlays.
pub fn play_icon(color: Color) -> impl Fn(&Theme, svg::Status) -> svg::Style {
    move |_theme: &Theme, _status: svg::Status| svg::Style { color: Some(color) }
}
