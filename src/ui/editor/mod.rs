// SPDX-License-Identifier: MPL-2.0
//! Image editor module with rotate, crop, and resize capabilities.
//!
//! This module follows a "state down, messages up" pattern similar to the settings
//! and viewer modules. The editor operates on a copy of the original image and only
//! modifies the source file when the user explicitly saves.

// TODO: Remove this once editor features are fully implemented
#![allow(dead_code)]

mod transform;

use crate::error::{Error, Result};
use crate::image_handler::ImageData;
use iced::Rectangle;
use image_rs::DynamicImage;
use std::path::PathBuf;

/// Contextual data needed to render the editor view.
pub struct ViewContext<'a> {
    pub i18n: &'a crate::i18n::fluent::I18n,
}

/// Local UI state for the editor screen.
#[derive(Clone)]
pub struct State {
    /// Path to the image being edited
    image_path: PathBuf,
    /// Original unmodified image data (for display)
    original_image: ImageData,
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
    /// Crop selection rectangle (in image coordinates)
    crop_selection: Option<Rectangle>,
    /// Crop aspect ratio constraint
    crop_ratio: CropRatio,
    /// Resize state
    resize_state: ResizeState,
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

/// Crop aspect ratio constraints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CropRatio {
    Free,
    Square,        // 1:1
    Landscape,     // 16:9
    Portrait,      // 9:16
    Photo,         // 4:3
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
    SaveRequested {
        path: PathBuf,
        overwrite: bool,
    },
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
        use iced::widget::{button, center, container, image, text, Row};
        use iced::{Background, Border, Color, ContentFit, Length};

        // Main layout: Row with sidebar + image area
        let mut main_row = Row::new().spacing(0);

