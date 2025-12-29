// SPDX-License-Identifier: MPL-2.0
//! Resize overlay renderer showing original and target dimensions.
//!
//! Uses f32 for canvas coordinates and u32 for pixel dimensions.
//! Precision loss in conversions is acceptable for typical image sizes.
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use crate::ui::design_tokens::palette::WHITE;
use crate::ui::image_editor::Message;
use crate::ui::theme;

/// Canvas program used to draw resize previews.
pub struct ResizeOverlayRenderer {
    /// Original image dimensions (reference markers - white rectangle)
    pub original_width: u32,
    pub original_height: u32,
    /// New dimensions after resize (preview - blue rectangle)
    pub new_width: u32,
    pub new_height: u32,
}

impl iced::widget::canvas::Program<Message> for ResizeOverlayRenderer {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        _event: &iced::Event,
        _bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Option<iced::widget::Action<Message>> {
        // No interaction needed for resize overlay
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
        use iced::widget::canvas::{Frame, Path, Stroke, Text};

        let mut frame = Frame::new(renderer, bounds.size());

        // Calculate the bounding box that contains both original and new dimensions
        // This ensures both rectangles fit in the viewport
        let max_width = self.original_width.max(self.new_width);
        let max_height = self.original_height.max(self.new_height);

        // Use the max dimensions for ContentFit::Contain calculation
        let max_aspect = max_width as f32 / max_height as f32;
        let bounds_aspect = bounds.width / bounds.height;

        let (display_width, display_height, offset_x, offset_y) = if max_aspect > bounds_aspect {
            // Wider - fit to width
            let display_width = bounds.width;
            let display_height = display_width / max_aspect;
            let offset_y = (bounds.height - display_height) / 2.0;
            (display_width, display_height, 0.0, offset_y)
        } else {
            // Taller - fit to height
            let display_height = bounds.height;
            let display_width = display_height * max_aspect;
            let offset_x = (bounds.width - display_width) / 2.0;
            (display_width, display_height, offset_x, 0.0)
        };

        // Scale factors (how many screen pixels per image pixel)
        let scale_x = display_width / max_width as f32;
        let scale_y = display_height / max_height as f32;

        // Calculate screen dimensions for original and new sizes using the same scale
        let original_screen_width = self.original_width as f32 * scale_x;
        let original_screen_height = self.original_height as f32 * scale_y;
        let new_screen_width = self.new_width as f32 * scale_x;
        let new_screen_height = self.new_height as f32 * scale_y;

        // Center both rectangles within the display area
        let original_x = offset_x + (display_width - original_screen_width) / 2.0;
        let original_y = offset_y + (display_height - original_screen_height) / 2.0;
        let new_x = offset_x + (display_width - new_screen_width) / 2.0;
        let new_y = offset_y + (display_height - new_screen_height) / 2.0;

        // Draw the original dimensions marker first (white stroke, thick)
        let original_rect = Path::rectangle(
            iced::Point::new(original_x, original_y),
            iced::Size::new(original_screen_width, original_screen_height),
        );
        frame.stroke(
            &original_rect,
            Stroke::default().with_width(3.0).with_color(WHITE),
        );

        // Draw the resized image area on top (blue stroke only, no fill to see through)
        let new_rect = Path::rectangle(
            iced::Point::new(new_x, new_y),
            iced::Size::new(new_screen_width, new_screen_height),
        );
        frame.stroke(
            &new_rect,
            Stroke::default()
                .with_width(3.0)
                .with_color(theme::resize_overlay_color()),
        );

        // Draw dimension labels
        let label_color = WHITE;
        let font_size = 16.0;

        // Original dimensions label (top-left of original rect)
        let original_label = format!("Original: {}×{}", self.original_width, self.original_height);
        frame.fill_text(Text {
            content: original_label,
            position: iced::Point::new(original_x, original_y - 20.0),
            color: label_color,
            size: font_size.into(),
            ..Text::default()
        });

        // New dimensions label (bottom-right of new rect)
        let new_label = format!("New: {}×{}", self.new_width, self.new_height);
        frame.fill_text(Text {
            content: new_label,
            position: iced::Point::new(new_x, new_y + new_screen_height + 5.0),
            color: theme::resize_overlay_color(),
            size: font_size.into(),
            ..Text::default()
        });

        vec![frame.into_geometry()]
    }
}
