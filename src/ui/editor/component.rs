// SPDX-License-Identifier: MPL-2.0
//! Public-facing view helpers and constructor for the editor facade.

use crate::config::BackgroundTheme;
use crate::error::{Error, Result};
use crate::image_handler::ImageData;
use iced::Element;
use image_rs;
use std::path::PathBuf;

use super::{state, view, Message, State};

/// Contextual data needed to render the editor view.
pub struct ViewContext<'a> {
    pub i18n: &'a crate::i18n::fluent::I18n,
    pub background_theme: BackgroundTheme,
}

impl State {
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
            crop_state: state::CropState::from_image(&image),
            crop_modified: false,
            resize_state: state::ResizeState::from_image(&image),
            crop_base_image: None,
            crop_base_width: image.width,
            crop_base_height: image.height,
            preview_image: None,
        })
    }

    /// Render the editor view.
    pub fn view<'a>(&'a self, ctx: ViewContext<'a>) -> Element<'a, Message> {
        view::render(self, ctx)
    }

    pub(crate) fn display_image(&self) -> &ImageData {
        self.preview_image.as_ref().unwrap_or(&self.current_image)
    }
}
