// SPDX-License-Identifier: MPL-2.0
//! Image editor module with rotate, crop, and resize capabilities.
//!
//! This module follows a "state down, messages up" pattern similar to the settings
//! and viewer modules. The editor operates on a copy of the original image and only
//! modifies the source file when the user explicitly saves.

use crate::config::BackgroundTheme;
use crate::error::{Error, Result};
use crate::image_handler::transform;
use crate::image_handler::ImageData;
use iced::Rectangle;

mod overlay;
mod state;
mod view;

pub use self::state::{
    CropDragState, CropOverlay, CropRatio, CropState, HandlePosition, ResizeOverlay, ResizeState,
};
use image_rs::DynamicImage;
use std::path::PathBuf;

/// Contextual data needed to render the editor view.
pub struct ViewContext<'a> {
    pub i18n: &'a crate::i18n::fluent::I18n,
    pub background_theme: BackgroundTheme,
}

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

/// Toolbar-specific messages.
#[derive(Debug, Clone)]
pub enum ToolbarMessage {
    BackToViewer,
}

/// Sidebar control messages.
#[derive(Debug, Clone)]
pub enum SidebarMessage {
    ToggleSidebar,
    SelectTool(EditorTool),
    RotateLeft,
    RotateRight,
    SetCropRatio(CropRatio),
    ApplyCrop,
    ScaleChanged(f32),
    WidthInputChanged(String),
    HeightInputChanged(String),
    ToggleLockAspect,
    ApplyResizePreset(f32),
    ApplyResize,
    Undo,
    Redo,
    NavigateNext,
    NavigatePrevious,
    Save,
    SaveAs,
    Cancel,
}

/// Canvas overlay interaction messages.
#[derive(Debug, Clone)]
pub enum CanvasMessage {
    CropOverlayMouseDown { x: f32, y: f32 },
    CropOverlayMouseMove { x: f32, y: f32 },
    CropOverlayMouseUp,
}

/// Messages emitted directly by the editor widgets.
#[derive(Debug, Clone)]
pub enum Message {
    Toolbar(ToolbarMessage),
    Sidebar(SidebarMessage),
    Canvas(CanvasMessage),
    /// Raw event for keyboard shortcuts
    RawEvent {
        window: iced::window::Id,
        event: iced::Event,
    },
}

impl From<ToolbarMessage> for Message {
    fn from(message: ToolbarMessage) -> Self {
        Message::Toolbar(message)
    }
}

impl From<SidebarMessage> for Message {
    fn from(message: SidebarMessage) -> Self {
        Message::Sidebar(message)
    }
}

impl From<CanvasMessage> for Message {
    fn from(message: CanvasMessage) -> Self {
        Message::Canvas(message)
    }
}

