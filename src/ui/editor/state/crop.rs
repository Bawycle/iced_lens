// SPDX-License-Identifier: MPL-2.0
//! Crop tool state and helpers.

use crate::image_handler::ImageData;

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
