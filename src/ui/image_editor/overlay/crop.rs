// SPDX-License-Identifier: MPL-2.0
//! Crop overlay renderer for interactive crop selection.

use crate::ui::design_tokens::sizing;
use crate::ui::image_editor::{CanvasMessage, Message};
use crate::ui::theme;

/// Canvas program used to draw and interact with the crop overlay.
pub struct CropOverlayRenderer {
    pub crop_x: u32,
    pub crop_y: u32,
    pub crop_width: u32,
    pub crop_height: u32,
    pub img_width: u32,
    pub img_height: u32,
}

impl CropOverlayRenderer {
    /// Convert screen coordinates to image coordinates (clamped to image bounds)
    fn screen_to_image_coords(
        &self,
        screen_pos: iced::Point,
        bounds: iced::Rectangle,
    ) -> Option<(f32, f32)> {
        // Calculate image position and scale (ContentFit::Contain logic)
        let img_aspect = self.img_width as f32 / self.img_height as f32;
        let bounds_aspect = bounds.width / bounds.height;

        let (img_display_width, img_display_height, img_offset_x, img_offset_y) =
            if img_aspect > bounds_aspect {
                let display_width = bounds.width;
                let display_height = bounds.width / img_aspect;
                let offset_y = (bounds.height - display_height) / 2.0;
                (display_width, display_height, 0.0, offset_y)
            } else {
                let display_height = bounds.height;
                let display_width = bounds.height * img_aspect;
                let offset_x = (bounds.width - display_width) / 2.0;
                (display_width, display_height, offset_x, 0.0)
            };

        // Clamp screen coordinates to image display area
        let clamped_x = screen_pos
            .x
            .max(img_offset_x)
            .min(img_offset_x + img_display_width);
        let clamped_y = screen_pos
            .y
            .max(img_offset_y)
            .min(img_offset_y + img_display_height);

        // Convert to image coordinates
        let img_x = ((clamped_x - img_offset_x) * (self.img_width as f32 / img_display_width))
            .max(0.0)
            .min(self.img_width as f32);
        let img_y = ((clamped_y - img_offset_y) * (self.img_height as f32 / img_display_height))
            .max(0.0)
            .min(self.img_height as f32);

        Some((img_x, img_y))
    }
}

impl iced::widget::canvas::Program<Message> for CropOverlayRenderer {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        bounds: iced::Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> Option<iced::widget::Action<Message>> {
        use iced::widget::Action;

