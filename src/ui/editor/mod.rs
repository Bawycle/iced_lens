// SPDX-License-Identifier: MPL-2.0
//! Image editor module with rotate, crop, and resize capabilities.
//!
//! This module follows a "state down, messages up" pattern similar to the settings
//! and viewer modules. The editor operates on a copy of the original image and only
//! modifies the source file when the user explicitly saves.

use crate::image_handler::ImageData;
use iced::Rectangle;

mod component;
mod messages;
mod overlay;
mod state;
mod view;

pub use self::state::{
    CropDragState, CropOverlay, CropRatio, CropState, HandlePosition, ResizeOverlay, ResizeState,
};
pub use component::ViewContext;
use image_rs::DynamicImage;
pub use messages::{CanvasMessage, Event, Message, SidebarMessage, ToolbarMessage};
use std::path::PathBuf;

/// Local UI state for the editor screen.
#[derive(Clone)]
pub struct State {
    /// Path to the image being edited
    image_path: PathBuf,
    /// Current edited image (after applying transformations, for display)
    current_image: ImageData,
    /// Working image for transformations (DynamicImage from image_rs crate)
    working_image: DynamicImage,
    /// Currently active editing tool
    active_tool: Option<EditorTool>,
    /// History of transformations for undo/redo
    transformation_history: Vec<Transformation>,
    /// Current position in history (for undo/redo)
    history_index: usize,
    /// Whether the sidebar is expanded
    sidebar_expanded: bool,
    /// Crop tool state
    crop_state: CropState,
    /// Track if crop state has been modified (to avoid auto-commit on tool close)
    crop_modified: bool,
    /// Image state when crop tool was opened (to calculate ratios from original, not from previous crops)
    crop_base_image: Option<DynamicImage>,
    crop_base_width: u32,
    crop_base_height: u32,
    /// Resize state
    resize_state: ResizeState,
    /// Optional preview image (used for live adjustments)
    preview_image: Option<ImageData>,
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("image_path", &self.image_path)
            .field("active_tool", &self.active_tool)
            .field("transformation_history", &self.transformation_history)
            .field("history_index", &self.history_index)
            .field("sidebar_expanded", &self.sidebar_expanded)
            .finish()
    }
}

/// Available editing tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTool {
    Rotate,
    Crop,
    Resize,
}

/// Image transformations that can be applied and undone.
#[derive(Debug, Clone, PartialEq)]
pub enum Transformation {
    RotateLeft,
    RotateRight,
    Crop { rect: Rectangle },
    Resize { width: u32, height: u32 },
}

impl State {
    /// Update the state and emit an [`Event`] for the parent when needed.
    pub fn update(&mut self, message: Message) -> Event {
        match message {
            Message::Toolbar(msg) => self.handle_toolbar_message(msg),
            Message::Sidebar(msg) => self.handle_sidebar_message(msg),
            Message::Canvas(msg) => self.handle_canvas_message(msg),
            Message::RawEvent { event, .. } => self.handle_raw_event(event),
        }
    }

    // Message handlers now live in state::routing

    /// Get the current image data.
    pub fn current_image(&self) -> &ImageData {
        &self.current_image
    }

    /// Get the image file path.
    pub fn image_path(&self) -> &std::path::Path {
        &self.image_path
    }

    /// Get the active tool.
    pub fn active_tool(&self) -> Option<EditorTool> {
        self.active_tool
    }

    /// Check if sidebar is expanded.
    pub fn is_sidebar_expanded(&self) -> bool {
        self.sidebar_expanded
    }
}

#[cfg(test)]
mod tests;
