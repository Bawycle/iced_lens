// SPDX-License-Identifier: MPL-2.0
//! Message routing helpers that keep the editor facade slim.

use crate::ui::image_editor::{
    CanvasMessage, EditorTool, Event, ImageSource, SidebarMessage, State, ToolbarMessage,
};
use iced::{self, keyboard};

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
        self.handle_crop_canvas_message(message)
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
            _ => Event::None,
        }
    }
}
