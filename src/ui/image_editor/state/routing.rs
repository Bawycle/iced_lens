// SPDX-License-Identifier: MPL-2.0
//! Message routing helpers that keep the editor facade slim.

use crate::ui::image_editor::{
    CanvasMessage, EditorTool, Event, ImageSource, SidebarMessage, State, ToolbarMessage,
};
use iced::widget::scrollable::AbsoluteOffset;
use iced::{self, keyboard, mouse, Point};

impl State {
    pub(crate) fn handle_toolbar_message(&mut self, message: ToolbarMessage) -> Event {
        match message {
            ToolbarMessage::BackToViewer => self.toolbar_back_to_viewer(),
        }
    }

    pub(crate) fn handle_sidebar_message(&mut self, message: SidebarMessage) -> Event {
        match message {
            SidebarMessage::ToggleSidebar => {
                self.sidebar_expanded = !self.sidebar_expanded;
                Event::None
            }
            SidebarMessage::SelectTool(tool) => {
                if self.active_tool == Some(tool) {
                    self.commit_active_tool_changes();
                    self.active_tool = None;
                    self.preview_image = None;
                    match tool {
                        EditorTool::Crop => self.teardown_crop_tool(),
                        EditorTool::Resize => self.hide_resize_overlay(),
                        EditorTool::Adjust => self.teardown_adjustment_tool(),
                        EditorTool::Deblur => self.teardown_deblur_tool(),
                        EditorTool::Rotate => {}
                    }
                } else {
                    self.commit_active_tool_changes();
                    if self.active_tool == Some(EditorTool::Crop) {
                        self.hide_crop_overlay();
                    }
                    if self.active_tool == Some(EditorTool::Resize) {
                        self.hide_resize_overlay();
                    }
                    if self.active_tool == Some(EditorTool::Adjust) {
                        self.teardown_adjustment_tool();
                    }
                    if self.active_tool == Some(EditorTool::Deblur) {
                        self.teardown_deblur_tool();
                    }
                    self.active_tool = Some(tool);
                    self.preview_image = None;

                    match tool {
                        EditorTool::Crop => self.prepare_crop_tool(),
                        EditorTool::Resize => {
                            // Option A2: No overlay - preview shows directly on canvas
                        }
                        EditorTool::Adjust => self.prepare_adjustment_tool(),
                        EditorTool::Deblur => self.prepare_deblur_tool(),
                        EditorTool::Rotate => {}
                    }
                }
                Event::None
            }
            SidebarMessage::RotateLeft => {
                self.commit_active_tool_changes();
                self.sidebar_rotate_left();
                Event::None
            }
            SidebarMessage::RotateRight => {
                self.commit_active_tool_changes();
                self.sidebar_rotate_right();
                Event::None
            }
            SidebarMessage::FlipHorizontal => {
                self.commit_active_tool_changes();
                self.sidebar_flip_horizontal();
                Event::None
            }
            SidebarMessage::FlipVertical => {
                self.commit_active_tool_changes();
                self.sidebar_flip_vertical();
                Event::None
            }
            SidebarMessage::SetCropRatio(ratio) => {
                self.set_crop_ratio_from_sidebar(ratio);
                Event::None
            }
            SidebarMessage::ApplyCrop => {
                self.apply_crop_from_sidebar();
                Event::None
            }
            SidebarMessage::ScaleChanged(percent) => {
                self.sidebar_scale_changed(percent);
                Event::None
            }
            SidebarMessage::WidthInputChanged(value) => {
                self.sidebar_width_input_changed(value);
                Event::None
            }
            SidebarMessage::HeightInputChanged(value) => {
                self.sidebar_height_input_changed(value);
                Event::None
            }
            SidebarMessage::ToggleLockAspect => {
                self.sidebar_toggle_lock();
                Event::None
            }
            SidebarMessage::ApplyResizePreset(percent) => {
                self.sidebar_scale_changed(percent);
                Event::None
            }
            SidebarMessage::ApplyResize => {
                self.sidebar_apply_resize();
                Event::None
            }
            SidebarMessage::BrightnessChanged(value) => {
                self.sidebar_brightness_changed(value);
                Event::None
            }
            SidebarMessage::ContrastChanged(value) => {
                self.sidebar_contrast_changed(value);
                Event::None
            }
            SidebarMessage::ApplyAdjustments => {
                self.sidebar_apply_adjustments();
                Event::None
            }
            SidebarMessage::ResetAdjustments => {
                self.sidebar_reset_adjustments();
                Event::None
            }
            SidebarMessage::ApplyDeblur => {
                self.sidebar_apply_deblur();
                Event::DeblurRequested
            }
            SidebarMessage::CancelDeblur => {
                self.sidebar_cancel_deblur();
                Event::DeblurCancelRequested
            }
            SidebarMessage::Undo => {
                self.commit_active_tool_changes();
                self.sidebar_undo();
                Event::None
            }
            SidebarMessage::Redo => {
                self.commit_active_tool_changes();
                self.sidebar_redo();
                Event::None
            }
            SidebarMessage::NavigateNext => self.sidebar_navigate_next(),
            SidebarMessage::NavigatePrevious => self.sidebar_navigate_previous(),
            SidebarMessage::Save => self.sidebar_save(),
            SidebarMessage::SaveAs => self.sidebar_save_as(),
            SidebarMessage::Cancel => self.sidebar_cancel(),
            SidebarMessage::SetExportFormat(format) => {
                self.set_export_format(format);
                Event::None
            }
        }
    }

