// SPDX-License-Identifier: MPL-2.0
//! Crop tool state and helpers.

use crate::image_handler::{transform, ImageData};
use crate::ui::editor::{CanvasMessage, Event, State, Transformation};
use iced::Rectangle;

/// Crop aspect ratio constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CropRatio {
    None, // No ratio selected
    Free,
    Square,        // 1:1
    Landscape,     // 16:9
    Portrait,      // 9:16
    Photo,         // 4:3
    PhotoPortrait, // 3:4
}

/// Position of a resize handle on the crop rectangle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandlePosition {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

/// Crop drag state for interactive overlay
#[derive(Debug, Clone, PartialEq)]
pub enum CropDragState {
    /// No active drag
    None,
    /// Dragging the entire rectangle
    DraggingRectangle {
        /// Starting rectangle position
        start_rect_x: u32,
        start_rect_y: u32,
        /// Starting cursor position (in image coordinates)
        start_cursor_x: f32,
        start_cursor_y: f32,
    },
    /// Dragging a resize handle
    DraggingHandle {
        /// Which handle is being dragged
        handle: HandlePosition,
        /// Starting rectangle dimensions
        start_rect: (u32, u32, u32, u32), // x, y, width, height
        /// Starting cursor position (in image coordinates)
        start_cursor_x: f32,
        start_cursor_y: f32,
    },
}

/// Interactive crop overlay state
#[derive(Debug, Clone, PartialEq)]
pub struct CropOverlay {
    /// Whether the overlay is currently visible
    pub visible: bool,
    /// Current drag operation, if any
    pub drag_state: CropDragState,
}

/// State for the crop tool.
#[derive(Debug, Clone, PartialEq)]
pub struct CropState {
    /// Crop rectangle in image coordinates
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    /// Selected aspect ratio constraint
    pub ratio: CropRatio,
    /// Interactive overlay state
    pub overlay: CropOverlay,
}

impl CropState {
    pub fn from_image(image: &ImageData) -> Self {
        let crop_width = (image.width as f32 * 0.75).round() as u32;
        let crop_height = (image.height as f32 * 0.75).round() as u32;
        let crop_x = (image.width.saturating_sub(crop_width)) / 2;
        let crop_y = (image.height.saturating_sub(crop_height)) / 2;

        Self {
            x: crop_x,
            y: crop_y,
            width: crop_width.max(1),
            height: crop_height.max(1),
            ratio: CropRatio::Free,
            overlay: CropOverlay {
                visible: false,
                drag_state: CropDragState::None,
            },
        }
    }
}

impl State {
    pub(crate) fn handle_crop_canvas_message(&mut self, message: CanvasMessage) -> Event {
        match message {
            CanvasMessage::CropOverlayMouseDown { x, y } => {
                self.handle_crop_overlay_mouse_down(x, y);
                Event::None
            }
            CanvasMessage::CropOverlayMouseMove { x, y } => {
                self.handle_crop_overlay_mouse_move(x, y);
                Event::None
            }
            CanvasMessage::CropOverlayMouseUp => {
                self.crop_state.overlay.drag_state = CropDragState::None;
                Event::None
            }
        }
    }

    pub(crate) fn prepare_crop_tool(&mut self) {
        self.crop_base_image = Some(self.working_image.clone());
        self.crop_base_width = self.current_image.width;
        self.crop_base_height = self.current_image.height;
        self.crop_state.x = 0;
        self.crop_state.y = 0;
        self.crop_state.width = self.current_image.width;
        self.crop_state.height = self.current_image.height;
        self.crop_state.ratio = CropRatio::None;
        self.hide_crop_overlay();
    }

    pub(crate) fn hide_crop_overlay(&mut self) {
        self.crop_state.overlay.visible = false;
        self.crop_state.overlay.drag_state = CropDragState::None;
    }

    pub(crate) fn teardown_crop_tool(&mut self) {
        self.crop_modified = false;
        self.crop_base_image = None;
        self.hide_crop_overlay();
    }

    pub(crate) fn set_crop_ratio_from_sidebar(&mut self, ratio: CropRatio) {
        self.crop_state.ratio = ratio;
        self.adjust_crop_to_ratio(ratio);
        self.crop_state.overlay.visible = true;
        self.crop_modified = true;
    }

    pub(crate) fn apply_crop_from_sidebar(&mut self) {
        if self.crop_state.overlay.visible {
            self.finalize_crop_overlay();
        }
    }

