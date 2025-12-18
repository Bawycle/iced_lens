// SPDX-License-Identifier: MPL-2.0
//! Metadata panel component for viewing and editing media file metadata.
//!
//! This module renders a sidebar panel showing EXIF data for images (camera settings,
//! GPS coordinates, etc.) and codec/format information for videos. It supports
//! both view mode (read-only) and edit mode (for modifying EXIF metadata).

pub mod state;
pub mod view;

pub use state::MetadataEditorState;
pub use view::{ViewContext, PANEL_WIDTH};

use crate::i18n::fluent::I18n;
use crate::media::metadata::MediaMetadata;
use std::path::PathBuf;

/// Identifies which metadata field is being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetadataField {
    CameraMake,
    CameraModel,
    DateTaken,
    ExposureTime,
    Aperture,
    Iso,
    Flash,
    FocalLength,
    FocalLength35mm,
    GpsLatitude,
    GpsLongitude,
}

impl MetadataField {
    /// Returns all field variants for iteration.
    pub const fn all() -> &'static [MetadataField] {
        &[
            MetadataField::CameraMake,
            MetadataField::CameraModel,
            MetadataField::DateTaken,
            MetadataField::ExposureTime,
            MetadataField::Aperture,
            MetadataField::Iso,
            MetadataField::FocalLength,
            MetadataField::GpsLatitude,
            MetadataField::GpsLongitude,
        ]
    }

    /// Returns true if this field is a GPS coordinate.
    pub const fn is_gps(&self) -> bool {
        matches!(self, MetadataField::GpsLatitude | MetadataField::GpsLongitude)
    }

    /// Returns the paired GPS field, if this is a GPS field.
    pub const fn gps_pair(&self) -> Option<MetadataField> {
        match self {
            MetadataField::GpsLatitude => Some(MetadataField::GpsLongitude),
            MetadataField::GpsLongitude => Some(MetadataField::GpsLatitude),
            _ => None,
        }
    }
}

/// Messages emitted by the metadata panel.
#[derive(Debug, Clone)]
pub enum Message {
    /// Close the panel.
    Close,
    /// Enter edit mode.
    EnterEditMode,
    /// Exit edit mode without saving.
    ExitEditMode,
    /// A field value has changed.
    FieldChanged(MetadataField, String),
    /// Save changes to the original file.
    Save,
    /// Save changes to a new file.
    SaveAs,
    /// Show a field in the editor (from "Add field" picker).
    ShowField(MetadataField),
    /// Remove/hide a field from the editor (clears value).
    RemoveField(MetadataField),
}

/// Events propagated to the parent application.
#[derive(Debug, Clone)]
pub enum Event {
    /// No action needed.
    None,
    /// Close the panel.
    Close,
    /// Request to enter edit mode (app should create MetadataEditorState).
    EnterEditModeRequested,
    /// Request to exit edit mode (app should clear MetadataEditorState).
    ExitEditModeRequested,
    /// Request to save metadata to the specified path.
    SaveRequested(PathBuf),
    /// Request to open Save As dialog.
    SaveAsRequested,
}

/// Extended context for rendering the metadata panel with edit support.
pub struct PanelContext<'a> {
    pub i18n: &'a I18n,
    pub metadata: Option<&'a MediaMetadata>,
    pub is_dark_theme: bool,
    /// Current file path (needed for save operations).
    pub current_path: Option<&'a PathBuf>,
    /// Editor state when in edit mode.
    pub editor_state: Option<&'a MetadataEditorState>,
    /// Whether the media is an image (edit supported) or video (edit not supported).
    pub is_image: bool,
}

/// Process a metadata panel message and return the corresponding event (new API).
pub fn update_with_state(
    state: Option<&mut MetadataEditorState>,
    message: Message,
    current_path: Option<&PathBuf>,
) -> Event {
    match message {
        Message::Close => Event::Close,
        Message::EnterEditMode => Event::EnterEditModeRequested,
        Message::ExitEditMode => Event::ExitEditModeRequested,
        Message::FieldChanged(field, value) => {
            if let Some(editor) = state {
                editor.set_field(&field, value);
            }
            Event::None
        }
        Message::Save => {
            if let Some(path) = current_path {
                Event::SaveRequested(path.clone())
            } else {
                Event::None
            }
        }
        Message::SaveAs => Event::SaveAsRequested,
        Message::ShowField(field) => {
            if let Some(editor) = state {
                editor.show_field(field);
            }
            Event::None
        }
        Message::RemoveField(field) => {
            if let Some(editor) = state {
                editor.remove_field(field);
            }
            Event::None
        }
    }
}

/// Process a metadata panel message (legacy API for backward compatibility).
pub fn update(message: Message) -> Event {
    match message {
        Message::Close => Event::Close,
        // New messages return None for backward compatibility
        Message::EnterEditMode => Event::EnterEditModeRequested,
        Message::ExitEditMode => Event::ExitEditModeRequested,
        Message::FieldChanged(_, _) => Event::None,
        Message::Save => Event::None,
        Message::SaveAs => Event::SaveAsRequested,
        Message::ShowField(_) => Event::None,
        Message::RemoveField(_) => Event::None,
    }
}

/// Render the metadata panel (new API with edit support).
///
/// This is the main entry point for rendering. It delegates to either
/// view mode or edit mode based on whether editor_state is present.
pub fn panel<'a>(ctx: PanelContext<'a>) -> iced::Element<'a, Message> {
    view::panel(ctx)
}

/// Render the metadata panel (legacy API for backward compatibility).
///
/// This function maintains backward compatibility with the old `ViewContext`.
/// For new code, prefer using `panel()` with `PanelContext`.
pub fn view<'a>(ctx: ViewContext<'a>) -> iced::Element<'a, Message> {
    // Convert legacy ViewContext to new PanelContext
    let is_image = matches!(ctx.metadata, Some(MediaMetadata::Image(_)));
    panel(PanelContext {
        i18n: ctx.i18n,
        metadata: ctx.metadata,
        is_dark_theme: ctx.is_dark_theme,
        current_path: None,
        editor_state: None,
        is_image,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_message_emits_close_event() {
        let event = update_with_state(None, Message::Close, None);
        assert!(matches!(event, Event::Close));
    }

    #[test]
    fn enter_edit_mode_emits_request() {
        let event = update_with_state(None, Message::EnterEditMode, None);
        assert!(matches!(event, Event::EnterEditModeRequested));
    }

    #[test]
    fn exit_edit_mode_emits_request() {
        let event = update_with_state(None, Message::ExitEditMode, None);
        assert!(matches!(event, Event::ExitEditModeRequested));
    }

    #[test]
    fn save_with_path_emits_save_requested() {
        let path = PathBuf::from("/test/image.jpg");
        let event = update_with_state(None, Message::Save, Some(&path));
        assert!(matches!(event, Event::SaveRequested(_)));
    }

    #[test]
    fn save_without_path_emits_none() {
        let event = update_with_state(None, Message::Save, None);
        assert!(matches!(event, Event::None));
    }

    #[test]
    fn save_as_emits_request() {
        let event = update_with_state(None, Message::SaveAs, None);
        assert!(matches!(event, Event::SaveAsRequested));
    }
}