        // Sidebar (always visible, but can be collapsed in future)
        if self.sidebar_expanded {
            let sidebar = self.build_sidebar(ctx);
            main_row = main_row.push(sidebar);
        } else {
            // Collapsed sidebar - just the hamburger button
            let toggle_button = button(text("☰").size(24))
                .on_press(Message::ToggleSidebar)
                .padding(12);

            let collapsed_sidebar = container(toggle_button)
                .width(Length::Fixed(60.0))
                .height(Length::Fill)
                .padding(10)
                .style(|_theme: &iced::Theme| iced::widget::container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.95, 0.95, 0.95))),
                    border: Border {
                        width: 0.0,
                        ..Default::default()
                    },
                    ..Default::default()
                });

            main_row = main_row.push(collapsed_sidebar);
        }

        // Image area with current preview
        let image_widget =
            image(self.current_image.handle.clone()).content_fit(ContentFit::Contain);

        let image_area = container(center(image_widget))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &iced::Theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgb(0.9, 0.9, 0.9))),
                ..Default::default()
            });

        main_row = main_row.push(image_area);

        container(main_row)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Build the expanded sidebar with tools and controls.
    fn build_sidebar<'a>(&'a self, ctx: ViewContext<'a>) -> iced::Element<'a, Message> {
        use iced::widget::{button, container, text, Column, Row};
        use iced::{alignment::Vertical, Background, Border, Color, Length};

        let mut sidebar_content = Column::new()
            .spacing(8)
            .padding(12)
            .width(Length::Fixed(180.0));

        // Hamburger toggle button
        let toggle_button = button(text("☰").size(20))
            .on_press(Message::ToggleSidebar)
            .padding(8)
            .style(iced::widget::button::secondary);

        sidebar_content = sidebar_content.push(
            Row::new()
                .push(toggle_button)
                .push(text("Tools").size(18))
                .spacing(8)
                .align_y(Vertical::Center),
        );

        sidebar_content = sidebar_content.push(iced::widget::horizontal_rule(1));

        // Rotate tools
        let rotate_left_btn =
            button(text(format!("↻\n{}", ctx.i18n.tr("editor-rotate-left"))).size(14))
                .on_press(Message::RotateLeft)
                .padding(12)
                .width(Length::Fill)
                .style(iced::widget::button::secondary);

        let rotate_right_btn =
            button(text(format!("↺\n{}", ctx.i18n.tr("editor-rotate-right"))).size(14))
                .on_press(Message::RotateRight)
                .padding(12)
                .width(Length::Fill)
                .style(iced::widget::button::secondary);

        sidebar_content = sidebar_content.push(rotate_left_btn);
        sidebar_content = sidebar_content.push(rotate_right_btn);

        sidebar_content = sidebar_content.push(iced::widget::horizontal_rule(1));

        // Main tool buttons
        let crop_btn = button(text(ctx.i18n.tr("editor-tool-crop")).size(16))
            .on_press(Message::SelectTool(EditorTool::Crop))
            .padding(12)
            .width(Length::Fill)
            .style(if self.active_tool == Some(EditorTool::Crop) {
                iced::widget::button::primary
            } else {
                iced::widget::button::secondary
            });

        let resize_btn = button(text(ctx.i18n.tr("editor-tool-resize")).size(16))
            .on_press(Message::SelectTool(EditorTool::Resize))
            .padding(12)
            .width(Length::Fill)
            .style(if self.active_tool == Some(EditorTool::Resize) {
                iced::widget::button::primary
            } else {
                iced::widget::button::secondary
            });

        sidebar_content = sidebar_content.push(crop_btn);
        sidebar_content = sidebar_content.push(resize_btn);

        // Spacer to push navigation and action buttons to bottom
        sidebar_content =
            sidebar_content.push(iced::widget::Space::new(Length::Fill, Length::Fill));

        sidebar_content = sidebar_content.push(iced::widget::horizontal_rule(1));

        // Navigation arrows
        let nav_row = Row::new()
            .spacing(8)
            .push(
                button(text("◀").size(20))
                    .on_press(Message::NavigatePrevious)
                    .padding([8, 16])
                    .width(Length::Fill),
            )
            .push(
                button(text("▶").size(20))
                    .on_press(Message::NavigateNext)
                    .padding([8, 16])
                    .width(Length::Fill),
            );

        sidebar_content = sidebar_content.push(nav_row);

        sidebar_content = sidebar_content.push(iced::widget::horizontal_rule(1));

        // Action buttons (Cancel/Save/Save As)
        let cancel_btn = button(text(ctx.i18n.tr("editor-cancel")).size(16))
            .on_press(Message::Cancel)
            .padding(12)
            .width(Length::Fill)
            .style(iced::widget::button::secondary);

        let save_btn = button(text(ctx.i18n.tr("editor-save")).size(16))
            .on_press(Message::Save)
            .padding(12)
            .width(Length::Fill)
            .style(iced::widget::button::primary);

        let save_as_btn = button(text(ctx.i18n.tr("editor-save-as")).size(16))
            .on_press(Message::SaveAs)
            .padding(12)
            .width(Length::Fill)
            .style(iced::widget::button::secondary);

        sidebar_content = sidebar_content.push(cancel_btn);
        sidebar_content = sidebar_content.push(save_btn);
        sidebar_content = sidebar_content.push(save_as_btn);

        // Container with background
        container(sidebar_content)
            .width(Length::Fixed(180.0))
            .height(Length::Fill)
            .style(|_theme: &iced::Theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgb(0.95, 0.95, 0.95))),
                border: Border {
                    width: 0.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    /// Update the state and emit an [`Event`] for the parent when needed.
    pub fn update(&mut self, message: Message) -> Event {
        match message {
            Message::ToggleSidebar => {
                self.sidebar_expanded = !self.sidebar_expanded;
                Event::None
            }
            Message::SelectTool(tool) => {
                self.active_tool = Some(tool);
                Event::None
            }
            Message::RotateLeft => {
                self.apply_dynamic_transformation(
                    Transformation::RotateLeft,
                    transform::rotate_left,
                );
                Event::None
            }
            Message::RotateRight => {
                self.apply_dynamic_transformation(
                    Transformation::RotateRight,
                    transform::rotate_right,
                );
                Event::None
            }
            Message::SetCropRatio(ratio) => {
                self.crop_ratio = ratio;
                Event::None
            }
            Message::UpdateCropSelection(_rect) => {
                // TODO: Implement crop selection
                Event::None
            }
            Message::ApplyCrop => {
                // TODO: Implement crop application
                Event::None
            }
            Message::ScaleChanged(_percent) => {
                // TODO: Implement scale change
                Event::None
            }
            Message::WidthInputChanged(_value) => {
                // TODO: Implement width input
                Event::None
            }
            Message::HeightInputChanged(_value) => {
                // TODO: Implement height input
                Event::None
            }
            Message::ToggleLockAspect => {
                self.resize_state.lock_aspect = !self.resize_state.lock_aspect;
                Event::None
            }
            Message::ApplyResizePreset(_percent) => {
                // TODO: Implement resize preset
                Event::None
            }
            Message::ApplyResize => {
                // TODO: Implement resize application
                Event::None
            }
            Message::Undo => {
                // TODO: Implement undo
                Event::None
            }
            Message::Redo => {
                // TODO: Implement redo
                Event::None
            }
            Message::NavigateNext => Event::NavigateNext,
            Message::NavigatePrevious => Event::NavigatePrevious,
            Message::Save => {
                // Save overwrites the original file (confirmation may be added later)
                Event::SaveRequested {
                    path: self.image_path.clone(),
                    overwrite: true,
                }
            }
            Message::SaveAs => {
                // TODO: Implement file picker dialog for save location
                // For now, emit event with overwrite: false to signal "save as" intent
                Event::SaveRequested {
                    path: self.image_path.clone(),
                    overwrite: false,
                }
            }
            Message::Cancel => Event::ExitEditor,
        }
    }

    /// Create a new editor state for the given image.
    pub fn new(image_path: PathBuf, image: ImageData) -> Result<Self> {
        let original_aspect = image.width as f32 / image.height as f32;
        let working_image =
            image_rs::open(&image_path).map_err(|err| Error::Io(err.to_string()))?;

        Ok(Self {
            image_path,
            original_image: image.clone(),
            current_image: image.clone(),
            working_image,
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
        })
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
                self.record_transformation(transformation);
            }
            Err(err) => {
                eprintln!("Failed to apply transformation: {err:?}");
            }
        }
    }

    fn sync_resize_state_dimensions(&mut self) {
        self.resize_state.width = self.current_image.width;
        self.resize_state.height = self.current_image.height;
        self.resize_state.width_input = self.current_image.width.to_string();
        self.resize_state.height_input = self.current_image.height.to_string();
    }

    fn record_transformation(&mut self, transformation: Transformation) {
        if self.history_index < self.transformation_history.len() {
            self.transformation_history.truncate(self.history_index);
        }
        self.transformation_history.push(transformation);
        self.history_index = self.transformation_history.len();
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
}
