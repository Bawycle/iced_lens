// SPDX-License-Identifier: MPL-2.0
//! Top-level messages and runtime flags for the application.

use crate::error::Error;
use crate::media::frame_export::ExportableFrame;
use crate::media::MediaData;
use crate::ui::about;
use crate::ui::diagnostics_screen;
use crate::ui::help;
use crate::ui::image_editor;
use crate::ui::metadata_panel;
use crate::ui::navbar;
use crate::ui::notifications;
use crate::ui::settings;
use crate::ui::viewer::component;
use std::path::PathBuf;
use std::time::Instant;

use super::Screen;

/// Top-level messages consumed by `App::update`. The variants forward
/// lower-level component messages while keeping a single update entrypoint.
#[derive(Debug, Clone)]
pub enum Message {
    Viewer(component::Message),
    SwitchScreen(Screen),
    Settings(settings::Message),
    ImageEditor(image_editor::Message),
    Navbar(navbar::Message),
    Help(help::Message),
    About(about::Message),
    Diagnostics(diagnostics_screen::Message),
    MetadataPanel(metadata_panel::Message),
    Notification(notifications::NotificationMessage),
    ImageEditorLoaded(Result<MediaData, Error>),
    SaveAsDialogResult(Option<PathBuf>),
    FrameCaptureDialogResult {
        path: Option<PathBuf>,
        frame: Option<ExportableFrame>,
    },
    /// Open the image editor with a captured video frame.
    OpenImageEditorWithFrame {
        frame: ExportableFrame,
        video_path: PathBuf,
        position_secs: f64,
    },
    Tick(Instant), // Periodic tick for overlay auto-hide
    /// Trigger the open file dialog from the empty state.
    OpenFileDialog,
    /// Result from the open file dialog.
    OpenFileDialogResult(Option<PathBuf>),
    /// A file was dropped on the window.
    FileDropped(PathBuf),
    /// Result from the metadata Save As dialog.
    MetadataSaveAsDialogResult(Option<PathBuf>),
    /// Progress update during deblur model download (0.0 - 1.0).
    DeblurDownloadProgress(f32),
    /// Result from deblur model download.
    DeblurDownloadCompleted(Result<(), String>),
    /// Result from deblur model validation.
    /// The boolean indicates whether this is a startup validation (true) vs user-initiated (false).
    DeblurValidationCompleted {
        result: Result<(), String>,
        is_startup: bool,
    },
    /// Result from applying AI deblur to an image.
    DeblurApplyCompleted(Result<Box<image_rs::DynamicImage>, String>),
    /// Progress update during upscale model download (0.0 - 1.0).
    UpscaleDownloadProgress(f32),
    /// Result from upscale model download.
    UpscaleDownloadCompleted(Result<(), String>),
    /// Result from upscale model validation.
    /// The boolean indicates whether this is a startup validation (true) vs user-initiated (false).
    UpscaleValidationCompleted {
        result: Result<(), String>,
        is_startup: bool,
    },
    /// Result from applying AI upscale resize to an image.
    UpscaleResizeCompleted(Result<Box<image_rs::DynamicImage>, String>),
    /// Window close was requested (user clicked X or pressed Alt+F4).
    WindowCloseRequested(iced::window::Id),
    /// Result from prefetching an image in the background.
    ImagePrefetched {
        path: PathBuf,
        result: Result<crate::media::ImageData, Error>,
    },
    /// Result from async directory scanning.
    DirectoryScanCompleted {
        /// The scanned media list.
        result: Result<crate::directory_scanner::MediaList, Error>,
        /// The path to load after scanning (if any).
        load_path: Option<PathBuf>,
    },
}

/// Runtime flags passed in from the CLI or launcher to tweak startup behavior.
#[derive(Debug, Default)]
pub struct Flags {
    /// Optional locale override in BCP-47 form (e.g. `fr`, `en-US`).
    pub lang: Option<String>,
    /// Optional image path to preload on startup.
    pub file_path: Option<String>,
    /// Optional directory containing Fluent `.ftl` files for custom builds.
    pub i18n_dir: Option<String>,
    /// Optional data directory override (for state files).
    /// Takes precedence over `ICED_LENS_DATA_DIR` environment variable.
    pub data_dir: Option<String>,
    /// Optional config directory override (for settings.toml).
    /// Takes precedence over `ICED_LENS_CONFIG_DIR` environment variable.
    pub config_dir: Option<String>,
}