        match event {
            // If cursor leaves the canvas, end any drag operation
            iced::Event::Mouse(iced::mouse::Event::CursorLeft) => {
                return Some(
                    Action::publish(Message::Canvas(CanvasMessage::CropOverlayMouseUp))
                        .and_capture(),
                );
            }
            iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
                if let Some(cursor_position) = cursor.position_in(bounds) {
                    if let Some((img_x, img_y)) =
                        self.screen_to_image_coords(cursor_position, bounds)
                    {
                        return Some(
                            Action::publish(Message::Canvas(CanvasMessage::CropOverlayMouseDown {
                                x: img_x,
                                y: img_y,
                            }))
                            .and_capture(),
                        );
                    }
                }
            }
            iced::Event::Mouse(iced::mouse::Event::CursorMoved { .. }) => {
                // If cursor is outside bounds during move, end drag
                if cursor.position_in(bounds).is_none() {
                    return Some(
                        Action::publish(Message::Canvas(CanvasMessage::CropOverlayMouseUp))
                            .and_capture(),
                    );
                }

                if let Some(cursor_position) = cursor.position_in(bounds) {
                    if let Some((img_x, img_y)) =
                        self.screen_to_image_coords(cursor_position, bounds)
                    {
                        return Some(
                            Action::publish(Message::Canvas(CanvasMessage::CropOverlayMouseMove {
                                x: img_x,
                                y: img_y,
                            }))
                            .and_capture(),
                        );
                    }
                }
            }
            iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                return Some(
                    Action::publish(Message::Canvas(CanvasMessage::CropOverlayMouseUp))
                        .and_capture(),
                );
            }
            _ => {}
        }

        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        use iced::widget::canvas::{Frame, Path, Stroke};

        let mut frame = Frame::new(renderer, bounds.size());

        // Calculate image position and scale (ContentFit::Contain logic)
        let img_aspect = self.img_width as f32 / self.img_height as f32;
        let bounds_aspect = bounds.width / bounds.height;

        let (img_display_width, img_display_height, img_offset_x, img_offset_y) =
            if img_aspect > bounds_aspect {
                // Image is wider - fit to width
                let display_width = bounds.width;
                let display_height = bounds.width / img_aspect;
                let offset_y = (bounds.height - display_height) / 2.0;
                (display_width, display_height, 0.0, offset_y)
            } else {
                // Image is taller - fit to height
                let display_height = bounds.height;
                let display_width = bounds.height * img_aspect;
                let offset_x = (bounds.width - display_width) / 2.0;
                (display_width, display_height, offset_x, 0.0)
            };

        // Scale factors
        let scale_x = img_display_width / self.img_width as f32;
        let scale_y = img_display_height / self.img_height as f32;

        // Convert crop coordinates from image space to screen space
        let crop_screen_x = img_offset_x + self.crop_x as f32 * scale_x;
        let crop_screen_y = img_offset_y + self.crop_y as f32 * scale_y;
        let crop_screen_width = self.crop_width as f32 * scale_x;
        let crop_screen_height = self.crop_height as f32 * scale_y;

        // Draw darkened overlay outside crop area
        let dark_overlay = crate::ui::theme::crop_overlay_outside_color();

        // Top rectangle
        if crop_screen_y > img_offset_y {
            frame.fill_rectangle(
                iced::Point::new(img_offset_x, img_offset_y),
                iced::Size::new(img_display_width, crop_screen_y - img_offset_y),
                dark_overlay,
            );
        }

        // Bottom rectangle
        let bottom_y = crop_screen_y + crop_screen_height;
        if bottom_y < img_offset_y + img_display_height {
            frame.fill_rectangle(
                iced::Point::new(img_offset_x, bottom_y),
                iced::Size::new(
                    img_display_width,
                    img_offset_y + img_display_height - bottom_y,
                ),
                dark_overlay,
            );
        }

        // Left rectangle
        if crop_screen_x > img_offset_x {
            frame.fill_rectangle(
                iced::Point::new(img_offset_x, crop_screen_y),
                iced::Size::new(crop_screen_x - img_offset_x, crop_screen_height),
                dark_overlay,
            );
        }

        // Right rectangle
        let right_x = crop_screen_x + crop_screen_width;
        if right_x < img_offset_x + img_display_width {
            frame.fill_rectangle(
                iced::Point::new(right_x, crop_screen_y),
                iced::Size::new(
                    img_offset_x + img_display_width - right_x,
                    crop_screen_height,
                ),
                dark_overlay,
            );
        }

        // Draw crop rectangle border
        let crop_rect = Path::rectangle(
            iced::Point::new(crop_screen_x, crop_screen_y),
            iced::Size::new(crop_screen_width, crop_screen_height),
        );
        frame.stroke(
            &crop_rect,
            Stroke::default()
                .with_width(2.0)
                .with_color(theme::crop_overlay_handle_color()),
        );

        // Draw rule-of-thirds grid
        let grid_color = theme::crop_overlay_grid_color();
        let third_width = crop_screen_width / 3.0;
        let third_height = crop_screen_height / 3.0;

        // Vertical lines
        for i in 1..3 {
            let x = crop_screen_x + third_width * i as f32;
            let line = Path::line(
                iced::Point::new(x, crop_screen_y),
                iced::Point::new(x, crop_screen_y + crop_screen_height),
            );
            frame.stroke(
                &line,
                Stroke::default().with_width(1.0).with_color(grid_color),
            );
        }

        // Horizontal lines
        for i in 1..3 {
            let y = crop_screen_y + third_height * i as f32;
            let line = Path::line(
                iced::Point::new(crop_screen_x, y),
                iced::Point::new(crop_screen_x + crop_screen_width, y),
            );
            frame.stroke(
                &line,
                Stroke::default().with_width(1.0).with_color(grid_color),
            );
        }

        // Draw resize handles
        let handle_size = sizing::CROP_HANDLE_SIZE;
        let handle_color = theme::crop_overlay_handle_color();
        let handles = [
            (crop_screen_x, crop_screen_y),                           // TopLeft
            (crop_screen_x + crop_screen_width / 2.0, crop_screen_y), // Top
            (crop_screen_x + crop_screen_width, crop_screen_y),       // TopRight
            (
                crop_screen_x + crop_screen_width,
                crop_screen_y + crop_screen_height / 2.0,
            ), // Right
            (
                crop_screen_x + crop_screen_width,
                crop_screen_y + crop_screen_height,
            ), // BottomRight
            (
                crop_screen_x + crop_screen_width / 2.0,
                crop_screen_y + crop_screen_height,
            ), // Bottom
            (crop_screen_x, crop_screen_y + crop_screen_height),      // BottomLeft
            (crop_screen_x, crop_screen_y + crop_screen_height / 2.0), // Left
        ];

        for (hx, hy) in handles {
            let handle = Path::rectangle(
                iced::Point::new(hx - handle_size / 2.0, hy - handle_size / 2.0),
                iced::Size::new(handle_size, handle_size),
            );
            frame.fill(&handle, handle_color);
            frame.stroke(
                &handle,
                Stroke::default()
                    .with_width(1.0)
                    .with_color(theme::crop_overlay_handle_border_color()),
            );
        }

        vec![frame.into_geometry()]
    }
}
