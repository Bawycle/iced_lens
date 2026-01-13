// SPDX-License-Identifier: MPL-2.0
//! Viewport state management
//!
//! Handles the scrollable viewport state including bounds and scroll offset.

use iced::widget::scrollable::AbsoluteOffset;
use iced::Rectangle;

/// Manages viewport and scroll state
#[derive(Debug, Clone)]
pub struct ViewportState {
    /// Current scroll offset
    pub offset: AbsoluteOffset,

    /// Previous scroll offset (for delta tracking)
    pub previous_offset: AbsoluteOffset,

    /// Current viewport bounds
    pub bounds: Option<Rectangle>,

    /// Previous viewport bounds (for layout change detection)
    pub previous_bounds: Option<Rectangle>,
}

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            offset: AbsoluteOffset { x: 0.0, y: 0.0 },
            previous_offset: AbsoluteOffset { x: 0.0, y: 0.0 },
            bounds: None,
            previous_bounds: None,
        }
    }
}

/// Minimum bounds change (in pixels) to trigger recenter.
/// This threshold prevents recentering on small layout fluctuations (e.g., during video loading)
/// while still catching significant layout changes like sidebar toggle (~290px).
const SIGNIFICANT_BOUNDS_CHANGE_THRESHOLD: f32 = 100.0;

impl ViewportState {
    /// Resets the scroll offset to zero (for recentering after layout changes).
    pub fn reset_offset(&mut self) {
        self.previous_offset = self.offset;
        self.offset = AbsoluteOffset { x: 0.0, y: 0.0 };
    }

    /// Updates the viewport state with new bounds and offset.
    /// Returns true if the bounds size changed significantly (layout change detected).
    /// Small changes (< 100px) are ignored to avoid resetting during content loading.
    pub fn update(&mut self, bounds: Rectangle, offset: AbsoluteOffset) -> bool {
        self.previous_offset = self.offset;
        self.offset = offset;
        self.previous_bounds = self.bounds;

        let significant_bounds_changed = match self.previous_bounds {
            Some(prev) => {
                // Only trigger on significant size changes (e.g., sidebar toggle)
                // Ignore small fluctuations during content loading
                (prev.width - bounds.width).abs() > SIGNIFICANT_BOUNDS_CHANGE_THRESHOLD
                    || (prev.height - bounds.height).abs() > SIGNIFICANT_BOUNDS_CHANGE_THRESHOLD
            }
            None => false, // First update, no change
        };

        self.bounds = Some(bounds);
        significant_bounds_changed
    }

    /// Checks if content of given size fits within current viewport bounds.
    #[must_use]
    pub fn content_fits(&self, content_width: f32, content_height: f32) -> bool {
        match self.bounds {
            Some(bounds) => content_width <= bounds.width && content_height <= bounds.height,
            None => false,
        }
    }

    /// Calculates the scroll position as percentage (0-100%)
    #[must_use]
    pub fn scroll_position_percentage(
        &self,
        image_width: f32,
        image_height: f32,
    ) -> Option<(f32, f32)> {
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
    use crate::test_utils::assert_abs_diff_eq;
    use iced::{Point, Size};

    #[test]
    fn default_viewport_has_zero_offset() {
        let state = ViewportState::default();
        assert_abs_diff_eq!(state.offset.x, 0.0);
        assert_abs_diff_eq!(state.offset.y, 0.0);
        assert!(state.bounds.is_none());
    }

    #[test]
    fn update_tracks_previous_offset() {
        let mut state = ViewportState::default();
        let bounds = Rectangle::new(Point::new(0.0, 0.0), Size::new(400.0, 300.0));

        state.update(bounds, AbsoluteOffset { x: 10.0, y: 5.0 });
        assert_abs_diff_eq!(state.previous_offset.x, 0.0);
        assert_abs_diff_eq!(state.offset.x, 10.0);

        state.update(bounds, AbsoluteOffset { x: 20.0, y: 15.0 });
        assert_abs_diff_eq!(state.previous_offset.x, 10.0);
        assert_abs_diff_eq!(state.offset.x, 20.0);
    }

    #[test]
    fn scroll_percentage_returns_none_when_image_fits() {
        let state = ViewportState {
            bounds: Some(Rectangle::new(
                Point::new(0.0, 0.0),
                Size::new(800.0, 600.0),
            )),
            ..ViewportState::default()
        };

        // Image smaller than viewport
        let result = state.scroll_position_percentage(400.0, 300.0);
        assert!(result.is_none());
    }

    #[test]
    fn scroll_percentage_calculates_correctly() {
        let state = ViewportState {
            bounds: Some(Rectangle::new(
                Point::new(0.0, 0.0),
                Size::new(400.0, 300.0),
            )),
            offset: AbsoluteOffset { x: 200.0, y: 150.0 },
            ..ViewportState::default()
        };

        // Image: 800x600, Viewport: 400x300
        // Max offset: 400x300
        // Current: 200x150 = 50% on both axes
        let result = state.scroll_position_percentage(800.0, 600.0);
        assert_eq!(result, Some((50.0, 50.0)));
    }
}
