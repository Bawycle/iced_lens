// SPDX-License-Identifier: MPL-2.0
//! Image editor module with rotate, crop, and resize capabilities.
//!
//! This module follows a "state down, messages up" pattern similar to the settings
//! and viewer modules. The editor operates on a copy of the original image and only
//! modifies the source file when the user explicitly saves.

// TODO: Remove this once editor features are fully implemented
#![allow(dead_code)]

use crate::image_handler::ImageData;
use iced::Rectangle;
use std::path::PathBuf;

/// Contextual data needed to render the editor view.
pub struct ViewContext<'a> {
    pub i18n: &'a crate::i18n::fluent::I18n,
}

/// Local UI state for the editor screen.
#[derive(Debug, Clone)]
pub struct State {
    /// Path to the image being edited
    image_path: PathBuf,
    /// Original unmodified image data
    original_image: ImageData,
    /// Current edited image (after applying transformations)
    current_image: ImageData,
    /// Currently active editing tool
    active_tool: Option<EditorTool>,
    /// History of transformations for undo/redo
    transformation_history: Vec<Transformation>,
    /// Current position in history (for undo/redo)
    history_index: usize,
    /// Whether the sidebar is expanded
    sidebar_expanded: bool,
    /// Crop selection rectangle (in image coordinates)
    crop_selection: Option<Rectangle>,
    /// Crop aspect ratio constraint
    crop_ratio: CropRatio,
    /// Resize state
    resize_state: ResizeState,
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

/// Crop aspect ratio constraints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CropRatio {
    Free,
    Square,      // 1:1
    Landscape,   // 16:9
    Portrait,    // 9:16
    Photo,       // 4:3
    PhotoPortrait, // 3:4
}

/// State for the resize tool.
#[derive(Debug, Clone, PartialEq)]
pub struct ResizeState {
    /// Scale percentage (10-200%)
    pub scale_percent: f32,
    /// Target width in pixels
    pub width: u32,
    /// Target height in pixels
    pub height: u32,
    /// Whether aspect ratio is locked
    pub lock_aspect: bool,
    /// Original aspect ratio
    pub original_aspect: f32,
    /// Width input field value
    pub width_input: String,
    /// Height input field value
    pub height_input: String,
}

/// Messages emitted directly by the editor widgets.
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle sidebar expanded/collapsed
    ToggleSidebar,
    /// Select an editing tool
    SelectTool(EditorTool),
    /// Apply rotation transformation
    RotateLeft,
    RotateRight,
    /// Crop-related messages
    SetCropRatio(CropRatio),
    UpdateCropSelection(Rectangle),
    ApplyCrop,
    /// Resize-related messages
    ScaleChanged(f32),
    WidthInputChanged(String),
    HeightInputChanged(String),
    ToggleLockAspect,
    ApplyResizePreset(f32), // Preset percentage (50%, 75%, 150%, 200%)
    ApplyResize,
    /// Undo/redo
    Undo,
    Redo,
    /// Navigation
    NavigateNext,
    NavigatePrevious,
    /// Save/cancel
    Save,
    SaveAs,
    Cancel,
}

/// Events propagated to the parent application for side effects.
#[derive(Debug, Clone)]
pub enum Event {
    None,
    /// Request to save the edited image
    SaveRequested { path: PathBuf, overwrite: bool },
    /// Request to exit editor mode
    ExitEditor,
    /// Request to navigate to next image
    NavigateNext,
    /// Request to navigate to previous image
    NavigatePrevious,
}

impl State {
    /// Render the editor view.
    pub fn view<'a>(&'a self, ctx: ViewContext<'a>) -> iced::Element<'a, Message> {
        use iced::widget::{button, column, container, text, Column};
        use iced::Length;

        // Minimal view for now - just a placeholder with Cancel button
        let title = text(ctx.i18n.tr("editor-title")).size(30);

        let cancel_button = button(text(ctx.i18n.tr("editor-cancel")))
            .on_press(Message::Cancel);

        let content: Column<'_, Message> = column![title, cancel_button]
            .spacing(20)
            .padding(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill)
            .into()
    }

    /// Update the state and emit an [`Event`] for the parent when needed.
    pub fn update(&mut self, message: Message) -> Event {
        match message {
            Message::Cancel => Event::ExitEditor,
            _ => Event::None,
        }
    }

    /// Create a new editor state for the given image.
    pub fn new(image_path: PathBuf, image: ImageData) -> Self {
        let original_aspect = image.width as f32 / image.height as f32;

        Self {
            image_path,
            original_image: image.clone(),
            current_image: image.clone(),
            active_tool: None,
            transformation_history: Vec::new(),
            history_index: 0,
            sidebar_expanded: true,
            crop_selection: None,
            crop_ratio: CropRatio::Free,
            resize_state: ResizeState {
                scale_percent: 100.0,
                width: image.width,
                height: image.height,
                lock_aspect: true,
                original_aspect,
                width_input: image.width.to_string(),
                height_input: image.height.to_string(),
            },
        }
    }

    /// Check if there are unsaved changes.
    pub fn has_unsaved_changes(&self) -> bool {
        !self.transformation_history.is_empty()
    }

    /// Check if undo is available.
    pub fn can_undo(&self) -> bool {
        self.history_index > 0
    }

    /// Check if redo is available.
    pub fn can_redo(&self) -> bool {
        self.history_index < self.transformation_history.len()
    }

    /// Get the current image data.
    pub fn current_image(&self) -> &ImageData {
        &self.current_image
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
mod tests {
    use super::*;
    use iced::widget::image;

    fn create_test_image() -> ImageData {
        ImageData {
            handle: image::Handle::from_rgba(4, 3, vec![0; 4 * 3 * 4]),
            width: 4,
            height: 3,
        }
    }

    #[test]
    fn new_editor_state_has_no_changes() {
        let img = create_test_image();
        let state = State::new(PathBuf::from("test.png"), img);

        assert!(!state.has_unsaved_changes());
        assert!(!state.can_undo());
        assert!(!state.can_redo());
        assert_eq!(state.active_tool(), None);
    }

    #[test]
    fn new_editor_state_initializes_resize_state() {
        let img = create_test_image();
        let state = State::new(PathBuf::from("test.png"), img);

        assert_eq!(state.resize_state.width, 4);
        assert_eq!(state.resize_state.height, 3);
        assert_eq!(state.resize_state.scale_percent, 100.0);
        assert!(state.resize_state.lock_aspect);
        assert_eq!(state.resize_state.original_aspect, 4.0 / 3.0);
    }

    #[test]
    fn sidebar_starts_expanded() {
        let img = create_test_image();
        let state = State::new(PathBuf::from("test.png"), img);

        assert!(state.is_sidebar_expanded());
    }

    #[test]
    fn crop_ratio_variants_are_distinct() {
        assert_ne!(CropRatio::Free, CropRatio::Square);
        assert_ne!(CropRatio::Landscape, CropRatio::Portrait);
        assert_ne!(CropRatio::Photo, CropRatio::PhotoPortrait);
    }
}