    pub(crate) fn adjust_crop_to_ratio(&mut self, ratio: CropRatio) {
        // Use base image dimensions (image when crop tool was opened), not current image
        let img_width = self.crop_base_width as f32;
        let img_height = self.crop_base_height as f32;

        let (new_width, new_height) = match ratio {
            CropRatio::None | CropRatio::Free => {
                // No adjustment needed
                return;
            }
            CropRatio::Square => {
                // 1:1 - make square, use smaller dimension
                let size = img_width.min(img_height);
                (size, size)
            }
            CropRatio::Landscape => {
                // 16:9
                let height = img_width * 9.0 / 16.0;
                if height <= img_height {
                    (img_width, height)
                } else {
                    let width = img_height * 16.0 / 9.0;
                    (width, img_height)
                }
            }
            CropRatio::Portrait => {
                // 9:16
                let width = img_height * 9.0 / 16.0;
                if width <= img_width {
                    (width, img_height)
                } else {
                    let height = img_width * 16.0 / 9.0;
                    (img_width, height)
                }
            }
            CropRatio::Photo => {
                // 4:3
                let height = img_width * 3.0 / 4.0;
                if height <= img_height {
                    (img_width, height)
                } else {
                    let width = img_height * 4.0 / 3.0;
                    (width, img_height)
                }
            }
            CropRatio::PhotoPortrait => {
                // 3:4
                let width = img_height * 3.0 / 4.0;
                if width <= img_width {
                    (width, img_height)
                } else {
                    let height = img_width * 4.0 / 3.0;
                    (img_width, height)
                }
            }
        };

        let new_width = new_width.round() as u32;
        let new_height = new_height.round() as u32;

        // Center the crop area (using base image dimensions)
        self.crop_state.width = new_width;
        self.crop_state.height = new_height;
        self.crop_state.x = (self.crop_base_width - new_width) / 2;
        self.crop_state.y = (self.crop_base_height - new_height) / 2;
    }

    pub(crate) fn apply_crop_from_base(&mut self) {
        // Apply crop from the base image (image when crop tool was opened)
        let Some(ref base_image) = self.crop_base_image else {
            eprintln!("No base image available for crop");
            return;
        };

        let x = self.crop_state.x;
        let y = self.crop_state.y;
        let width = self.crop_state.width;
        let height = self.crop_state.height;

        // Validate crop bounds
        if width == 0 || height == 0 || x >= self.crop_base_width || y >= self.crop_base_height {
            eprintln!("Invalid crop bounds: ({}, {}, {}×{})", x, y, width, height);
            return;
        }

        // Apply crop transformation from base image
        if let Some(cropped) = transform::crop(base_image, x, y, width, height) {
            match transform::dynamic_to_image_data(&cropped) {
                Ok(image_data) => {
                    self.working_image = cropped;
                    self.current_image = image_data;
                    self.sync_resize_state_dimensions();

                    // Record transformation for undo/redo
                    self.record_transformation(Transformation::Crop {
                        rect: Rectangle {
                            x: x as f32,
                            y: y as f32,
                            width: width as f32,
                            height: height as f32,
                        },
                    });

                    // Note: Do NOT update crop_base_image here!
                    // All crops within the same session should be relative to the base image
                    // captured when the Crop tool was opened. The base is only updated when
                    // the user closes and reopens the Crop tool.
                }
                Err(err) => {
                    eprintln!("Failed to convert cropped image: {err:?}");
                }
            }
        } else {
            eprintln!("Crop operation returned None");
        }
    }

    pub(crate) fn finalize_crop_overlay(&mut self) {
        if !self.crop_state.overlay.visible {
            return;
        }

        self.apply_crop_from_base();
        self.crop_state.overlay.visible = false;
        self.crop_state.overlay.drag_state = CropDragState::None;
        self.crop_modified = false;
        self.crop_state.ratio = CropRatio::None;
        self.crop_state.x = 0;
        self.crop_state.y = 0;
        self.crop_state.width = self.current_image.width;
        self.crop_state.height = self.current_image.height;
        self.crop_base_image = Some(self.working_image.clone());
        self.crop_base_width = self.current_image.width;
        self.crop_base_height = self.current_image.height;
    }

