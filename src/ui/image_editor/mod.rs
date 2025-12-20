// SPDX-License-Identifier: MPL-2.0
//! Image editor module with rotate, crop, and resize capabilities.
//!
//! This module follows a "state down, messages up" pattern similar to the settings
//! and viewer modules. The editor operates on a copy of the original image and only
//! modifies the source file when the user explicitly saves.

use crate::media::frame_export::ExportFormat;
use crate::media::ImageData;
use crate::ui::state::{DragState, ViewportState, ZoomState};

mod component;
mod messages;
mod overlay;
mod state;
mod view;

pub use self::state::{
    AdjustmentState, CropDragState, CropOverlay, CropRatio, CropState, DeblurState, HandlePosition,
    ResizeOverlay, ResizeState,
};
pub use component::{EditorTool, Transformation, ViewContext};
use image_rs::DynamicImage;
pub use messages::{CanvasMessage, Event, Message, SidebarMessage, ToolbarMessage};
use std::path::PathBuf;

/// Source of the image being edited.
#[derive(Debug, Clone)]
pub enum ImageSource {
    /// Image loaded from a file on disk.
    File(PathBuf),
    /// Captured video frame (no source file).
    CapturedFrame {
        /// Original video path (for default filename generation).
        video_path: PathBuf,
        /// Position in seconds when frame was captured.
        position_secs: f64,
    },
}

/// Local UI state for the editor screen.
#[derive(Clone)]
pub struct State {
    /// Source of the image being edited (file or captured frame).
    image_source: ImageSource,
    /// Original image (for undo/redo replay).
    /// For files, this is loaded from disk. For captured frames, stored at creation.
    original_image: DynamicImage,
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
    /// Adjustment state (brightness/contrast)
    adjustment_state: AdjustmentState,
    /// Deblur state (AI-powered deblurring)
    deblur_state: DeblurState,
    /// Optional preview image (used for live adjustments)
    preview_image: Option<ImageData>,
    /// Viewport state for tracking canvas bounds and scroll position
    pub viewport: ViewportState,
    /// Export format for Save As (used when editing captured frames).
    export_format: ExportFormat,
    /// Zoom state for the editor canvas
    pub zoom: ZoomState,
    /// Current cursor position (for zoom-on-scroll detection)
    cursor_position: Option<iced::Point>,
    /// Drag state for pan navigation
    drag: DragState,
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("image_source", &self.image_source)
            .field("active_tool", &self.active_tool)
            .field("transformation_history", &self.transformation_history)
            .field("history_index", &self.history_index)
            .field("sidebar_expanded", &self.sidebar_expanded)
            .field("export_format", &self.export_format)
            .finish()
    }
}

impl State {
    /// Update the state and emit an [`Event`] for the parent when needed.
    pub fn update(&mut self, message: Message) -> Event {
        match message {
            Message::Toolbar(msg) => self.handle_toolbar_message(msg),
            Message::Sidebar(msg) => self.handle_sidebar_message(msg),
            Message::Canvas(msg) => self.handle_canvas_message(msg),
            Message::RawEvent { event, .. } => self.handle_raw_event(event),
            Message::ViewportChanged { bounds, offset } => {
                self.viewport.update(bounds, offset);
                Event::None
            }
            Message::SpinnerTick => {
                self.deblur_state.tick_spinner();
                Event::None
            }
        }
    }

    /// Returns the subscriptions needed for the editor (spinner animation during deblur).
    pub fn subscription(&self) -> iced::Subscription<Message> {
        if self.deblur_state.is_processing {
            // Animate spinner at 60 FPS while processing
            iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::SpinnerTick)
        } else {
            iced::Subscription::none()
        }
    }

    // Message handlers now live in state::routing

    /// Get the current image data.
    pub fn current_image(&self) -> &ImageData {
        &self.current_image
    }

    /// Get the working image for transformations.
    pub fn working_image(&self) -> &DynamicImage {
        &self.working_image
    }

    /// Get the image source.
    pub fn image_source(&self) -> &ImageSource {
        &self.image_source
    }

    /// Get the image file path (if editing a file).
    pub fn image_path(&self) -> Option<&std::path::Path> {
        match &self.image_source {
            ImageSource::File(path) => Some(path),
            ImageSource::CapturedFrame { .. } => None,
        }
    }

    /// Check if editing a captured frame (no source file).
    pub fn is_captured_frame(&self) -> bool {
        matches!(self.image_source, ImageSource::CapturedFrame { .. })
    }

    /// Get the active tool.
    pub fn active_tool(&self) -> Option<EditorTool> {
        self.active_tool
    }

    /// Check if sidebar is expanded.
    pub fn is_sidebar_expanded(&self) -> bool {
        self.sidebar_expanded
    }

    /// Get the selected export format.
    pub fn export_format(&self) -> ExportFormat {
        self.export_format
    }

    /// Set the export format.
    pub fn set_export_format(&mut self, format: ExportFormat) {
        self.export_format = format;
    }

    /// Get the resize thumbnail preview (for sidebar display).
    pub fn resize_thumbnail(&self) -> Option<&ImageData> {
        // Only return thumbnail when resize tool is active
        if self.active_tool == Some(EditorTool::Resize) {
            self.preview_image.as_ref()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests;
