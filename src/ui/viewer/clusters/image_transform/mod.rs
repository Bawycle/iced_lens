// SPDX-License-Identifier: MPL-2.0
//! Image transformation cluster - zoom, pan, rotation managed together.
//!
//! This cluster groups related image manipulation operations that have
//! strong internal coupling (e.g., double-click affects both zoom and pan).
//!
//! ## Composition
//!
//! - `ZoomState`: Integrated directly (not via wrapper)
//! - `drag::State`: Reused from subcomponents (cursor tracking, double-click)
//! - `rotation::State`: Reused from subcomponents (temporary rotation)

use crate::media::ImageData;
use crate::ui::state::ZoomState;
use crate::ui::viewer::subcomponents::{drag, rotation};
use iced::widget::scrollable::AbsoluteOffset;
use iced::Point;

/// Image transformation cluster state.
///
/// Combines zoom, pan (drag), and rotation into a cohesive unit.
/// Internal interactions (e.g., double-click toggling fit-to-window and resetting pan)
/// are handled within the cluster, not by the orchestrator.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// Zoom state - integrated directly (not a wrapper).
    pub zoom: ZoomState,

    /// Drag/pan state with cursor tracking and double-click detection.
    pub drag: drag::State,

    /// Temporary rotation state (resets on navigation).
    pub rotation: rotation::State,
}

