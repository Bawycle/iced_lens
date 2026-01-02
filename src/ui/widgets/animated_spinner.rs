// SPDX-License-Identifier: MPL-2.0
//! Animated spinner widget using Canvas for smooth rotation.

use crate::ui::design_tokens::sizing;
use iced::widget::canvas::{self, Cache, Canvas, Frame, Geometry, Path, Stroke};
use iced::{mouse, Color, Length, Point, Rectangle, Renderer, Theme};
use std::f32::consts::PI;

/// Animated spinner that rotates smoothly.
pub struct AnimatedSpinner {
    cache: Cache,
    rotation: f32, // Rotation angle in radians
    color: Color,
    size: f32,
}

impl AnimatedSpinner {
    /// Creates a new animated spinner with the given color and rotation angle.
    #[must_use]
    pub fn new(color: Color, rotation: f32) -> Self {
        Self {
            cache: Cache::default(),
            rotation,
            color,
            size: sizing::ICON_XL,
        }
    }

    /// Updates the rotation angle and invalidates the cache.
    #[must_use]
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self.cache.clear();
        self
    }

    /// Creates a Canvas widget from this spinner.
    pub fn into_element<Message: 'static>(self) -> iced::Element<'static, Message> {
        let size = self.size;
        Canvas::new(self)
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .into()
    }
}

impl<Message> canvas::Program<Message> for AnimatedSpinner {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self
            .cache
            .draw(renderer, bounds.size(), |frame: &mut Frame| {
                let center = frame.center();
                let radius = frame.width().min(frame.height()) / 2.0 - 4.0;

                // Draw background circle (subtle)
                let background_circle = Path::circle(center, radius);
                frame.stroke(
                    &background_circle,
                    Stroke::default().with_width(3.0).with_color(Color {
                        a: 0.25,
                        ..self.color
                    }),
                );

                // Draw rotating arc (the animated part)
                // Arc from rotation angle to rotation + 180°
                let start_angle = self.rotation - PI / 2.0; // -90° offset to start at top
                let end_angle = start_angle + PI; // 180° arc

                // Build arc path manually
                let mut arc_path = canvas::path::Builder::new();

                // Move to start point
                let start_x = center.x + radius * start_angle.cos();
                let start_y = center.y + radius * start_angle.sin();
                arc_path.move_to(Point::new(start_x, start_y));

                // Draw arc using multiple small line segments for smooth appearance
                let segments = 30;
                #[allow(clippy::cast_precision_loss)]
                // segments=30, i∈[1,30] - well within f32 precision
                for i in 1..=segments {
                    let t = i as f32 / segments as f32;
                    let angle = start_angle + (end_angle - start_angle) * t;
                    let x = center.x + radius * angle.cos();
                    let y = center.y + radius * angle.sin();
                    arc_path.line_to(Point::new(x, y));
                }

                let arc = arc_path.build();
                frame.stroke(
                    &arc,
                    Stroke::default()
                        .with_width(3.0)
                        .with_color(self.color)
                        .with_line_cap(canvas::LineCap::Round),
                );
            });

        vec![geometry]
    }
}
