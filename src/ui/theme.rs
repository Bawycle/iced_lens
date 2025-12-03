// SPDX-License-Identifier: MPL-2.0
//! Shared UI color helpers to keep major surfaces visually consistent.

use crate::config::BackgroundTheme;
use iced::widget::{canvas, container, Container, Stack};
use iced::{mouse, Background, Border, Color, Element, Length, Rectangle, Theme};

/// Background color used by the viewer toolbar/header.
pub fn viewer_toolbar_background() -> Color {
    Color::from_rgb8(32, 33, 36)
}

/// Flat color used when the viewer/editor background theme is set to "Light".
pub fn viewer_light_surface_color() -> Color {
    Color::from_rgb8(245, 245, 245)
}

/// Flat color used when the viewer/editor background theme is set to "Dark".
pub fn viewer_dark_surface_color() -> Color {
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

/// Returns `true` if the configured background theme expects a checkerboard surface.
pub fn is_checkerboard(theme: BackgroundTheme) -> bool {
    matches!(theme, BackgroundTheme::Checkerboard)
}

/// Helper to wrap content with a checkerboard backdrop.
pub fn wrap_with_checkerboard<'a, Message: 'a>(
    content: Container<'a, Message>,
) -> Element<'a, Message> {
    Stack::new()
        .push(
            canvas::Canvas::new(CheckerboardBackground)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .push(content)
        .into()
}

/// Lightweight checkerboard pattern shared across viewer/editor backgrounds.
#[derive(Debug, Clone, Copy, Default)]
pub struct CheckerboardBackground;

impl CheckerboardBackground {
    const TILE: f32 = 20.0;
}

impl<Message> canvas::Program<Message> for CheckerboardBackground {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let tile = Self::TILE;
        let light = Color::from_rgb(0.85, 0.85, 0.85);
        let dark = Color::from_rgb(0.75, 0.75, 0.75);

        let cols = ((bounds.width / tile).ceil() as i32).max(1);
        let rows = ((bounds.height / tile).ceil() as i32).max(1);

        for row in 0..rows {
            for col in 0..cols {
                let color = if (row + col) % 2 == 0 { light } else { dark };
                let x = col as f32 * tile;
                let y = row as f32 * tile;
                let path = canvas::Path::rectangle(
                    iced::Point::new(x, y),
                    iced::Size::new(tile + 0.5, tile + 0.5),
                );
                frame.fill(&path, color);
            }
        }

        vec![frame.into_geometry()]
    }
}