/// Messages for the image transformation cluster.
#[derive(Debug, Clone)]
pub enum Message {
    // ═══════════════════════════════════════════════════════════════════════
    // ZOOM MESSAGES
    // ═══════════════════════════════════════════════════════════════════════
    /// Zoom in by one step.
    ZoomIn,
    /// Zoom out by one step.
    ZoomOut,
    /// Reset zoom to default (100%).
    ZoomReset,
    /// Set fit-to-window mode.
    SetFitToWindow(bool),
    /// Zoom input text changed.
    ZoomInputChanged(String),
    /// Zoom input submitted (Enter pressed).
    ZoomInputSubmitted,
    /// Zoom via mouse wheel with cursor position for zoom-towards-cursor.
    ZoomWheel {
        /// Zoom delta (positive = zoom in, negative = zoom out).
        delta: f32,
        /// Cursor position for zoom-towards-cursor calculation.
        cursor_pos: Point,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // DRAG/PAN MESSAGES - delegated to drag sub-component
    // ═══════════════════════════════════════════════════════════════════════
    /// Start drag operation.
    StartDrag {
        position: Point,
        viewport_offset: AbsoluteOffset,
    },
    /// Update drag position.
    UpdateDrag(Point),
    /// End drag operation.
    EndDrag,
    /// Mouse moved (for cursor tracking).
    MouseMoved(Point),
    /// Click detected (for double-click detection).
    Click(Point),

    // ═══════════════════════════════════════════════════════════════════════
    // ROTATION MESSAGES - delegated to rotation sub-component
    // ═══════════════════════════════════════════════════════════════════════
    /// Rotate 90° clockwise.
    RotateClockwise,
    /// Rotate 90° counter-clockwise.
    RotateCounterClockwise,
    /// Reset rotation to 0° (called when media changes).
    ResetRotation,

    // ═══════════════════════════════════════════════════════════════════════
    // CROSS-CUTTING MESSAGES - handled within the cluster
    // ═══════════════════════════════════════════════════════════════════════
    /// Reset all transformations for new media.
    /// Called when a new image/video is loaded.
    ResetForNewMedia,
}

/// Effects produced by image transformation operations.
#[derive(Debug, Clone)]
pub enum Effect {
    /// No effect.
    None,
    /// Zoom level changed - view needs refresh.
    ZoomChanged,
    /// Fit-to-window mode changed.
    FitToWindowChanged(bool),
    /// Rotation changed - view needs refresh and cache rebuild.
    RotationChanged,
    /// New viewport offset to apply (from drag).
    SetViewportOffset(AbsoluteOffset),
    /// Preferences should be persisted (zoom settings changed).
    PersistPreferences,
    /// Double-click detected - orchestrator decides action (currently: toggle fullscreen).
    DoubleClick,
}

impl State {
    /// Handle a cluster message.
    ///
    /// This method handles all zoom, drag, and rotation messages, including
    /// cross-cutting interactions like double-click toggling fit-to-window.
    #[allow(clippy::needless_pass_by_value)]
    pub fn handle(&mut self, msg: Message) -> Effect {
        match msg {
            // ═══════════════════════════════════════════════════════════════
            // ZOOM HANDLERS
            // ═══════════════════════════════════════════════════════════════
            Message::ZoomIn => {
                self.zoom.zoom_in();
                Effect::ZoomChanged
            }
            Message::ZoomOut => {
                self.zoom.zoom_out();
                Effect::ZoomChanged
            }
            Message::ZoomReset => {
                self.zoom.reset_zoom();
                Effect::ZoomChanged
            }
            Message::SetFitToWindow(enabled) => {
                if enabled {
                    self.zoom.enable_fit_to_window();
                } else {
                    self.zoom.disable_fit_to_window();
                }
                Effect::FitToWindowChanged(enabled)
            }
            Message::ZoomInputChanged(input) => {
                self.zoom.on_zoom_input_changed(input);
                Effect::None
            }
            Message::ZoomInputSubmitted => {
                if self.zoom.on_zoom_input_submitted() {
                    Effect::ZoomChanged
                } else {
                    Effect::None // Invalid input
                }
            }
            Message::ZoomWheel { delta, cursor_pos } => {
                // Zoom towards cursor position
                self.zoom_towards_cursor(delta, cursor_pos);
                Effect::ZoomChanged
            }

            // ═══════════════════════════════════════════════════════════════
            // DRAG HANDLERS - delegated to sub-component
            // ═══════════════════════════════════════════════════════════════
            Message::StartDrag {
                position,
                viewport_offset,
            } => {
                let effect = self.drag.handle(drag::Message::StartDrag {
                    position,
                    viewport_offset,
                });
                self.convert_drag_effect(effect)
            }
            Message::UpdateDrag(pos) => {
                let effect = self.drag.handle(drag::Message::UpdateDrag(pos));
                self.convert_drag_effect(effect)
            }
            Message::EndDrag => {
                let effect = self.drag.handle(drag::Message::EndDrag);
                self.convert_drag_effect(effect)
            }
            Message::MouseMoved(pos) => {
                let effect = self.drag.handle(drag::Message::MouseMoved(pos));
                self.convert_drag_effect(effect)
            }
            Message::Click(pos) => {
                let effect = self.drag.handle(drag::Message::Click(pos));
                // Double-click is returned to orchestrator which decides the action
                // (currently: toggle fullscreen)
                match effect {
                    drag::Effect::DoubleClick => Effect::DoubleClick,
                    _ => self.convert_drag_effect(effect),
                }
            }

            // ═══════════════════════════════════════════════════════════════
            // ROTATION HANDLERS - delegated to sub-component
            // ═══════════════════════════════════════════════════════════════
            Message::RotateClockwise => {
                let effect = self.rotation.handle(rotation::Message::RotateClockwise);
                self.convert_rotation_effect(effect)
            }
            Message::RotateCounterClockwise => {
                let effect = self
                    .rotation
                    .handle(rotation::Message::RotateCounterClockwise);
                self.convert_rotation_effect(effect)
            }
            Message::ResetRotation => {
                let effect = self.rotation.handle(rotation::Message::Reset);
                self.convert_rotation_effect(effect)
            }

            // ═══════════════════════════════════════════════════════════════
            // CROSS-CUTTING HANDLERS
            // ═══════════════════════════════════════════════════════════════
            Message::ResetForNewMedia => {
                // Reset zoom to fit-to-window
                self.zoom.enable_fit_to_window();
                // Reset rotation
                self.rotation.handle(rotation::Message::Reset);
                // Note: drag state doesn't need explicit reset
                Effect::FitToWindowChanged(true)
            }
        }
    }

    /// Zoom towards cursor position (mouse wheel zoom).
    fn zoom_towards_cursor(&mut self, delta: f32, _cursor_pos: Point) {
        // For now, simple zoom in/out without cursor tracking
        // TODO: Implement proper zoom-towards-cursor with viewport offset adjustment
        if delta > 0.0 {
            self.zoom.zoom_in();
        } else {
            self.zoom.zoom_out();
        }
    }

    /// Convert drag effect to cluster effect.
    fn convert_drag_effect(&self, effect: drag::Effect) -> Effect {
        match effect {
            drag::Effect::None => Effect::None,
            drag::Effect::DoubleClick => {
                // Should not reach here - handled in Click handler
                Effect::None
            }
            drag::Effect::SetViewportOffset(offset) => Effect::SetViewportOffset(offset),
        }
    }

