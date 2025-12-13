// SPDX-License-Identifier: MPL-2.0
//! Top-level messages and runtime flags for the application.

use crate::error::Error;
use crate::media::frame_export::ExportableFrame;
use crate::media::MediaData;
use crate::ui::about;
use crate::ui::help;
use crate::ui::image_editor;
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
}
