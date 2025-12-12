// SPDX-License-Identifier: MPL-2.0
//! Checkerboard component used as a background for transparent content.

use crate::ui::design_tokens::palette;
use iced::widget::{canvas, Container, Stack};
use iced::{mouse, Color, Element, Length, Rectangle, Theme};

const TILE_SIZE: f32 = 20.0;
const LIGHT_TILE: Color = palette::GRAY_100;
const DARK_TILE: Color = palette::GRAY_200;

/// Checkerboard pattern widget.
#[derive(Debug, Clone, Copy, Default)]
pub struct Checkerboard;

impl<Message> canvas::Program<Message> for Checkerboard {
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

        let cols = ((bounds.width / TILE_SIZE).ceil() as i32).max(1);
        let rows = ((bounds.height / TILE_SIZE).ceil() as i32).max(1);

        for row in 0..rows {
            for col in 0..cols {
                let color = if (row + col) % 2 == 0 {
                    LIGHT_TILE
                } else {
                    DARK_TILE
                };
                let x = col as f32 * TILE_SIZE;
                let y = row as f32 * TILE_SIZE;
                let path = canvas::Path::rectangle(
                    iced::Point::new(x, y),
                    iced::Size::new(TILE_SIZE + 0.5, TILE_SIZE + 0.5),
                );
                frame.fill(&path, color);
            }
        }

        vec![frame.into_geometry()]
    }
}

/// Helper to wrap arbitrary content with a checkerboard background.
pub fn wrap<'a, Message: 'a>(content: Container<'a, Message>) -> Element<'a, Message> {
    Stack::new()
        .push(
            canvas::Canvas::new(Checkerboard)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .push(content)
        .into()
}

const _: () = {
    assert!(TILE_SIZE > 0.0);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_are_different() {
        assert_ne!(LIGHT_TILE, DARK_TILE);
    }
}
