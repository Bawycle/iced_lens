// SPDX-License-Identifier: MPL-2.0
//! Viewport state management
//!
//! Handles the scrollable viewport state including bounds and scroll offset.

use iced::Rectangle;
use iced::widget::scrollable::AbsoluteOffset;

/// Manages viewport and scroll state
#[derive(Debug, Clone)]
pub struct ViewportState {
    /// Current scroll offset
    pub offset: AbsoluteOffset,

    /// Previous scroll offset (for delta tracking)
    pub previous_offset: AbsoluteOffset,

    /// Current viewport bounds
    pub bounds: Option<Rectangle>,
}

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            offset: AbsoluteOffset { x: 0.0, y: 0.0 },
            previous_offset: AbsoluteOffset { x: 0.0, y: 0.0 },
            bounds: None,
        }
    }
}

impl ViewportState {
    /// Updates the viewport state with new bounds and offset
    pub fn update(&mut self, bounds: Rectangle, offset: AbsoluteOffset) {
        self.previous_offset = self.offset;
        self.offset = offset;
        self.bounds = Some(bounds);
    }

    /// Calculates the scroll position as percentage (0-100%)
    pub fn scroll_position_percentage(&self, image_width: f32, image_height: f32) -> Option<(f32, f32)> {
        let viewport = self.bounds?;

        // If image is smaller than viewport, no scrolling needed
        if image_width <= viewport.width && image_height <= viewport.height {
            return None;
        }

        let max_offset_x = (image_width - viewport.width).max(0.0);
        let max_offset_y = (image_height - viewport.height).max(0.0);

        let percent_x = if max_offset_x > 0.0 {
            (self.offset.x / max_offset_x * 100.0).min(100.0)
        } else {
            0.0
        };

        let percent_y = if max_offset_y > 0.0 {
            (self.offset.y / max_offset_y * 100.0).min(100.0)
        } else {
            0.0
        };

        Some((percent_x, percent_y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::{Point, Size};

    #[test]
    fn default_viewport_has_zero_offset() {
        let state = ViewportState::default();
        assert_eq!(state.offset.x, 0.0);
        assert_eq!(state.offset.y, 0.0);
        assert!(state.bounds.is_none());
    }

    #[test]
    fn update_tracks_previous_offset() {
        let mut state = ViewportState::default();
        let bounds = Rectangle::new(Point::new(0.0, 0.0), Size::new(400.0, 300.0));

        state.update(bounds, AbsoluteOffset { x: 10.0, y: 5.0 });
        assert_eq!(state.previous_offset.x, 0.0);
        assert_eq!(state.offset.x, 10.0);

        state.update(bounds, AbsoluteOffset { x: 20.0, y: 15.0 });
        assert_eq!(state.previous_offset.x, 10.0);
        assert_eq!(state.offset.x, 20.0);
    }

    #[test]
    fn scroll_percentage_returns_none_when_image_fits() {
        let mut state = ViewportState::default();
        state.bounds = Some(Rectangle::new(Point::new(0.0, 0.0), Size::new(800.0, 600.0)));

        // Image smaller than viewport
        let result = state.scroll_position_percentage(400.0, 300.0);
        assert!(result.is_none());
    }

    #[test]
    fn scroll_percentage_calculates_correctly() {
        let mut state = ViewportState::default();
        state.bounds = Some(Rectangle::new(Point::new(0.0, 0.0), Size::new(400.0, 300.0)));
        state.offset = AbsoluteOffset { x: 200.0, y: 150.0 };

        // Image: 800x600, Viewport: 400x300
        // Max offset: 400x300
        // Current: 200x150 = 50% on both axes
        let result = state.scroll_position_percentage(800.0, 600.0);
        assert_eq!(result, Some((50.0, 50.0)));
    }
}