    pub(crate) fn handle_canvas_message(&mut self, message: CanvasMessage) -> Event {
        match message {
            CanvasMessage::CursorMoved { position } => {
                self.cursor_position = Some(position);
                Event::None
            }
            CanvasMessage::CursorLeft => {
                self.cursor_position = None;
                Event::None
            }
            _ => self.handle_crop_canvas_message(message),
        }
    }

    pub(crate) fn handle_raw_event(&mut self, event: iced::Event) -> Event {
        match event {
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) => {
                if self.has_unsaved_changes() {
                    self.discard_changes();
                    Event::None
                } else {
                    Event::ExitEditor
                }
            }
            iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if modifiers.command() =>
            {
                match key {
                    keyboard::Key::Character(ref c) if c.as_str() == "s" => {
                        // Ctrl+S only works for file mode, not captured frames
                        if let ImageSource::File(path) = &self.image_source {
                            if self.has_unsaved_changes() {
                                return Event::SaveRequested {
                                    path: path.clone(),
                                    overwrite: true,
                                };
                            }
                        }
                        Event::None
                    }
                    keyboard::Key::Character(ref c) if c.as_str() == "z" => {
                        self.commit_active_tool_changes();
                        self.sidebar_undo();
                        Event::None
                    }
                    keyboard::Key::Character(ref c) if c.as_str() == "y" => {
                        self.commit_active_tool_changes();
                        self.sidebar_redo();
                        Event::None
                    }
                    _ => Event::None,
                }
            }
            iced::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                self.handle_wheel_zoom(delta);
                Event::None
            }
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) = self.cursor_position {
                    self.handle_mouse_button_pressed(position);
                }
                Event::None
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                self.handle_mouse_button_released();
                Event::None
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                self.cursor_position = Some(position);
                if self.drag.is_dragging {
                    return self.handle_cursor_moved_during_drag(position);
                }
                Event::None
            }
            iced::Event::Mouse(mouse::Event::CursorLeft) => {
                self.cursor_position = None;
                self.drag.stop();
                Event::None
            }
            _ => Event::None,
        }
    }

    /// Handles wheel scroll for zooming when cursor is over the canvas.
    fn handle_wheel_zoom(&mut self, delta: mouse::ScrollDelta) {
        // Only zoom if cursor is over the canvas area
        if !self.is_cursor_over_canvas() {
            return;
        }

        let steps = scroll_steps(&delta);
        if steps.abs() < f32::EPSILON {
            return;
        }

        let new_zoom = self.zoom.zoom_percent + steps * self.zoom.zoom_step_percent;
        self.zoom.apply_manual_zoom(new_zoom);
    }

    /// Checks if the cursor is currently positioned over the canvas area.
    fn is_cursor_over_canvas(&self) -> bool {
        let cursor = match self.cursor_position {
            Some(pos) => pos,
            None => return false,
        };

        let bounds = match self.viewport.bounds {
            Some(bounds) => bounds,
            None => return false,
        };

        // Check if cursor is within the canvas viewport bounds
        cursor.x >= bounds.x
            && cursor.x <= bounds.x + bounds.width
            && cursor.y >= bounds.y
            && cursor.y <= bounds.y + bounds.height
    }

    /// Handles mouse button press for starting pan drag.
    fn handle_mouse_button_pressed(&mut self, position: Point) {
        // Don't start pan if not over canvas
        if !self.is_cursor_over_canvas() {
            return;
        }

        // Don't start pan if crop tool is active and interacting with overlay
        if self.active_tool == Some(EditorTool::Crop) && self.crop_state.overlay.visible {
            // Crop overlay handles its own mouse events
            return;
        }

        // Start drag for panning
        self.drag.start(position, self.viewport.offset);
    }

    /// Handles mouse button release to stop pan drag.
    fn handle_mouse_button_released(&mut self) {
        self.drag.stop();
    }

    /// Updates the viewport when dragging to pan the image.
    fn handle_cursor_moved_during_drag(&mut self, position: Point) -> Event {
        let proposed_offset = match self.drag.calculate_offset(position) {
            Some(offset) => offset,
            None => return Event::None,
        };

        // Get viewport and scaled image size to clamp offset
        let viewport_bounds = match self.viewport.bounds {
            Some(bounds) => bounds,
            None => {
                self.viewport.offset = proposed_offset;
                return Event::None;
            }
        };

        let zoom_scale = self.zoom.zoom_percent / 100.0;
        let scaled_width = self.current_image.width as f32 * zoom_scale;
        let scaled_height = self.current_image.height as f32 * zoom_scale;

        // Calculate maximum offsets (how far we can scroll)
        let max_offset_x = (scaled_width - viewport_bounds.width).max(0.0);
        let max_offset_y = (scaled_height - viewport_bounds.height).max(0.0);

        // Clamp offset to valid range
        let clamped_offset = AbsoluteOffset {
            x: if max_offset_x > 0.0 {
                proposed_offset.x.clamp(0.0, max_offset_x)
            } else {
                0.0
            },
            y: if max_offset_y > 0.0 {
                proposed_offset.y.clamp(0.0, max_offset_y)
            } else {
                0.0
            },
        };

        self.viewport.offset = clamped_offset;

        // Calculate relative offset for scroll
        let relative_x = if max_offset_x > 0.0 {
            clamped_offset.x / max_offset_x
        } else {
            0.0
        };

        let relative_y = if max_offset_y > 0.0 {
            clamped_offset.y / max_offset_y
        } else {
            0.0
        };

        Event::ScrollTo {
            x: relative_x,
            y: relative_y,
        }
    }

    /// Check if dragging is active (for cursor display).
    pub fn is_dragging(&self) -> bool {
        self.drag.is_dragging
    }
}

/// Normalizes mouse wheel units (lines vs. pixels) into abstract step values
/// so zooming feels consistent across platforms.
fn scroll_steps(delta: &mouse::ScrollDelta) -> f32 {
    match delta {
        mouse::ScrollDelta::Lines { y, .. } => *y,
        mouse::ScrollDelta::Pixels { y, .. } => *y / 120.0,
    }
}