    /// Convert rotation effect to cluster effect.
    fn convert_rotation_effect(&self, effect: rotation::Effect) -> Effect {
        match effect {
            rotation::Effect::None => Effect::None,
            rotation::Effect::RotationChanged => Effect::RotationChanged,
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ACCESSORS
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if a drag is currently in progress.
    #[must_use]
    pub fn is_dragging(&self) -> bool {
        self.drag.is_dragging()
    }

    /// Get the current cursor position (if known).
    #[must_use]
    pub fn cursor_position(&self) -> Option<Point> {
        self.drag.cursor_position()
    }

    /// Get the current zoom percentage.
    #[must_use]
    pub fn zoom_percent(&self) -> f32 {
        self.zoom.zoom_percent
    }

    /// Check if fit-to-window mode is enabled.
    #[must_use]
    pub fn fit_to_window(&self) -> bool {
        self.zoom.fit_to_window
    }

    /// Get the zoom input value.
    #[must_use]
    pub fn zoom_input(&self) -> &str {
        self.zoom.zoom_input_value()
    }

    /// Get the current rotation angle.
    #[must_use]
    pub fn rotation_angle(&self) -> crate::ui::state::RotationAngle {
        self.rotation.angle()
    }

    /// Check if the image is currently rotated.
    #[must_use]
    pub fn is_rotated(&self) -> bool {
        self.rotation.is_rotated()
    }

    /// Get cached rotated image if available and angle matches.
    #[must_use]
    pub fn cached_rotated_image(&self) -> Option<&ImageData> {
        self.rotation.cached_image()
    }

    /// Set the rotation cache with a rotated image.
    pub fn set_rotation_cache(&mut self, image: ImageData) {
        self.rotation.set_cache(image);
    }

    /// Clear the rotation cache.
    pub fn clear_rotation_cache(&mut self) {
        self.rotation.clear_cache();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_in_increases_zoom() {
        let mut state = State::default();
        let initial = state.zoom_percent();
        state.handle(Message::ZoomIn);
        assert!(state.zoom_percent() > initial);
    }

    #[test]
    fn zoom_out_decreases_zoom() {
        let mut state = State::default();
        state.handle(Message::ZoomIn);
        state.handle(Message::ZoomIn);
        let after_zoom_in = state.zoom_percent();
        state.handle(Message::ZoomOut);
        assert!(state.zoom_percent() < after_zoom_in);
    }

    #[test]
    fn double_click_returns_double_click_effect() {
        let mut state = State::default();

        // First click: no effect
        let effect = state.handle(Message::Click(Point::new(0.0, 0.0)));
        assert!(matches!(effect, Effect::None));

        // Second click (immediate): double-click detected
        let effect = state.handle(Message::Click(Point::new(0.0, 0.0)));
        assert!(matches!(effect, Effect::DoubleClick));
    }

    #[test]
    fn rotate_clockwise_changes_angle() {
        let mut state = State::default();
        assert!(!state.is_rotated());

        state.handle(Message::RotateClockwise);
        assert!(state.is_rotated());
        assert_eq!(state.rotation_angle().degrees(), 90);
    }

    #[test]
    fn reset_for_new_media_resets_all() {
        let mut state = State::default();

        // Make some changes
        state.handle(Message::ZoomIn);
        state.handle(Message::RotateClockwise);

        // Reset for new media
        let effect = state.handle(Message::ResetForNewMedia);

        assert!(matches!(effect, Effect::FitToWindowChanged(true)));
        assert!(state.fit_to_window());
        assert!(!state.is_rotated());
    }

    #[test]
    fn drag_start_and_stop() {
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
    fn mouse_moved_updates_cursor() {
        let mut state = State::default();
        assert!(state.cursor_position().is_none());

        state.handle(Message::MouseMoved(Point::new(50.0, 75.0)));
        assert_eq!(state.cursor_position(), Some(Point::new(50.0, 75.0)));
    }

    #[test]
    fn zoom_wheel_changes_zoom() {
        let mut state = State::default();
        state.zoom.disable_fit_to_window();
        let initial = state.zoom_percent();

        state.handle(Message::ZoomWheel {
            delta: 1.0,
            cursor_pos: Point::new(100.0, 100.0),
        });
        assert!(state.zoom_percent() > initial);
    }
}
