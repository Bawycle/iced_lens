// SPDX-License-Identifier: MPL-2.0
//! Drag/pan sub-component with double-click detection.

use crate::ui::state::DragState;
use iced::widget::scrollable::AbsoluteOffset;
use iced::Point;
use std::time::{Duration, Instant};

/// Time threshold for double-click detection.
const DOUBLE_CLICK_THRESHOLD: Duration = Duration::from_millis(350);

/// Drag sub-component state.
/// Encapsulates `DragState` and adds cursor tracking and double-click detection.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// The underlying drag state (existing type).
    pub inner: DragState,
    /// Current cursor position within the viewer.
    pub cursor_position: Option<Point>,
    /// Last click timestamp for double-click detection.
    last_click: Option<Instant>,
}

/// Messages for the drag sub-component.
#[derive(Debug, Clone)]
pub enum Message {
    /// Start drag - requires current viewport offset from orchestrator.
    StartDrag {
        position: Point,
        viewport_offset: AbsoluteOffset,
    },
    /// Update drag position - returns new viewport offset.
    UpdateDrag(Point),
    /// End drag operation.
    EndDrag,
    /// Mouse moved (for cursor tracking).
    MouseMoved(Point),
    /// Click detected (for double-click).
    Click(Point),
}

/// Effects produced by drag operations.
#[derive(Debug, Clone)]
pub enum Effect {
    /// No effect.
    None,
    /// Double-click detected - orchestrator should toggle fit-to-window.
    DoubleClick,
    /// New viewport offset to apply.
    SetViewportOffset(AbsoluteOffset),
}

impl State {
    /// Handle a drag message.
    ///
    /// Note: Takes `Message` by value following Iced's `update(message: Message)` pattern.
    /// Clippy's `needless_pass_by_value` is suppressed because this is the standard
    /// TEA/Iced pattern where messages are moved into the handler.
    #[allow(clippy::needless_pass_by_value)]
    pub fn handle(&mut self, msg: Message) -> Effect {
        match msg {
            Message::StartDrag {
                position,
                viewport_offset,
            } => {
                self.inner.start(position, viewport_offset);
                Effect::None
            }
            Message::UpdateDrag(current_position) => {
                if let Some(new_offset) = self.inner.calculate_offset(current_position) {
                    Effect::SetViewportOffset(new_offset)
                } else {
                    Effect::None
                }
            }
            Message::EndDrag => {
                self.inner.stop();
                Effect::None
            }
            Message::MouseMoved(pos) => {
                self.cursor_position = Some(pos);
                Effect::None
            }
            Message::Click(pos) => {
                self.cursor_position = Some(pos);
                let now = Instant::now();

                let is_double_click = self
                    .last_click
                    .is_some_and(|t| now.duration_since(t) < DOUBLE_CLICK_THRESHOLD);

                self.last_click = Some(now);

                if is_double_click {
                    self.last_click = None; // Reset to avoid triple-click
                    Effect::DoubleClick
                } else {
                    Effect::None
                }
            }
        }
    }

    /// Check if a drag is currently in progress.
    #[must_use]
    pub fn is_dragging(&self) -> bool {
        self.inner.is_dragging
    }

    /// Get the current cursor position (if known).
    #[must_use]
    pub fn cursor_position(&self) -> Option<Point> {
        self.cursor_position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_and_stop_drag() {
        let mut state = State::default();
        assert!(!state.is_dragging());

        state.handle(Message::StartDrag {
            position: Point::new(100.0, 100.0),
            viewport_offset: AbsoluteOffset { x: 0.0, y: 0.0 },
        });
        assert!(state.is_dragging());

        state.handle(Message::EndDrag);
        assert!(!state.is_dragging());
    }

    #[test]
    fn mouse_moved_updates_cursor_position() {
        let mut state = State::default();
        assert!(state.cursor_position().is_none());

        state.handle(Message::MouseMoved(Point::new(50.0, 75.0)));
        assert_eq!(state.cursor_position(), Some(Point::new(50.0, 75.0)));
    }

    #[test]
    fn double_click_within_threshold() {
        let mut state = State::default();
        state.handle(Message::Click(Point::new(0.0, 0.0)));

        // Immediate second click should be detected as double-click
        let effect = state.handle(Message::Click(Point::new(0.0, 0.0)));
        assert!(matches!(effect, Effect::DoubleClick));
    }

    #[test]
    fn single_click_returns_none() {
        let mut state = State::default();
        let effect = state.handle(Message::Click(Point::new(0.0, 0.0)));
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn update_drag_returns_offset_when_dragging() {
        let mut state = State::default();
        state.handle(Message::StartDrag {
            position: Point::new(100.0, 100.0),
            viewport_offset: AbsoluteOffset { x: 50.0, y: 50.0 },
        });

        let effect = state.handle(Message::UpdateDrag(Point::new(120.0, 110.0)));
        assert!(matches!(effect, Effect::SetViewportOffset(_)));
    }

    #[test]
    fn update_drag_returns_none_when_not_dragging() {
        let mut state = State::default();
        let effect = state.handle(Message::UpdateDrag(Point::new(100.0, 100.0)));
        assert!(matches!(effect, Effect::None));
    }
}