    /// Handle mouse down on crop overlay to start dragging
    pub(crate) fn handle_crop_overlay_mouse_down(&mut self, x: f32, y: f32) {
        // Check if clicking on a handle
        if let Some(handle) = self.get_handle_at_position(x, y) {
            self.crop_state.overlay.drag_state = CropDragState::DraggingHandle {
                handle,
                start_rect: (
                    self.crop_state.x,
                    self.crop_state.y,
                    self.crop_state.width,
                    self.crop_state.height,
                ),
                start_cursor_x: x,
                start_cursor_y: y,
            };
        } else if self.is_point_in_crop_rect(x, y) {
            // Start dragging the entire rectangle
            self.crop_state.overlay.drag_state = CropDragState::DraggingRectangle {
                start_rect_x: self.crop_state.x,
                start_rect_y: self.crop_state.y,
                start_cursor_x: x,
                start_cursor_y: y,
            };
        }
    }

    /// Handle mouse move on crop overlay to update drag
    pub(crate) fn handle_crop_overlay_mouse_move(&mut self, x: f32, y: f32) {
        match self.crop_state.overlay.drag_state.clone() {
            CropDragState::DraggingRectangle {
                start_rect_x,
                start_rect_y,
                start_cursor_x,
                start_cursor_y,
            } => {
                // Calculate delta
                let delta_x = x - start_cursor_x;
                let delta_y = y - start_cursor_y;

                // Update position with bounds checking
                let new_x = (start_rect_x as f32 + delta_x)
                    .max(0.0)
                    .min((self.crop_base_width - self.crop_state.width) as f32);
                let new_y = (start_rect_y as f32 + delta_y)
                    .max(0.0)
                    .min((self.crop_base_height - self.crop_state.height) as f32);

                self.crop_state.x = new_x as u32;
                self.crop_state.y = new_y as u32;
                self.crop_modified = true;
            }
            CropDragState::DraggingHandle {
                handle,
                start_rect,
                start_cursor_x,
                start_cursor_y,
            } => {
                // Calculate delta
                let delta_x = x - start_cursor_x;
                let delta_y = y - start_cursor_y;

                // Update crop dimensions based on which handle is being dragged
                self.update_crop_from_handle_drag(handle, start_rect, delta_x, delta_y);
                self.crop_modified = true;
                // Switch to Free ratio when manually resizing
                self.crop_state.ratio = CropRatio::Free;
            }
            CropDragState::None => {}
        }
    }

    fn is_point_in_crop_rect(&self, x: f32, y: f32) -> bool {
        let rect_x = self.crop_state.x as f32;
        let rect_y = self.crop_state.y as f32;
        let rect_w = self.crop_state.width as f32;
        let rect_h = self.crop_state.height as f32;

        x >= rect_x && x <= rect_x + rect_w && y >= rect_y && y <= rect_y + rect_h
    }

    fn get_handle_at_position(&self, x: f32, y: f32) -> Option<HandlePosition> {
        const HANDLE_SIZE: f32 = 20.0; // Handle click area size (extended for better grabbability)

        let rect_x = self.crop_state.x as f32;
        let rect_y = self.crop_state.y as f32;
        let rect_w = self.crop_state.width as f32;
        let rect_h = self.crop_state.height as f32;

        // Define handle positions
        let handles = [
            (HandlePosition::TopLeft, rect_x, rect_y),
            (HandlePosition::Top, rect_x + rect_w / 2.0, rect_y),
            (HandlePosition::TopRight, rect_x + rect_w, rect_y),
            (
                HandlePosition::Right,
                rect_x + rect_w,
                rect_y + rect_h / 2.0,
            ),
            (
                HandlePosition::BottomRight,
                rect_x + rect_w,
                rect_y + rect_h,
            ),
            (
                HandlePosition::Bottom,
                rect_x + rect_w / 2.0,
                rect_y + rect_h,
            ),
            (HandlePosition::BottomLeft, rect_x, rect_y + rect_h),
            (HandlePosition::Left, rect_x, rect_y + rect_h / 2.0),
        ];

        // Check each handle
        for (handle, hx, hy) in handles {
            if (x - hx).abs() <= HANDLE_SIZE && (y - hy).abs() <= HANDLE_SIZE {
                return Some(handle);
            }
        }

        None
    }