/// Events propagated to the parent application for side effects.
#[derive(Debug, Clone)]
pub enum Event {
    None,
    /// Request to save the edited image
    SaveRequested {
        path: PathBuf,
        overwrite: bool,
    },
    /// Request to open file picker for "Save As"
    SaveAsRequested,
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
        view::render(self, ctx)
    }

    /// Update the state and emit an [`Event`] for the parent when needed.
    pub fn update(&mut self, message: Message) -> Event {
        match message {
            Message::Toolbar(msg) => self.handle_toolbar_message(msg),
            Message::Sidebar(msg) => self.handle_sidebar_message(msg),
            Message::Canvas(msg) => self.handle_canvas_message(msg),
            Message::RawEvent { event, .. } => self.handle_raw_event(event),
        }
    }

    fn handle_toolbar_message(&mut self, message: ToolbarMessage) -> Event {
        match message {
            ToolbarMessage::BackToViewer => {
                if self.has_unsaved_changes() {
                    Event::None
                } else {
                    Event::ExitEditor
                }
            }
        }
    }

    fn handle_sidebar_message(&mut self, message: SidebarMessage) -> Event {
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
                    if tool == EditorTool::Crop {
                        self.crop_modified = false;
                        self.crop_base_image = None;
                        self.crop_state.overlay.visible = false;
                        self.crop_state.overlay.drag_state = CropDragState::None;
                    }
                    if tool == EditorTool::Resize {
                        self.resize_state.overlay.visible = false;
                    }
                } else {
                    self.commit_active_tool_changes();
                    if self.active_tool == Some(EditorTool::Crop) {
                        self.crop_state.overlay.visible = false;
                        self.crop_state.overlay.drag_state = CropDragState::None;
                    }
                    if self.active_tool == Some(EditorTool::Resize) {
                        self.resize_state.overlay.visible = false;
                    }
                    self.active_tool = Some(tool);
                    self.preview_image = None;

                    if tool == EditorTool::Crop {
                        self.crop_base_image = Some(self.working_image.clone());
                        self.crop_base_width = self.current_image.width;
                        self.crop_base_height = self.current_image.height;
                        self.crop_state.x = 0;
                        self.crop_state.y = 0;
                        self.crop_state.width = self.current_image.width;
                        self.crop_state.height = self.current_image.height;
                        self.crop_state.ratio = CropRatio::None;
                        self.crop_state.overlay.visible = false;
                    }

                    if tool == EditorTool::Resize {
                        self.resize_state.overlay.visible = true;
                        self.resize_state.overlay.set_original_dimensions(
                            self.current_image.width,
                            self.current_image.height,
                        );
                    }
                }
                Event::None
            }
            SidebarMessage::RotateLeft => {
                self.commit_active_tool_changes();
                self.apply_dynamic_transformation(
                    Transformation::RotateLeft,
                    transform::rotate_left,
                );
                Event::None
            }
            SidebarMessage::RotateRight => {
                self.commit_active_tool_changes();
                self.apply_dynamic_transformation(
                    Transformation::RotateRight,
                    transform::rotate_right,
                );
                Event::None
            }
            SidebarMessage::SetCropRatio(ratio) => {
                self.crop_state.ratio = ratio;
                self.adjust_crop_to_ratio(ratio);
                self.crop_state.overlay.visible = true;
                self.crop_modified = true;
                Event::None
            }
            SidebarMessage::ApplyCrop => {
                if self.crop_state.overlay.visible {
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
                Event::None
            }
            SidebarMessage::ScaleChanged(percent) => {
                self.set_resize_percent(percent);
                Event::None
            }
            SidebarMessage::WidthInputChanged(value) => {
                self.handle_width_input_change(value);
                Event::None
            }
            SidebarMessage::HeightInputChanged(value) => {
                self.handle_height_input_change(value);
                Event::None
            }
            SidebarMessage::ToggleLockAspect => {
                self.toggle_resize_lock();
                Event::None
            }
            SidebarMessage::ApplyResizePreset(percent) => {
                self.set_resize_percent(percent);
                Event::None
            }
            SidebarMessage::ApplyResize => {
                self.apply_resize_dimensions();
                Event::None
            }
            SidebarMessage::Undo => {
                self.commit_active_tool_changes();
                if self.can_undo() {
                    self.history_index -= 1;
                    self.replay_transformations_up_to_index();
                }
                Event::None
            }
            SidebarMessage::Redo => {
                self.commit_active_tool_changes();
                if self.can_redo() {
                    self.history_index += 1;
                    self.replay_transformations_up_to_index();
                }
                Event::None
            }
            SidebarMessage::NavigateNext => {
                if self.has_unsaved_changes() {
                    Event::None
                } else {
                    self.commit_active_tool_changes();
                    Event::NavigateNext
                }
            }
            SidebarMessage::NavigatePrevious => {
                if self.has_unsaved_changes() {
                    Event::None
                } else {
                    self.commit_active_tool_changes();
                    Event::NavigatePrevious
                }
            }
            SidebarMessage::Save => {
                self.commit_active_tool_changes();
                Event::SaveRequested {
                    path: self.image_path.clone(),
                    overwrite: true,
                }
            }
            SidebarMessage::SaveAs => {
                self.commit_active_tool_changes();
                Event::SaveAsRequested
            }
            SidebarMessage::Cancel => {
                self.discard_changes();
                Event::None
            }
        }
    }

    fn handle_canvas_message(&mut self, message: CanvasMessage) -> Event {
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

    fn handle_raw_event(&mut self, event: iced::Event) -> Event {
        use iced::keyboard;

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
                        if self.has_unsaved_changes() {
                            Event::SaveRequested {
                                path: self.image_path.clone(),
                                overwrite: true,
                            }
                        } else {
                            Event::None
                        }
                    }
                    keyboard::Key::Character(ref c) if c.as_str() == "z" => {
                        self.commit_active_tool_changes();
                        if self.can_undo() {
                            self.history_index -= 1;
                            self.replay_transformations_up_to_index();
                        }
                        Event::None
                    }
                    keyboard::Key::Character(ref c) if c.as_str() == "y" => {
                        self.commit_active_tool_changes();
                        if self.can_redo() {
                            self.history_index += 1;
                            self.replay_transformations_up_to_index();
                        }
                        Event::None
                    }
                    _ => Event::None,
                }
            }
            _ => Event::None,
        }
    }

    /// Create a new editor state for the given image.
    pub fn new(image_path: PathBuf, image: ImageData) -> Result<Self> {
        let working_image =
            image_rs::open(&image_path).map_err(|err| Error::Io(err.to_string()))?;

        Ok(Self {
            image_path,
            current_image: image.clone(),
            working_image,
            active_tool: None,
            transformation_history: Vec::new(),
            history_index: 0,
            sidebar_expanded: true,
            crop_state: CropState::from_image(&image),
            crop_modified: false,
            resize_state: ResizeState::from_image(&image),
            crop_base_image: None,
            crop_base_width: image.width,
            crop_base_height: image.height,
            preview_image: None,
        })
    }
    /// Check if there are unsaved changes based on transformation history.
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

    /// Save the edited image to a file, preserving the original format.
    pub fn save_image(&mut self, path: &std::path::Path) -> Result<()> {
        use image_rs::ImageFormat;

        // Detect format from file extension
        let format = match path.extension().and_then(|s| s.to_str()) {
            Some("jpg") | Some("jpeg") => ImageFormat::Jpeg,
            Some("png") => ImageFormat::Png,
            Some("gif") => ImageFormat::Gif,
            Some("bmp") => ImageFormat::Bmp,
            Some("ico") => ImageFormat::Ico,
            Some("tiff") | Some("tif") => ImageFormat::Tiff,
            Some("webp") => ImageFormat::WebP,
            _ => ImageFormat::Png, // Default fallback
        };

        // Save the working image
        self.working_image
            .save_with_format(path, format)
            .map_err(|err| Error::Io(format!("Failed to save image: {}", err)))?;

        // Clear transformation history after successful save
        self.transformation_history.clear();
        self.history_index = 0;

        Ok(())
    }

    /// Get the current image data.
    pub fn current_image(&self) -> &ImageData {
        &self.current_image
    }

    /// Get the image file path.
    pub fn image_path(&self) -> &std::path::Path {
        &self.image_path
    }

    fn display_image(&self) -> &ImageData {
        self.preview_image.as_ref().unwrap_or(&self.current_image)
    }

    /// Get the active tool.
    pub fn active_tool(&self) -> Option<EditorTool> {
        self.active_tool
    }

    /// Check if sidebar is expanded.
    pub fn is_sidebar_expanded(&self) -> bool {
        self.sidebar_expanded
    }

    fn apply_dynamic_transformation<F>(&mut self, transformation: Transformation, operation: F)
    where
        F: Fn(&DynamicImage) -> DynamicImage,
    {
        let updated = operation(&self.working_image);
        match transform::dynamic_to_image_data(&updated) {
            Ok(image_data) => {
                self.working_image = updated;
                self.current_image = image_data;
                self.sync_resize_state_dimensions();
                self.preview_image = None;
                self.record_transformation(transformation);
            }
            Err(err) => {
                eprintln!("Failed to apply transformation: {err:?}");
            }
        }
    }

    fn sync_resize_state_dimensions(&mut self) {
        self.resize_state.sync_from_image(&self.current_image);
    }

    fn record_transformation(&mut self, transformation: Transformation) {
        if self.history_index < self.transformation_history.len() {
            self.transformation_history.truncate(self.history_index);
        }
        self.transformation_history.push(transformation);
        self.history_index = self.transformation_history.len();
    }

    fn base_width(&self) -> f32 {
        self.current_image.width.max(1) as f32
    }

    fn base_height(&self) -> f32 {
        self.current_image.height.max(1) as f32
    }

    fn commit_active_tool_changes(&mut self) {
        if matches!(self.active_tool, Some(EditorTool::Crop))
            && self.crop_modified
            && self.crop_state.overlay.visible
        {
            self.finalize_crop_overlay();
        }
    }

    fn set_resize_percent(&mut self, percent: f32) {
        let clamped = percent.clamp(10.0, 200.0);
        self.resize_state.scale_percent = clamped;
        let width = (self.base_width() * clamped / 100.0).round().max(1.0) as u32;
        let height = (self.base_height() * clamped / 100.0).round().max(1.0) as u32;

        if self.resize_state.lock_aspect {
            self.set_width_preserving_aspect(width);
        } else {
            self.resize_state.width = width;
            self.resize_state.height = height;
            self.resize_state.width_input = width.to_string();
            self.resize_state.height_input = height.to_string();
        }

        self.update_resize_preview();
    }

    fn handle_width_input_change(&mut self, value: String) {
        self.resize_state.width_input = value.clone();
        if let Some(width) = parse_dimension_input(&value) {
            if self.resize_state.lock_aspect {
                self.set_width_preserving_aspect(width);
            } else {
                let width = width.max(1);
                self.resize_state.width = width;
                self.resize_state.width_input = width.to_string();
            }
            self.update_scale_percent_from_width();
        }
    }

    fn handle_height_input_change(&mut self, value: String) {
        self.resize_state.height_input = value.clone();
        if let Some(height) = parse_dimension_input(&value) {
            if self.resize_state.lock_aspect {
                self.set_height_preserving_aspect(height);
                self.update_scale_percent_from_width();
            } else {
                let height = height.max(1);
                self.resize_state.height = height;
                self.resize_state.height_input = height.to_string();
            }
            self.update_resize_preview();
        }
    }

    fn toggle_resize_lock(&mut self) {
        self.resize_state.lock_aspect = !self.resize_state.lock_aspect;
        if self.resize_state.lock_aspect {
            let width = self.resize_state.width;
            self.set_width_preserving_aspect(width);
        }
        self.update_resize_preview();
    }

    fn set_width_preserving_aspect(&mut self, width: u32) {
        let width = width.max(1);
        let aspect = self.resize_state.original_aspect.max(f32::EPSILON);
        let height = (width as f32 / aspect).round().max(1.0) as u32;
        self.resize_state.width = width;
        self.resize_state.height = height;
        self.resize_state.width_input = width.to_string();
        self.resize_state.height_input = height.to_string();
    }

    fn set_height_preserving_aspect(&mut self, height: u32) {
        let height = height.max(1);
        let aspect = self.resize_state.original_aspect.max(f32::EPSILON);
        let width = (height as f32 * aspect).round().max(1.0) as u32;
        self.resize_state.height = height;
        self.resize_state.width = width.max(1);
        self.resize_state.width_input = self.resize_state.width.to_string();
        self.resize_state.height_input = height.to_string();
    }

    fn update_scale_percent_from_width(&mut self) {
        let base_width = self.base_width();
        if base_width <= 0.0 {
            return;
        }
        let percent = (self.resize_state.width as f32 / base_width) * 100.0;
        let clamped = percent.clamp(10.0, 200.0);
        if (clamped - percent).abs() > f32::EPSILON {
            self.set_resize_percent(clamped);
        } else {
            self.resize_state.scale_percent = clamped;
            self.update_resize_preview();
        }
    }

    fn apply_resize_dimensions(&mut self) {
        let target_width = self.resize_state.width.max(1);
        let target_height = self.resize_state.height.max(1);
        if target_width == self.current_image.width && target_height == self.current_image.height {
            return;
        }

        self.apply_dynamic_transformation(
            Transformation::Resize {
                width: target_width,
                height: target_height,
            },
            move |image| transform::resize(image, target_width, target_height),
        );

        self.resize_state
            .overlay
            .set_original_dimensions(self.current_image.width, self.current_image.height);
    }

    fn update_resize_preview(&mut self) {
        // Don't generate preview when overlay is visible - the overlay will show the preview
        if self.resize_state.overlay.visible {
            self.preview_image = None;
            return;
        }

        let target_width = self.resize_state.width.max(1);
        let target_height = self.resize_state.height.max(1);
        if target_width == self.current_image.width && target_height == self.current_image.height {
            self.preview_image = None;
            return;
        }

        let preview_dynamic = transform::resize(&self.working_image, target_width, target_height);
        match transform::dynamic_to_image_data(&preview_dynamic) {
            Ok(image_data) => {
                self.preview_image = Some(image_data);
            }
            Err(err) => {
                eprintln!("Failed to build resize preview: {err:?}");
                self.preview_image = None;
            }
        }
    }

    fn adjust_crop_to_ratio(&mut self, ratio: CropRatio) {
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

    fn apply_crop_from_base(&mut self) {
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

    fn finalize_crop_overlay(&mut self) {
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
    fn handle_crop_overlay_mouse_down(&mut self, x: f32, y: f32) {
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
    fn handle_crop_overlay_mouse_move(&mut self, x: f32, y: f32) {
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

    /// Check if a point is inside the crop rectangle
    fn is_point_in_crop_rect(&self, x: f32, y: f32) -> bool {
        let rect_x = self.crop_state.x as f32;
        let rect_y = self.crop_state.y as f32;
        let rect_w = self.crop_state.width as f32;
        let rect_h = self.crop_state.height as f32;

        x >= rect_x && x <= rect_x + rect_w && y >= rect_y && y <= rect_y + rect_h
    }

    /// Get the handle at the given position, if any
    fn get_handle_at_position(&self, x: f32, y: f32) -> Option<HandlePosition> {
        const HANDLE_SIZE: f32 = 12.0; // Handle click area size

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

    /// Update crop rectangle from handle drag
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

    /// Apply aspect ratio constraint to current crop dimensions
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

    /// Discard all changes and reset to original image state.
    pub fn discard_changes(&mut self) {
        // Reload the working image from disk
        match image_rs::open(&self.image_path) {
            Ok(fresh_image) => {
                self.working_image = fresh_image;
                match transform::dynamic_to_image_data(&self.working_image) {
                    Ok(image_data) => {
                        self.current_image = image_data.clone();
                        self.sync_resize_state_dimensions();

                        // Reset crop state
                        let crop_width = (self.current_image.width as f32 * 0.75).round() as u32;
                        let crop_height = (self.current_image.height as f32 * 0.75).round() as u32;
                        self.crop_state.x = (self.current_image.width - crop_width) / 2;
                        self.crop_state.y = (self.current_image.height - crop_height) / 2;
                        self.crop_state.width = crop_width;
                        self.crop_state.height = crop_height;
                        self.crop_state.ratio = CropRatio::Free;
                        self.crop_state.overlay.visible = false;
                        self.crop_state.overlay.drag_state = CropDragState::None;
                        self.crop_modified = false;

                        // Hide resize overlay to avoid stale rectangles after cancel
                        self.resize_state.overlay.visible = false;
                        self.resize_state.overlay.set_original_dimensions(
                            self.current_image.width,
                            self.current_image.height,
                        );

                        // Clear transformation history
                        self.transformation_history.clear();
                        self.history_index = 0;

                        // Clear active tool and preview
                        self.active_tool = None;
                        self.preview_image = None;
                    }
                    Err(err) => {
                        eprintln!("Failed to convert reloaded image: {err:?}");
                    }
                }
            }
            Err(err) => {
                eprintln!("Failed to reload original image: {err:?}");
            }
        }
    }

    /// Replay transformations from the original image up to the current history_index.
    /// This is used for undo/redo operations.
    fn replay_transformations_up_to_index(&mut self) {
        // Reload the original image from disk
        let Ok(mut working_image) = image_rs::open(&self.image_path) else {
            eprintln!("Failed to reload original image for replay");
            return;
        };

        // Apply transformations up to history_index
        for i in 0..self.history_index {
            if i >= self.transformation_history.len() {
                break;
            }

            working_image = match &self.transformation_history[i] {
                Transformation::RotateLeft => transform::rotate_left(&working_image),
                Transformation::RotateRight => transform::rotate_right(&working_image),
                Transformation::Crop { rect } => {
                    let x = rect.x.max(0.0) as u32;
                    let y = rect.y.max(0.0) as u32;
                    let width = rect.width.max(1.0) as u32;
                    let height = rect.height.max(1.0) as u32;
                    match transform::crop(&working_image, x, y, width, height) {
                        Some(cropped) => cropped,
                        None => {
                            eprintln!("Failed to apply crop during replay: invalid crop area");
                            working_image
                        }
                    }
                }
                Transformation::Resize { width, height } => {
                    transform::resize(&working_image, *width, *height)
                }
            };
        }

        // Update current state with replayed image
        self.working_image = working_image;
        match transform::dynamic_to_image_data(&self.working_image) {
            Ok(image_data) => {
                self.current_image = image_data;
                self.sync_resize_state_dimensions();
                self.preview_image = None;
            }
            Err(err) => {
                eprintln!("Failed to convert replayed image: {err:?}");
            }
        }
    }
}

fn parse_dimension_input(value: &str) -> Option<u32> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    match trimmed.parse::<u32>() {
        Ok(result) if result > 0 => Some(result),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::image;
    use image_rs::{Rgba, RgbaImage};
    use tempfile::tempdir;

    fn create_test_image(width: u32, height: u32) -> (tempfile::TempDir, PathBuf, ImageData) {
        let temp_dir = tempdir().expect("temp dir");
        let path = temp_dir.path().join("test.png");
        let img = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 255]));
        img.save(&path).expect("write png");
        let pixels = vec![0; (width * height * 4) as usize];
        let image = ImageData {
            handle: image::Handle::from_rgba(width, height, pixels),
            width,
            height,
        };
        (temp_dir, path, image)
    }

    #[test]
    fn new_editor_state_has_no_changes() {
        let (_dir, path, img) = create_test_image(4, 3);
        let state = State::new(path, img).expect("editor state");

        assert!(!state.has_unsaved_changes());
        assert!(!state.can_undo());
        assert!(!state.can_redo());
        assert_eq!(state.active_tool(), None);
    }

    #[test]
    fn new_editor_state_initializes_resize_state() {
        let (_dir, path, img) = create_test_image(4, 3);
        let state = State::new(path, img).expect("editor state");

        assert_eq!(state.resize_state.width, 4);
        assert_eq!(state.resize_state.height, 3);
        assert_eq!(state.resize_state.scale_percent, 100.0);
        assert!(state.resize_state.lock_aspect);
        assert_eq!(state.resize_state.original_aspect, 4.0 / 3.0);
    }

    #[test]
    fn sidebar_starts_expanded() {
        let (_dir, path, img) = create_test_image(4, 3);
        let state = State::new(path, img).expect("editor state");

        assert!(state.is_sidebar_expanded());
    }

    #[test]
    fn crop_ratio_variants_are_distinct() {
        assert_ne!(CropRatio::Free, CropRatio::Square);
        assert_ne!(CropRatio::Landscape, CropRatio::Portrait);
        assert_ne!(CropRatio::Photo, CropRatio::PhotoPortrait);
    }

    #[test]
    fn apply_resize_updates_image_dimensions() {
        let (_dir, path, img) = create_test_image(8, 6);
        let mut state = State::new(path, img).expect("editor state");

        state.update(Message::Sidebar(SidebarMessage::SelectTool(
            EditorTool::Resize,
        )));
        state.resize_state.width = 4;
        state.resize_state.height = 3;
        state.resize_state.width_input = "4".into();
        state.resize_state.height_input = "3".into();
        state.update(Message::Sidebar(SidebarMessage::ApplyResize));

        assert_eq!(state.current_image.width, 4);
        assert_eq!(state.current_image.height, 3);
    }

    #[test]
    fn cancel_hides_resize_overlay() {
        let (_dir, path, img) = create_test_image(5, 4);
        let mut state = State::new(path, img).expect("editor state");

        state.update(Message::Sidebar(SidebarMessage::SelectTool(
            EditorTool::Resize,
        )));
        assert!(state.resize_state.overlay.visible);

        state.discard_changes();

        assert!(!state.resize_state.overlay.visible);
    }
}
