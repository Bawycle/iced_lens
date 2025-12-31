// SPDX-License-Identifier: MPL-2.0
//! Public-facing view helpers and constructor for the editor facade.

use crate::config::BackgroundTheme;
use crate::error::{Error, Result};
use crate::media::deblur::ModelStatus;
use crate::media::frame_export::{ExportFormat, ExportableFrame};
use crate::media::upscale::UpscaleModelStatus;
use crate::media::ImageData;
use iced::{Element, Rectangle};
use image_rs;
use std::path::PathBuf;

use super::{state, view, ImageSource, Message, State};

/// Contextual data needed to render the editor view.
pub struct ViewContext<'a> {
    pub i18n: &'a crate::i18n::fluent::I18n,
    pub background_theme: BackgroundTheme,
    /// True if the application is using dark theme.
    pub is_dark_theme: bool,
    /// Current status of the AI deblur model.
    pub deblur_model_status: &'a ModelStatus,
    /// Current status of the AI upscale model.
    pub upscale_model_status: &'a UpscaleModelStatus,
    /// Whether AI upscaling is enabled for resize operations > 100%.
    pub enable_upscale: bool,
}

impl State {
    /// Create a new editor state for the given image file.
    ///
    /// # Errors
    ///
    /// Returns an error if the image file cannot be opened or read.
    pub fn new(image_path: PathBuf, image: &ImageData) -> Result<Self> {
        let working_image =
            image_rs::open(&image_path).map_err(|err| Error::Io(err.to_string()))?;

        Ok(Self {
            image_source: ImageSource::File(image_path),
            original_image: working_image.clone(),
            current_image: image.clone(),
            working_image,
            active_tool: None,
            transformation_history: Vec::new(),
            history_index: 0,
            sidebar_expanded: true,
            crop: state::CropState::from_image(image),
            crop_modified: false,
            resize: state::ResizeState::from_image(image),
            adjustment: state::AdjustmentState::default(),
            deblur: state::DeblurState::default(),
            crop_base_image: None,
            crop_base_width: image.width,
            crop_base_height: image.height,
            preview_image: None,
            viewport: crate::ui::state::ViewportState::default(),
            export_format: ExportFormat::Png,
            zoom: crate::ui::state::ZoomState::default(),
            cursor_position: None,
            cursor_over_canvas: false,
            drag: crate::ui::state::DragState::default(),
        })
    }

    /// Create a new editor state for a captured video frame.
    ///
    /// # Errors
    ///
    /// Returns an error if the frame cannot be converted to an image.
    pub fn from_captured_frame(
        frame: &ExportableFrame,
        video_path: PathBuf,
        position_secs: f64,
    ) -> Result<Self> {
        let working_image = frame
            .to_dynamic_image()
            .ok_or_else(|| Error::Io("Failed to convert frame to image".to_string()))?;
        let image = frame.to_image_data();

        Ok(Self {
            image_source: ImageSource::CapturedFrame {
                video_path,
                position_secs,
            },
            original_image: working_image.clone(),
            current_image: image.clone(),
            working_image,
            active_tool: None,
            transformation_history: Vec::new(),
            history_index: 0,
            sidebar_expanded: true,
            crop: state::CropState::from_image(&image),
            crop_modified: false,
            resize: state::ResizeState::from_image(&image),
            adjustment: state::AdjustmentState::default(),
            deblur: state::DeblurState::default(),
            crop_base_image: None,
            crop_base_width: image.width,
            crop_base_height: image.height,
            preview_image: None,
            viewport: crate::ui::state::ViewportState::default(),
            export_format: ExportFormat::Png,
            zoom: crate::ui::state::ZoomState::default(),
            cursor_position: None,
            cursor_over_canvas: false,
            drag: crate::ui::state::DragState::default(),
        })
    }

    /// Render the editor view.
    pub fn view<'a>(&'a self, ctx: &ViewContext<'a>) -> Element<'a, Message> {
        view::render(self, ctx)
    }

    pub(crate) fn display_image(&self) -> &ImageData {
        // For resize tool, always show the original image on canvas
        // (preview is shown as thumbnail in sidebar to avoid zoom confusion)
        if self.active_tool == Some(EditorTool::Resize) {
            return &self.current_image;
        }
        self.preview_image.as_ref().unwrap_or(&self.current_image)
    }
}

/// Available editing tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTool {
    Rotate,
    Crop,
    Resize,
    Adjust,
    Deblur,
}

/// Image transformations that can be applied and undone.
///
/// Note: AI transformations (`Deblur`, `UpscaleResize`) store the result image
/// because model inference is non-deterministic and expensive to re-run
/// during undo/redo.
#[derive(Debug, Clone)]
pub enum Transformation {
    RotateLeft,
    RotateRight,
    FlipHorizontal,
    FlipVertical,
    Crop {
        rect: Rectangle,
    },
    Resize {
        width: u32,
        height: u32,
    },
    /// AI upscale resize transformation with cached result for undo/redo.
    UpscaleResize {
        /// The upscaled image result (boxed to keep enum size small).
        result: Box<image_rs::DynamicImage>,
    },
    AdjustBrightness {
        value: i32,
    },
    AdjustContrast {
        value: i32,
    },
    /// AI deblur transformation with cached result for undo/redo.
    Deblur {
        /// The deblurred image result (boxed to keep enum size small).
        result: Box<image_rs::DynamicImage>,
    },
}