    fn update_crop_from_handle_drag(
        &mut self,
        handle: HandlePosition,
        start_rect: (u32, u32, u32, u32),
        delta_x: f32,
        delta_y: f32,
    ) {
        let (start_x, start_y, start_w, start_h) = start_rect;

        match handle {
            HandlePosition::TopLeft => {
                let new_x = (start_x as f32 + delta_x)
                    .max(0.0)
                    .min(start_x as f32 + start_w as f32 - 10.0);
                let new_y = (start_y as f32 + delta_y)
                    .max(0.0)
                    .min(start_y as f32 + start_h as f32 - 10.0);
                self.crop_state.width = (start_x + start_w) - new_x as u32;
                self.crop_state.height = (start_y + start_h) - new_y as u32;
                self.crop_state.x = new_x as u32;
                self.crop_state.y = new_y as u32;
            }
            HandlePosition::Top => {
                let new_y = (start_y as f32 + delta_y)
                    .max(0.0)
                    .min(start_y as f32 + start_h as f32 - 10.0);
                self.crop_state.height = (start_y + start_h) - new_y as u32;
                self.crop_state.y = new_y as u32;
            }
            HandlePosition::TopRight => {
                let new_y = (start_y as f32 + delta_y)
                    .max(0.0)
                    .min(start_y as f32 + start_h as f32 - 10.0);
                let new_w = (start_w as f32 + delta_x)
                    .max(10.0)
                    .min((self.crop_base_width - start_x) as f32);
                self.crop_state.width = new_w as u32;
                self.crop_state.height = (start_y + start_h) - new_y as u32;
                self.crop_state.y = new_y as u32;
            }
            HandlePosition::Right => {
                let new_w = (start_w as f32 + delta_x)
                    .max(10.0)
                    .min((self.crop_base_width - start_x) as f32);
                self.crop_state.width = new_w as u32;
            }
            HandlePosition::BottomRight => {
                let new_w = (start_w as f32 + delta_x)
                    .max(10.0)
                    .min((self.crop_base_width - start_x) as f32);
                let new_h = (start_h as f32 + delta_y)
                    .max(10.0)
                    .min((self.crop_base_height - start_y) as f32);
                self.crop_state.width = new_w as u32;
                self.crop_state.height = new_h as u32;
            }
            HandlePosition::Bottom => {
                let new_h = (start_h as f32 + delta_y)
                    .max(10.0)
                    .min((self.crop_base_height - start_y) as f32);
                self.crop_state.height = new_h as u32;
            }
            HandlePosition::BottomLeft => {
                let new_x = (start_x as f32 + delta_x)
                    .max(0.0)
                    .min(start_x as f32 + start_w as f32 - 10.0);
                let new_h = (start_h as f32 + delta_y)
                    .max(10.0)
                    .min((self.crop_base_height - start_y) as f32);
                self.crop_state.width = (start_x + start_w) - new_x as u32;
                self.crop_state.height = new_h as u32;
                self.crop_state.x = new_x as u32;
            }
            HandlePosition::Left => {
                let new_x = (start_x as f32 + delta_x)
                    .max(0.0)
                    .min(start_x as f32 + start_w as f32 - 10.0);
                self.crop_state.width = (start_x + start_w) - new_x as u32;
                self.crop_state.x = new_x as u32;
            }
        }

        // Apply aspect ratio constraint if needed
        if self.crop_state.ratio != CropRatio::Free {
            self.apply_aspect_ratio_constraint_to_current_crop();
        }
    }

    fn apply_aspect_ratio_constraint_to_current_crop(&mut self) {
        let target_ratio = match self.crop_state.ratio {
            CropRatio::None | CropRatio::Free => return, // No constraint
            CropRatio::Square => 1.0,
            CropRatio::Landscape => 16.0 / 9.0,
            CropRatio::Portrait => 9.0 / 16.0,
            CropRatio::Photo => 4.0 / 3.0,
            CropRatio::PhotoPortrait => 3.0 / 4.0,
        };

        // Adjust height to match ratio, keeping width fixed
        let new_height = (self.crop_state.width as f32 / target_ratio).round() as u32;

        // Check if new height fits
        if self.crop_state.y + new_height <= self.crop_base_height {
            self.crop_state.height = new_height;
        } else {
            // Height doesn't fit, adjust width instead
            let available_height = self.crop_base_height - self.crop_state.y;
            self.crop_state.height = available_height;
            self.crop_state.width = (available_height as f32 * target_ratio).round() as u32;
        }
    }
}
