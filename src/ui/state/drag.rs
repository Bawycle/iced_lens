// SPDX-License-Identifier: MPL-2.0
//! Drag state management
//!
//! Handles grab-and-drag interaction state for panning through images.

use iced::widget::scrollable::AbsoluteOffset;
use iced::Point;

/// Manages grab-and-drag state
#[derive(Debug, Clone, Default)]
pub struct DragState {
    /// Whether a drag operation is currently active
    pub is_dragging: bool,

    /// Position where the drag started
    pub start_position: Option<Point>,

    /// Viewport offset when the drag started
    pub start_offset: Option<AbsoluteOffset>,
}

impl DragState {
    /// Starts a drag operation
    pub fn start(&mut self, position: Point, offset: AbsoluteOffset) {
        self.is_dragging = true;
        self.start_position = Some(position);
        self.start_offset = Some(offset);
    }

    /// Stops the drag operation
    pub fn stop(&mut self) {
        self.is_dragging = false;
        self.start_position = None;
        self.start_offset = None;
    }

    /// Calculates the new offset based on cursor movement during drag
    #[must_use] 
    pub fn calculate_offset(&self, current_position: Point) -> Option<AbsoluteOffset> {
        if !self.is_dragging {
            return None;
        }

        let start_pos = self.start_position?;
        let start_offset = self.start_offset?;

        // Calculate delta: how much the cursor has moved
        let delta_x = current_position.x - start_pos.x;
        let delta_y = current_position.y - start_pos.y;

        // Calculate new viewport offset (inverse direction: moving cursor right scrolls content left)
        Some(AbsoluteOffset {
            x: (start_offset.x - delta_x).max(0.0),
            y: (start_offset.y - delta_y).max(0.0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_drag_state_is_not_dragging() {
        let state = DragState::default();
        assert!(!state.is_dragging);
        assert!(state.start_position.is_none());
        assert!(state.start_offset.is_none());
    }

    #[test]
    fn start_drag_sets_state() {
        let mut state = DragState::default();
        state.start(Point::new(100.0, 50.0), AbsoluteOffset { x: 20.0, y: 10.0 });

        assert!(state.is_dragging);
        assert_eq!(state.start_position, Some(Point::new(100.0, 50.0)));
        assert_eq!(
            state.start_offset,
            Some(AbsoluteOffset { x: 20.0, y: 10.0 })
        );
    }

    #[test]
    fn stop_drag_clears_state() {
        let mut state = DragState::default();
        state.start(Point::new(100.0, 50.0), AbsoluteOffset { x: 20.0, y: 10.0 });
        state.stop();

        assert!(!state.is_dragging);
        assert!(state.start_position.is_none());
        assert!(state.start_offset.is_none());
    }

    #[test]
    fn calculate_offset_returns_none_when_not_dragging() {
        let state = DragState::default();
        let result = state.calculate_offset(Point::new(100.0, 50.0));
        assert!(result.is_none());
    }

    #[test]
    fn calculate_offset_works_correctly() {
        let mut state = DragState::default();
        state.start(
            Point::new(200.0, 150.0),
            AbsoluteOffset { x: 50.0, y: 30.0 },
        );

        // Move cursor to (180, 130) - moved left/up by 20 pixels
        let new_offset = state.calculate_offset(Point::new(180.0, 130.0));

        // Cursor moved left (-20), so offset should increase (+20)
        // New offset: 50 - (-20) = 70, 30 - (-20) = 50
        assert_eq!(new_offset, Some(AbsoluteOffset { x: 70.0, y: 50.0 }));
    }
}
