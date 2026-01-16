// SPDX-License-Identifier: MPL-2.0
//! Update logic and message handlers for the application.
//!
//! This module contains the main `update` function and all specialized
//! message handlers for different parts of the application.

use super::{notifications, persistence, Message, Screen};
use crate::config;
use crate::diagnostics::{
    AIModel, AppStateEvent, DiagnosticsHandle, ErrorType, FilterChangeType, NavigationContext,
    UserAction, WarningType,
};
use crate::i18n::fluent::I18n;
use crate::media::metadata::MediaMetadata;
use crate::media::{
    self, frame_export::ExportableFrame, MaxSkipAttempts, MediaData, MediaNavigator,
};
use crate::ui::about::{self, Event as AboutEvent};
use crate::ui::design_tokens::sizing;
use crate::ui::diagnostics_screen::{self, Event as DiagnosticsEvent};
use crate::ui::help::{self, Event as HelpEvent};
use crate::ui::image_editor::{self, Event as ImageEditorEvent, State as ImageEditorState};
use crate::ui::metadata_panel::{self, Event as MetadataPanelEvent, MetadataEditorState};
use crate::ui::navbar::{self, Event as NavbarEvent};
use crate::ui::settings::{self, Event as SettingsEvent, State as SettingsState};
use crate::ui::theming::ThemeMode;
use crate::ui::viewer::{component, controls, filter_dropdown, video_controls};
use crate::video_player::KeyboardSeekStep;
// Re-export NavigationDirection from viewer component (single source of truth)
pub use crate::ui::viewer::NavigationDirection;
use iced::{window, Point, Size, Task};
use std::path::PathBuf;
use std::time::Instant;

/// Navigation mode determines which media types to include.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationMode {
    /// Navigate to any media (images and videos).
    AllMedia,
    /// Navigate only to images (skip videos) - used by editor.
    ImagesOnly,
}

/// Parameters for viewer area validation.
#[allow(clippy::struct_excessive_bools)]
pub struct ViewerAreaParams {
    /// Whether fullscreen mode is active.
    pub is_fullscreen: bool,
    /// Whether the metadata panel is visible.
    pub metadata_panel_visible: bool,
    /// Whether the hamburger menu is open.
    pub menu_open: bool,
    /// Whether the current media is a video (video toolbar visible).
    pub is_video: bool,
    /// Whether the video overflow menu is open (adds extra toolbar height).
    pub overflow_menu_open: bool,
}

/// Checks if a cursor position is within the viewer area (the pane where media is displayed).
///
/// In fullscreen mode, the entire window is considered the viewer area.
/// In windowed mode, excludes (from top to bottom):
/// - Navbar at the top
/// - Hamburger dropdown menu (when open)
/// - Media toolbar (zoom controls) - positioned at top of viewer content
/// - Video toolbar (when showing video) - below media toolbar
/// - Video overflow menu (when open) - below video toolbar
/// - Metadata panel on the right
fn is_in_viewer_area(cursor: Point, window_size: Size, params: &ViewerAreaParams) -> bool {
    if params.is_fullscreen {
        // In fullscreen, the entire window is viewer (overlays float)
        return true;
    }

    // Calculate top exclusion zone
    // In windowed mode, controls are at the TOP of the viewer area (below navbar)
    let mut top_exclusion = sizing::NAVBAR_HEIGHT;

    // Add hamburger menu height if open
    if params.menu_open {
        top_exclusion += sizing::HAMBURGER_MENU_HEIGHT;
    }

    // Media toolbar (zoom controls) is always visible in windowed mode
    top_exclusion += sizing::MEDIA_TOOLBAR_HEIGHT;

    // Video toolbar is only visible when showing a video
    if params.is_video {
        top_exclusion += sizing::VIDEO_TOOLBAR_HEIGHT;
        if params.overflow_menu_open {
            top_exclusion += sizing::VIDEO_TOOLBAR_HEIGHT; // Overflow menu has same height
        }
    }

    // Check if cursor is in the top exclusion zone
    if cursor.y < top_exclusion {
        return false;
    }

    // Check if cursor is in the metadata panel (on the right)
    if params.metadata_panel_visible && cursor.x > window_size.width - sizing::SIDEBAR_WIDTH {
        return false;
    }

    true
}

/// Context for update operations containing mutable references to app state.
pub struct UpdateContext<'a> {
    pub i18n: &'a mut I18n,
    pub screen: &'a mut Screen,
    pub settings: &'a mut SettingsState,
    pub viewer: &'a mut component::State,
    pub image_editor: &'a mut Option<ImageEditorState>,
    pub media_navigator: &'a mut MediaNavigator,
    pub fullscreen: &'a mut bool,
    pub window_id: &'a mut Option<window::Id>,
    pub window_size: &'a Option<iced::Size>,
    pub theme_mode: &'a mut ThemeMode,
    pub video_autoplay: &'a mut bool,
    pub audio_normalization: &'a mut bool,
    pub menu_open: &'a mut bool,
    pub info_panel_open: &'a mut bool,
    pub current_metadata: &'a mut Option<MediaMetadata>,
    pub metadata_editor_state: &'a mut Option<MetadataEditorState>,
    pub help_state: &'a mut help::State,
    pub persisted: &'a mut super::persisted_state::AppState,
    pub notifications: &'a mut notifications::Manager,
    pub cancellation_token: &'a media::deblur::CancellationToken,
    pub prefetch_cache: &'a mut media::prefetch::ImagePrefetchCache,
    /// Handle for logging diagnostic events.
    pub diagnostics: &'a DiagnosticsHandle,
    /// Timestamp when AI deblur operation started (for duration tracking).
    pub deblur_started_at: &'a mut Option<Instant>,
    /// Timestamp when AI upscale operation started (for duration tracking).
    pub upscale_started_at: &'a mut Option<Instant>,
    /// Scale factor for current upscale operation.
    pub upscale_scale_factor: &'a mut Option<f32>,
}

impl UpdateContext<'_> {
    /// Creates a `PreferencesContext` for persisting preferences.
    pub fn preferences_context(&mut self) -> persistence::PreferencesContext<'_> {
        persistence::PreferencesContext {
            viewer: self.viewer,
            settings: self.settings,
            theme_mode: *self.theme_mode,
            video_autoplay: *self.video_autoplay,
            audio_normalization: *self.audio_normalization,
            // Use settings values directly to ensure changes are persisted immediately
            frame_cache_mb: self.settings.frame_cache_mb(),
            frame_history_mb: self.settings.frame_history_mb(),
            keyboard_seek_step_secs: self.settings.keyboard_seek_step_secs(),
            notifications: self.notifications,
            media_navigator: self.media_navigator,
        }
    }
}

/// Logs the `MediaLoadingStarted` diagnostic event when a media load is initiated.
///
/// This should be called from the App layer when starting to load media,
/// following the architectural principle R1: collect at handler level.
pub(super) fn log_media_loading_started(path: &std::path::Path, diagnostics: &DiagnosticsHandle) {
    use crate::diagnostics::MediaType;

    // Detect media type from extension
    let media_type = path.extension().map_or(MediaType::Unknown, |ext| {
        let ext = ext.to_string_lossy().to_lowercase();
        if ["mp4", "webm", "avi", "mkv", "mov", "m4v", "wmv", "flv"].contains(&ext.as_str()) {
            MediaType::Video
        } else {
            MediaType::Image
        }
    });

    // Get file size from filesystem
    let file_size_bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    // Get path metadata (extension, storage_type, path_hash)
    let metadata = diagnostics.media_metadata(path);

    diagnostics.log_state(AppStateEvent::MediaLoadingStarted {
        media_type,
        file_size_bytes,
        dimensions: None, // Not known until load completes
        extension: metadata.extension,
        storage_type: metadata.storage_type,
        path_hash: metadata.path_hash,
    });
}

/// Logs the `MediaLoaded` diagnostic event when media is successfully loaded.
///
/// This should be called from the App layer when media loads successfully,
/// following the architectural principle R1: collect at handler level.
fn log_media_loaded(
    media: &MediaData,
    path: Option<&std::path::Path>,
    diagnostics: &DiagnosticsHandle,
) {
    use crate::diagnostics::{Dimensions, MediaType};

    let (media_type, dimensions) = match media {
        MediaData::Image(img) => (
            MediaType::Image,
            Some(Dimensions::new(img.width, img.height)),
        ),
        MediaData::Video(video_data) => {
            let dims = if video_data.width > 0 && video_data.height > 0 {
                Some(Dimensions::new(video_data.width, video_data.height))
            } else {
                None
            };
            (MediaType::Video, dims)
        }
    };

    let metadata = path.map_or_else(Default::default, |p| diagnostics.media_metadata(p));
    let file_size_bytes = path
        .and_then(|p| std::fs::metadata(p).ok())
        .map_or(0, |m| m.len());

    diagnostics.log_state(AppStateEvent::MediaLoaded {
        media_type,
        file_size_bytes,
        dimensions,
        extension: metadata.extension,
        storage_type: metadata.storage_type,
        path_hash: metadata.path_hash,
    });
}

/// Logs the `MediaFailed` diagnostic event when media loading fails.
///
/// This should be called from the App layer when media loading fails,
/// following the architectural principle R1: collect at handler level.
fn log_media_failed(
    error: &crate::error::Error,
    path: Option<&std::path::Path>,
    diagnostics: &DiagnosticsHandle,
) {
    use crate::diagnostics::MediaType;

    // Detect media type from extension if path is available
    let media_type = path
        .and_then(|p| p.extension())
        .map_or(MediaType::Unknown, |ext| {
            let ext = ext.to_string_lossy().to_lowercase();
            if ["mp4", "webm", "avi", "mkv", "mov", "m4v", "wmv", "flv"].contains(&ext.as_str()) {
                MediaType::Video
            } else {
                MediaType::Image
            }
        });

    let metadata = path.map_or_else(Default::default, |p| diagnostics.media_metadata(p));

    diagnostics.log_state(AppStateEvent::MediaFailed {
        media_type,
        reason: format!("{error}"),
        extension: metadata.extension,
        storage_type: metadata.storage_type,
        path_hash: metadata.path_hash,
    });
}

/// Handles state updates after successful media load.
fn handle_successful_media_load(ctx: &mut UpdateContext<'_>) {
    *ctx.metadata_editor_state = None;
    if let Some(path) = ctx.viewer.current_media_path.as_ref() {
        *ctx.current_metadata = media::metadata::extract_metadata(path);
        ctx.persisted.set_last_open_directory_from_file(path);
        if let Some(key) = ctx.persisted.save() {
            ctx.notifications.push(
                notifications::Notification::warning(&key)
                    .with_warning_type(WarningType::ConfigurationIssue),
            );
        }
    } else {
        *ctx.current_metadata = None;
    }
    ctx.notifications.clear_load_errors();
}

/// Logs diagnostic events for viewer messages at handler level (R1 principle).
///
/// Returns `true` if the message is a successful media load.
fn log_viewer_message_diagnostics(
    message: &component::Message,
    path: Option<&std::path::Path>,
    seek_preview: Option<f64>,
    diagnostics: &DiagnosticsHandle,
) -> bool {
    match message {
        component::Message::MediaLoaded(Ok(media)) => {
            log_media_loaded(media, path, diagnostics);
            true
        }
        component::Message::MediaLoaded(Err(error)) => {
            log_media_failed(error, path, diagnostics);
            false
        }
        component::Message::VideoControls(video_controls::Message::TogglePlayback) => {
            diagnostics.log_action(UserAction::TogglePlayback);
            false
        }
        component::Message::VideoControls(video_controls::Message::SeekCommit) => {
            if let Some(position_secs) = seek_preview {
                diagnostics.log_action(UserAction::SeekVideo { position_secs });
            }
            false
        }
        component::Message::VideoControls(video_controls::Message::StepForward) => {
            diagnostics.log_action(UserAction::StepForward);
            false
        }
        component::Message::VideoControls(video_controls::Message::StepBackward) => {
            diagnostics.log_action(UserAction::StepBackward);
            false
        }
        _ => false,
    }
}

/// Logs video/audio control actions at handler level (R1 principle).
///
/// This function logs actions that require viewer state to capture resulting values
/// (e.g., volume level after `SetVolume`, mute state after `ToggleMute`).
/// Called BEFORE the message is processed, so for toggle actions we compute the resulting state.
fn log_video_audio_action(
    viewer: &component::State,
    message: &video_controls::Message,
    diagnostics: &DiagnosticsHandle,
) {
    match message {
        video_controls::Message::SetVolume(volume) => {
            diagnostics.log_action(UserAction::SetVolume {
                volume: volume.value(),
            });
        }
        video_controls::Message::ToggleMute => {
            // Logging BEFORE processing, so resulting state is opposite of current
            let is_muted = !viewer.video_muted();
            diagnostics.log_action(UserAction::ToggleMute { is_muted });
        }
        video_controls::Message::ToggleLoop => {
            // Logging BEFORE processing, so resulting state is opposite of current
            let is_looping = !viewer.video_loop();
            diagnostics.log_action(UserAction::ToggleLoop { is_looping });
        }
        video_controls::Message::CaptureFrame => {
            // Get video position for timestamp
            let timestamp_secs = viewer.video_position().unwrap_or(0.0);
            diagnostics.log_action(UserAction::CaptureFrame { timestamp_secs });
        }
        // IncreasePlaybackSpeed and DecreasePlaybackSpeed are handled separately
        // because we need the resulting speed value from the player
        _ => {}
    }
}

/// Logs view control actions at handler level (R1 principle).
///
/// This function logs actions AFTER they have been processed, so we can capture
/// the resulting state (e.g., zoom percent after `ZoomIn`, rotation angle after `RotateClockwise`).
fn log_view_control_action(
    viewer: &component::State,
    message: &controls::Message,
    diagnostics: &DiagnosticsHandle,
) {
    // zoom_percent is bounded to 10-800%, safe to cast to u16
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let zoom_percent = viewer.zoom_state().zoom_percent as u16;

    match message {
        controls::Message::ZoomIn => {
            diagnostics.log_action(UserAction::ZoomIn {
                resulting_zoom_percent: zoom_percent,
            });
        }
        controls::Message::ZoomOut => {
            diagnostics.log_action(UserAction::ZoomOut {
                resulting_zoom_percent: zoom_percent,
            });
        }
        controls::Message::ResetZoom => {
            diagnostics.log_action(UserAction::ResetZoom);
        }
        controls::Message::SetFitToWindow(is_fit) => {
            diagnostics.log_action(UserAction::ToggleFitToWindow { is_fit: *is_fit });
        }
        controls::Message::ToggleFullscreen => {
            diagnostics.log_action(UserAction::ToggleFullscreen);
        }
        controls::Message::RotateClockwise => {
            diagnostics.log_action(UserAction::RotateClockwise {
                resulting_angle: viewer.current_rotation().degrees(),
            });
        }
        controls::Message::RotateCounterClockwise => {
            diagnostics.log_action(UserAction::RotateCounterClockwise {
                resulting_angle: viewer.current_rotation().degrees(),
            });
        }
        // ZoomInputChanged and ZoomInputSubmitted are UI state changes, not user actions
        _ => {}
    }
}

/// Logs diagnostic events for editor messages at handler level (R1 principle).
///
/// This function intercepts editor messages BEFORE they are processed by the editor state
/// to capture user intent. We log here rather than in the editor state to maintain
/// separation of concerns (diagnostics belong at the app/handler level).
fn log_editor_action(
    diagnostics: &DiagnosticsHandle,
    editor: &image_editor::State,
    message: &image_editor::Message,
) {
    use image_editor::{SidebarMessage, ToolbarMessage};

    match message {
        // Crop action - capture crop dimensions from current crop state
        image_editor::Message::Sidebar(SidebarMessage::ApplyCrop) => {
            let crop = editor.crop();
            diagnostics.log_action(UserAction::ApplyCrop {
                x: crop.x,
                y: crop.y,
                width: crop.width,
                height: crop.height,
            });
        }

        // Resize action - capture resize parameters
        image_editor::Message::Sidebar(SidebarMessage::ApplyResize) => {
            let resize = editor.resize();
            diagnostics.log_action(UserAction::ApplyResize {
                scale_percent: resize.scale.value(),
                new_width: resize.width,
                new_height: resize.height,
            });
        }

        // Deblur action (intent to apply - state event EditorDeblurStarted tracks actual start)
        image_editor::Message::Sidebar(SidebarMessage::ApplyDeblur) => {
            diagnostics.log_action(UserAction::ApplyDeblur);
        }

        // Undo action - capture what operation is being undone
        image_editor::Message::Sidebar(SidebarMessage::Undo) => {
            let operation_type = editor.undo_operation_type();
            diagnostics.log_action(UserAction::Undo { operation_type });
        }

        // Redo action - capture what operation is being redone
        image_editor::Message::Sidebar(SidebarMessage::Redo) => {
            let operation_type = editor.redo_operation_type();
            diagnostics.log_action(UserAction::Redo { operation_type });
        }

        // Save or Save As action
        image_editor::Message::Sidebar(SidebarMessage::Save | SidebarMessage::SaveAs) => {
            let format = editor.export_format().extension().to_string();
            diagnostics.log_action(UserAction::SaveImage { format });
        }

        // Return to viewer (Cancel/Back button)
        image_editor::Message::Toolbar(ToolbarMessage::BackToViewer) => {
            let had_unsaved_changes = editor.has_unsaved_changes();
            diagnostics.log_action(UserAction::ReturnToViewer {
                had_unsaved_changes,
            });
        }

        // All other messages don't need user action logging
        _ => {}
    }
}

/// Handles viewer component messages.
// Allow too_many_lines: Effect handling in one place improves readability over splitting.
#[allow(clippy::too_many_lines)]
pub fn handle_viewer_message(
    ctx: &mut UpdateContext<'_>,
    message: component::Message,
) -> Task<Message> {
    if let component::Message::RawEvent { window, .. } = &message {
        *ctx.window_id = Some(*window);
    }

    // Log diagnostic events at handler level (R1: collect at handler level)
    let is_successful_load = log_viewer_message_diagnostics(
        &message,
        ctx.viewer.current_media_path.as_deref(),
        ctx.viewer.seek_preview_position(),
        ctx.diagnostics,
    );

    // Log video/audio control actions (requires viewer state for context)
    // Note: Some actions log BEFORE processing (toggles) so we can predict resulting state
    if let component::Message::VideoControls(ref video_msg) = message {
        log_video_audio_action(ctx.viewer, video_msg, ctx.diagnostics);
    }

    // Check if this is a speed change action (need to log AFTER processing)
    let is_speed_change = matches!(
        message,
        component::Message::VideoControls(
            video_controls::Message::IncreasePlaybackSpeed
                | video_controls::Message::DecreasePlaybackSpeed
        )
    );

    // Capture view control message before processing (for logging AFTER)
    let view_control_msg = match &message {
        component::Message::Controls(msg) => Some(msg.clone()),
        _ => None,
    };

    let (effect, task) = ctx
        .viewer
        .handle_message(message, ctx.i18n, ctx.diagnostics);

    // Log speed changes AFTER processing to capture resulting speed
    if is_speed_change {
        if let Some(speed) = ctx.viewer.video_playback_speed() {
            ctx.diagnostics
                .log_action(UserAction::SetPlaybackSpeed { speed });
        }
    }

    // Log view control actions AFTER processing to capture resulting state
    if let Some(ref msg) = view_control_msg {
        log_view_control_action(ctx.viewer, msg, ctx.diagnostics);
    }

    if is_successful_load {
        handle_successful_media_load(ctx);
    }

    let viewer_task = task.map(Message::Viewer);
    let side_effect = match effect {
        component::Effect::PersistPreferences => {
            persistence::persist_preferences(&mut ctx.preferences_context())
        }
        component::Effect::ToggleFullscreen => {
            // Guard: cannot toggle fullscreen when metadata editor has unsaved changes
            let has_unsaved_changes = ctx
                .metadata_editor_state
                .as_ref()
                .is_some_and(crate::ui::metadata_panel::MetadataEditorState::has_changes);
            if has_unsaved_changes {
                Task::none()
            } else {
                toggle_fullscreen(ctx.fullscreen, ctx.window_id.as_ref(), ctx.info_panel_open)
            }
        }
        component::Effect::ExitFullscreen => {
            ctx.diagnostics.log_action(UserAction::ExitFullscreen);
            update_fullscreen_mode(ctx.fullscreen, ctx.window_id.as_ref(), false)
        }
        component::Effect::OpenSettings => {
            *ctx.screen = Screen::Settings;
            Task::none()
        }
        component::Effect::EnterEditor => handle_screen_switch(ctx, Screen::ImageEditor),
        component::Effect::NavigateNext => handle_navigate_next(ctx),
        component::Effect::NavigatePrevious => handle_navigate_previous(ctx),
        component::Effect::CaptureFrame {
            frame,
            video_path,
            position_secs,
        } => handle_capture_frame(frame, video_path, position_secs),
        component::Effect::RequestDelete => handle_delete_current_media(ctx),
        component::Effect::ToggleInfoPanel => {
            *ctx.info_panel_open = !*ctx.info_panel_open;
            Task::none()
        }
        component::Effect::OpenFileDialog => {
            handle_open_file_dialog(ctx.persisted.last_open_directory.clone())
        }
        component::Effect::ShowErrorNotification { key, args } => {
            let mut notification =
                notifications::Notification::error(key).with_error_type(ErrorType::DecodeError);
            for (arg_key, arg_value) in args {
                notification = notification.with_arg(arg_key, arg_value);
            }
            ctx.notifications.push(notification);
            Task::none()
        }
        component::Effect::RetryNavigation {
            direction,
            skip_attempts,
            skipped_files,
        } => handle_retry_navigation(ctx, direction, skip_attempts, skipped_files),
        component::Effect::ShowSkippedFilesNotification { skipped_files } => {
            let files_text = format_skipped_files_message(ctx.i18n, &skipped_files);
            ctx.notifications.push(
                notifications::Notification::warning("notification-skipped-corrupted-files")
                    .with_warning_type(WarningType::UnsupportedFormat)
                    .with_arg("files", files_text)
                    .auto_dismiss(std::time::Duration::from_secs(8)),
            );
            Task::none()
        }
        component::Effect::ConfirmNavigation {
            path,
            skipped_files,
        } => {
            // Confirm navigation position in MediaNavigator
            ctx.media_navigator.confirm_navigation(&path);

            // Show notification if any files were skipped during navigation
            if !skipped_files.is_empty() {
                let files_text = format_skipped_files_message(ctx.i18n, &skipped_files);
                ctx.notifications.push(
                    notifications::Notification::warning("notification-skipped-corrupted-files")
                        .with_warning_type(WarningType::UnsupportedFormat)
                        .with_arg("files", files_text)
                        .auto_dismiss(std::time::Duration::from_secs(8)),
                );
            }

            // Trigger image prefetching for adjacent images
            trigger_prefetch(ctx)
        }
        component::Effect::FilterChanged(filter_msg) => handle_filter_changed(ctx, filter_msg),
        component::Effect::None => Task::none(),
    };
    Task::batch([viewer_task, side_effect])
}

/// Handles screen transitions.
pub fn handle_screen_switch(ctx: &mut UpdateContext<'_>, target: Screen) -> Task<Message> {
    // Guard: cannot enter ImageEditor when metadata editor has unsaved changes
    // Note: Settings/Help/About are safe to navigate to (state is preserved)
    if matches!(ctx.screen, Screen::Viewer) && matches!(target, Screen::ImageEditor) {
        let has_unsaved_changes = ctx
            .metadata_editor_state
            .as_ref()
            .is_some_and(crate::ui::metadata_panel::MetadataEditorState::has_changes);
        if has_unsaved_changes {
            return Task::none();
        }
    }

    // Handle Settings → Viewer transition
    if matches!(target, Screen::Viewer) && matches!(ctx.screen, Screen::Settings) {
        match ctx.settings.ensure_zoom_step_committed() {
            Ok(Some(value)) => {
                ctx.viewer.set_zoom_step_percent(value);
                *ctx.screen = target;
                return persistence::persist_preferences(&mut ctx.preferences_context());
            }
            Ok(None) => {
                *ctx.screen = target;
                return Task::none();
            }
            Err(_) => {
                *ctx.screen = Screen::Settings;
                return Task::none();
            }
        }
    }

    // Handle Viewer → Editor transition
    if matches!(target, Screen::ImageEditor) && matches!(ctx.screen, Screen::Viewer) {
        // Use media_navigator as single source of truth for current path
        if let (Some(image_path), Some(media_data)) = (
            ctx.media_navigator
                .current_media_path()
                .map(std::path::Path::to_path_buf),
            ctx.viewer.media().cloned(),
        ) {
            // Editor only supports images in v0.2, not videos
            let image_data = match media_data {
                MediaData::Image(img) => img,
                MediaData::Video(_) => {
                    ctx.notifications.push(
                        notifications::Notification::warning(
                            "notification-video-editing-unsupported",
                        )
                        .with_warning_type(crate::diagnostics::WarningType::UnsupportedFormat),
                    );
                    return Task::none();
                }
            };

            // Synchronize media_navigator with viewer state before entering editor
            let (config, _) = config::load();
            let sort_order = config.display.sort_order.unwrap_or_default();
            if ctx
                .media_navigator
                .scan_directory(&image_path, sort_order)
                .is_err()
            {
                ctx.notifications.push(
                    notifications::Notification::warning("notification-scan-dir-error")
                        .with_warning_type(WarningType::Other),
                );
            }

            match ImageEditorState::new(image_path, &image_data) {
                Ok(state) => {
                    *ctx.image_editor = Some(state);
                    *ctx.screen = target;

                    // Log editor opened event
                    ctx.diagnostics.log_state(AppStateEvent::EditorOpened {
                        tool: None, // No tool selected initially
                    });

                    // Trigger deferred AI model validation on first editor access
                    let validation_task = trigger_deferred_ai_validation(ctx);
                    return validation_task;
                }
                Err(_) => {
                    ctx.notifications.push(
                        notifications::Notification::error("notification-editor-create-error")
                            .with_error_type(ErrorType::InternalError),
                    );
                }
            }
            return Task::none();
        }
        // Can't enter editor screen without an image
        return Task::none();
    }

    // Handle Editor → Viewer transition
    if matches!(target, Screen::Viewer) && matches!(ctx.screen, Screen::ImageEditor) {
        // Check if editor had unsaved changes before closing
        let had_unsaved_changes = ctx
            .image_editor
            .as_ref()
            .is_some_and(ImageEditorState::has_unsaved_changes);

        ctx.diagnostics.log_state(AppStateEvent::EditorClosed {
            had_unsaved_changes,
        });

        *ctx.image_editor = None;
        *ctx.screen = target;
        return Task::none();
    }

    *ctx.screen = target;
    Task::none()
}

/// Handles settings component messages.
#[allow(clippy::too_many_lines)]
pub fn handle_settings_message(
    ctx: &mut UpdateContext<'_>,
    message: settings::Message,
) -> Task<Message> {
    match ctx.settings.update(message) {
        SettingsEvent::None => Task::none(),
        SettingsEvent::BackToViewer => {
            *ctx.screen = Screen::Viewer;
            Task::none()
        }
        SettingsEvent::BackToViewerWithZoomChange(value) => {
            ctx.viewer.set_zoom_step_percent(value);
            *ctx.screen = Screen::Viewer;
            persistence::persist_preferences(&mut ctx.preferences_context())
        }
        SettingsEvent::LanguageSelected(locale) => {
            persistence::apply_language_change(ctx.i18n, ctx.viewer, &locale, ctx.notifications)
        }
        SettingsEvent::ZoomStepChanged(value) => {
            ctx.viewer.set_zoom_step_percent(value);
            persistence::persist_preferences(&mut ctx.preferences_context())
        }
        SettingsEvent::BackgroundThemeSelected(_)
        | SettingsEvent::SortOrderSelected(_)
        | SettingsEvent::OverlayTimeoutChanged(_)
        | SettingsEvent::FrameCacheMbChanged(_)
        | SettingsEvent::FrameHistoryMbChanged(_)
        | SettingsEvent::DeblurModelUrlChanged(_)
        | SettingsEvent::UpscaleModelUrlChanged(_) => {
            persistence::persist_preferences(&mut ctx.preferences_context())
        }
        SettingsEvent::ThemeModeSelected(mode) => {
            *ctx.theme_mode = mode;
            persistence::persist_preferences(&mut ctx.preferences_context())
        }
        SettingsEvent::VideoAutoplayChanged(enabled) => {
            *ctx.video_autoplay = enabled;
            ctx.viewer.set_video_autoplay(enabled);
            persistence::persist_preferences(&mut ctx.preferences_context())
        }
        SettingsEvent::AudioNormalizationChanged(enabled) => {
            *ctx.audio_normalization = enabled;
            persistence::persist_preferences(&mut ctx.preferences_context())
        }
        SettingsEvent::KeyboardSeekStepChanged(step) => {
            ctx.viewer
                .set_keyboard_seek_step(KeyboardSeekStep::new(step));
            persistence::persist_preferences(&mut ctx.preferences_context())
        }
        SettingsEvent::MaxSkipAttemptsChanged(attempts) => {
            ctx.viewer
                .set_max_skip_attempts(MaxSkipAttempts::new(attempts));
            persistence::persist_preferences(&mut ctx.preferences_context())
        }
        // AI settings events
        SettingsEvent::RequestEnableDeblur => {
            use iced::futures::channel::{mpsc, oneshot};
            use iced::futures::stream;
            use iced::futures::StreamExt;

            // Log state event for diagnostics
            ctx.diagnostics
                .log_state(AppStateEvent::ModelDownloadStarted {
                    model: AIModel::Deblur,
                });

            // Start the download/validation process
            // Set status to downloading and start async task
            ctx.settings
                .set_deblur_model_status(crate::media::deblur::ModelStatus::Downloading {
                    progress: 0.0,
                });

            let url = ctx.settings.deblur_model_url().to_string();

            // Channels for progress and result
            let (progress_tx, progress_rx) = mpsc::channel::<f32>(100);
            let (result_tx, result_rx) = oneshot::channel::<Result<u64, String>>();

            // Spawn the download task
            let url_clone = url.clone();
            tokio::spawn(async move {
                let mut progress_tx = progress_tx;
                let download_result =
                    crate::media::deblur::download_model(&url_clone, |progress| {
                        let _ = progress_tx.try_send(progress);
                    })
                    .await;

                // Send the result through oneshot channel
                let _ = result_tx.send(download_result.map_err(|e| e.to_string()));
                // progress_tx is dropped here, closing the channel
            });

            // State for the stream
            #[allow(clippy::items_after_statements)]
            enum DownloadPhase {
                ReceivingProgress {
                    progress_rx: mpsc::Receiver<f32>,
                    result_rx: oneshot::Receiver<Result<u64, String>>,
                },
                WaitingForResult {
                    result_rx: oneshot::Receiver<Result<u64, String>>,
                },
                Completed,
            }

            let download_stream = stream::unfold(
                DownloadPhase::ReceivingProgress {
                    progress_rx,
                    result_rx,
                },
                |phase| async move {
                    match phase {
                        DownloadPhase::ReceivingProgress {
                            mut progress_rx,
                            result_rx,
                        } => {
                            // Try to receive progress
                            match progress_rx.next().await {
                                Some(progress) => Some((
                                    Message::DeblurDownloadProgress(progress),
                                    DownloadPhase::ReceivingProgress {
                                        progress_rx,
                                        result_rx,
                                    },
                                )),
                                None => {
                                    // Progress channel closed, wait for result
                                    Some((
                                        Message::DeblurDownloadProgress(1.0), // Show 100%
                                        DownloadPhase::WaitingForResult { result_rx },
                                    ))
                                }
                            }
                        }
                        DownloadPhase::WaitingForResult { result_rx } => {
                            // Get the download result
                            match result_rx.await {
                                Ok(Ok(_bytes)) => Some((
                                    Message::DeblurDownloadCompleted(Ok(())),
                                    DownloadPhase::Completed,
                                )),
                                Ok(Err(e)) => Some((
                                    Message::DeblurDownloadCompleted(Err(e)),
                                    DownloadPhase::Completed,
                                )),
                                Err(_) => Some((
                                    Message::DeblurDownloadCompleted(Err(
                                        "Download task cancelled".to_string(),
                                    )),
                                    DownloadPhase::Completed,
                                )),
                            }
                        }
                        DownloadPhase::Completed => None, // Terminate the stream
                    }
                },
            );

            Task::stream(download_stream)
        }
        SettingsEvent::DisableDeblur => {
            // User disabled the feature - persist the state and delete the model
            ctx.persisted.enable_deblur = false;
            if let Some(key) = ctx.persisted.save() {
                ctx.notifications.push(
                    notifications::Notification::warning(&key)
                        .with_warning_type(WarningType::ConfigurationIssue),
                );
            }
            // Delete the model file
            let _ = std::fs::remove_file(crate::media::deblur::get_model_path());
            Task::none()
        }
        // AI Upscale settings events
        SettingsEvent::RequestEnableUpscale => {
            use iced::futures::channel::{mpsc, oneshot};
            use iced::futures::stream;
            use iced::futures::StreamExt;

            // Log state event for diagnostics
            ctx.diagnostics
                .log_state(AppStateEvent::ModelDownloadStarted {
                    model: AIModel::Upscale,
                });

            // Start the download/validation process
            ctx.settings.set_upscale_model_status(
                crate::media::upscale::UpscaleModelStatus::Downloading { progress: 0.0 },
            );

            let url = ctx.settings.upscale_model_url().to_string();

            // Channels for progress and result
            let (progress_tx, progress_rx) = mpsc::channel::<f32>(100);
            let (result_tx, result_rx) = oneshot::channel::<Result<u64, String>>();

            // Spawn the download task
            let url_clone = url.clone();
            tokio::spawn(async move {
                let mut progress_tx = progress_tx;
                let download_result =
                    crate::media::upscale::download_model(&url_clone, |progress| {
                        let _ = progress_tx.try_send(progress);
                    })
                    .await;

                let _ = result_tx.send(download_result.map_err(|e| e.to_string()));
            });

            // State for the stream
            #[allow(clippy::items_after_statements)]
            enum UpscaleDownloadPhase {
                ReceivingProgress {
                    progress_rx: mpsc::Receiver<f32>,
                    result_rx: oneshot::Receiver<Result<u64, String>>,
                },
                WaitingForResult {
                    result_rx: oneshot::Receiver<Result<u64, String>>,
                },
                Completed,
            }

            let download_stream = stream::unfold(
                UpscaleDownloadPhase::ReceivingProgress {
                    progress_rx,
                    result_rx,
                },
                |phase| async move {
                    match phase {
                        UpscaleDownloadPhase::ReceivingProgress {
                            mut progress_rx,
                            result_rx,
                        } => match progress_rx.next().await {
                            Some(progress) => Some((
                                Message::UpscaleDownloadProgress(progress),
                                UpscaleDownloadPhase::ReceivingProgress {
                                    progress_rx,
                                    result_rx,
                                },
                            )),
                            None => Some((
                                Message::UpscaleDownloadProgress(1.0),
                                UpscaleDownloadPhase::WaitingForResult { result_rx },
                            )),
                        },
                        UpscaleDownloadPhase::WaitingForResult { result_rx } => {
                            match result_rx.await {
                                Ok(Ok(_bytes)) => Some((
                                    Message::UpscaleDownloadCompleted(Ok(())),
                                    UpscaleDownloadPhase::Completed,
                                )),
                                Ok(Err(e)) => Some((
                                    Message::UpscaleDownloadCompleted(Err(e)),
                                    UpscaleDownloadPhase::Completed,
                                )),
                                Err(_) => Some((
                                    Message::UpscaleDownloadCompleted(Err(
                                        "Download task cancelled".to_string(),
                                    )),
                                    UpscaleDownloadPhase::Completed,
                                )),
                            }
                        }
                        UpscaleDownloadPhase::Completed => None,
                    }
                },
            );

            Task::stream(download_stream)
        }
        SettingsEvent::DisableUpscale => {
            ctx.persisted.enable_upscale = false;
            if let Some(key) = ctx.persisted.save() {
                ctx.notifications.push(
                    notifications::Notification::warning(&key)
                        .with_warning_type(WarningType::ConfigurationIssue),
                );
            }
            let _ = std::fs::remove_file(crate::media::upscale::get_model_path());
            Task::none()
        }
        SettingsEvent::PersistFiltersChanged(_enabled) => {
            // Setting is already updated in settings state, just persist to config
            persistence::persist_preferences(&mut ctx.preferences_context())
        }
    }
}

/// Handles image editor component messages.
pub fn handle_editor_message(
    ctx: &mut UpdateContext<'_>,
    message: image_editor::Message,
) -> Task<Message> {
    let Some(editor_state) = ctx.image_editor.as_mut() else {
        return Task::none();
    };

    // Log editor user actions at handler level (R1: collect at handler level)
    log_editor_action(ctx.diagnostics, editor_state, &message);

    match editor_state.update(message) {
        ImageEditorEvent::None => Task::none(),
        ImageEditorEvent::ExitEditor => {
            // Get the image source before dropping the editor
            let image_source = editor_state.image_source().clone();

            // Log editor closed event before dropping editor state
            let had_unsaved_changes = editor_state.has_unsaved_changes();
            ctx.diagnostics.log_state(AppStateEvent::EditorClosed {
                had_unsaved_changes,
            });

            *ctx.image_editor = None;
            *ctx.screen = Screen::Viewer;

            // For file mode: reload the image in the viewer to show any saved changes
            // For captured frame mode: just return to viewer without reloading
            match image_source {
                image_editor::ImageSource::File(current_media_path) => {
                    // Set loading state via encapsulated method
                    ctx.viewer.start_loading();

                    // Log MediaLoadingStarted at handler level (R1: collect at handler level)
                    log_media_loading_started(&current_media_path, ctx.diagnostics);

                    // Reload the image in the viewer to show any saved changes
                    Task::perform(
                        async move { media::load_media(&current_media_path) },
                        |result| Message::Viewer(component::Message::MediaLoaded(result)),
                    )
                }
                image_editor::ImageSource::CapturedFrame { .. } => {
                    // Just return to viewer, no need to reload anything
                    Task::none()
                }
            }
        }
        ImageEditorEvent::NavigateNext => handle_editor_navigate_next(ctx),
        ImageEditorEvent::NavigatePrevious => handle_editor_navigate_previous(ctx),
        ImageEditorEvent::SaveRequested { path, overwrite: _ } => {
            // Save the edited image
            if let Some(editor) = ctx.image_editor.as_mut() {
                match editor.save_image(&path) {
                    Ok(result) => {
                        ctx.notifications.push(notifications::Notification::success(
                            "notification-save-success",
                        ));
                        // Show warning if metadata preservation failed
                        if let Some(warning_key) = result.metadata_warning {
                            ctx.notifications.push(
                                notifications::Notification::warning(warning_key)
                                    .with_warning_type(crate::diagnostics::WarningType::MetadataIssue),
                            );
                        }
                    }
                    Err(_err) => {
                        ctx.notifications.push(
                            notifications::Notification::error("notification-save-error")
                                .with_error_type(crate::diagnostics::ErrorType::ExportError),
                        );
                    }
                }
            }
            Task::none()
        }
        ImageEditorEvent::SaveAsRequested => {
            let editor_state = ctx.image_editor.as_ref().expect("editor state exists");
            let last_dir = ctx.persisted.last_save_directory.clone();
            handle_save_as_dialog(editor_state, last_dir)
        }
        ImageEditorEvent::DeblurRequested => handle_deblur_request(ctx),
        ImageEditorEvent::DeblurCancelRequested => {
            // Cancel is handled by the editor state itself (sets cancel_requested flag)
            // The actual inference task will check this flag and stop
            ctx.diagnostics
                .log_state(AppStateEvent::EditorDeblurCancelled);
            Task::none()
        }
        ImageEditorEvent::UpscaleResizeRequested { width, height } => {
            handle_upscale_resize_request(ctx, width, height)
        }
        ImageEditorEvent::ScrollTo { x, y } => {
            use iced::widget::scrollable::RelativeOffset;
            use iced::widget::{operation, Id};
            operation::snap_to(
                Id::new("image-editor-canvas-scrollable"),
                RelativeOffset { x, y },
            )
        }
    }
}

/// Handles the request to apply AI deblur to the current image in the editor.
fn handle_deblur_request(ctx: &mut UpdateContext<'_>) -> Task<Message> {
    let Some(editor_state) = ctx.image_editor.as_ref() else {
        return Task::none();
    };

    // Store start time for duration tracking
    *ctx.deblur_started_at = Some(Instant::now());

    // Log state event for diagnostics
    ctx.diagnostics
        .log_state(AppStateEvent::EditorDeblurStarted);

    // Get the current working image from the editor
    let working_image = editor_state.working_image().clone();

    // Run the deblur inference in a blocking task to avoid blocking the UI
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                let mut manager = crate::media::deblur::DeblurManager::new();
                manager.load_session(None)?; // No cancellation for user-initiated deblur
                manager.deblur(&working_image)
            })
            .await
            .map_err(|e| crate::media::deblur::DeblurError::InferenceFailed(e.to_string()))?
        },
        |result: crate::media::deblur::DeblurResult<image_rs::DynamicImage>| match result {
            Ok(deblurred) => Message::DeblurApplyCompleted(Ok(Box::new(deblurred))),
            Err(e) => Message::DeblurApplyCompleted(Err(e.to_string())),
        },
    )
}

/// Handles resize request that may use AI upscaling.
/// - If AI upscaling is enabled and model is ready: run async AI inference
/// - Otherwise: fall back to standard Lanczos resize
fn handle_upscale_resize_request(
    ctx: &mut UpdateContext<'_>,
    target_width: u32,
    target_height: u32,
) -> Task<Message> {
    let Some(editor_state) = ctx.image_editor.as_mut() else {
        return Task::none();
    };

    // Check if AI upscaling should be used
    let use_ai_upscale = ctx.persisted.enable_upscale
        && matches!(
            ctx.settings.upscale_model_status(),
            media::upscale::UpscaleModelStatus::Ready
        );

    if use_ai_upscale {
        // Get the current working image from the editor
        let working_image = editor_state.working_image().clone();

        // Store start time and calculate scale factor for duration tracking
        *ctx.upscale_started_at = Some(Instant::now());
        // Precision/truncation acceptable for ratio calculation (scale_factor is approximate)
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        let scale_factor = {
            let original_pixels =
                f64::from(working_image.width()) * f64::from(working_image.height());
            let target_pixels = f64::from(target_width) * f64::from(target_height);
            (target_pixels / original_pixels).sqrt() as f32
        };
        *ctx.upscale_scale_factor = Some(scale_factor);

        // Log AI upscale action (R1: collect at handler level)
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        ctx.diagnostics.log_action(UserAction::ApplyUpscale {
            scale_factor: scale_factor.round() as u32,
        });

        // Run the AI upscale + Lanczos resize in a blocking task
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    let mut manager = media::upscale::UpscaleManager::new();
                    manager.load_session(None)?;
                    manager.upscale_to_size(&working_image, target_width, target_height)
                })
                .await
                .map_err(|e| media::upscale::UpscaleError::InferenceFailed(e.to_string()))?
            },
            |result: media::upscale::UpscaleResult<image_rs::DynamicImage>| match result {
                Ok(upscaled) => Message::UpscaleResizeCompleted(Ok(Box::new(upscaled))),
                Err(e) => Message::UpscaleResizeCompleted(Err(e.to_string())),
            },
        )
    } else {
        // Fall back to standard Lanczos resize (sync)
        // Clear the processing state that was set by the event emission
        editor_state.clear_upscale_processing();
        editor_state.sidebar_apply_resize();
        Task::none()
    }
}

/// Handles Save As dialog request.
fn handle_save_as_dialog(
    editor_state: &ImageEditorState,
    last_save_directory: Option<PathBuf>,
) -> Task<Message> {
    use crate::media::frame_export::{generate_default_filename, ExportFormat};

    let image_source = editor_state.image_source().clone();
    let export_format = editor_state.export_format();

    // Get filter based on selected export format
    let (filter_name, filter_ext): (&str, Vec<&str>) = match export_format {
        ExportFormat::Png => ("PNG Image", vec!["png"]),
        ExportFormat::Jpeg => ("JPEG Image", vec!["jpg", "jpeg"]),
        ExportFormat::WebP => ("WebP Image", vec!["webp"]),
    };

    // Generate filename based on image source, with selected format extension
    let filename = match &image_source {
        image_editor::ImageSource::File(path) => {
            // Replace extension with selected format
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
            format!("{}.{}", stem, export_format.extension())
        }
        image_editor::ImageSource::CapturedFrame {
            video_path,
            position_secs,
        } => generate_default_filename(video_path, *position_secs, export_format),
    };

    Task::perform(
        async move {
            let mut dialog = rfd::AsyncFileDialog::new()
                .set_file_name(&filename)
                .add_filter(filter_name, &filter_ext);

            // Use last save directory if available
            if let Some(dir) = last_save_directory {
                if dir.exists() {
                    dialog = dialog.set_directory(&dir);
                }
            }

            dialog.save_file().await.map(|h| h.path().to_path_buf())
        },
        Message::SaveAsDialogResult,
    )
}

/// Handles editor navigation to next image (skips videos).
fn handle_editor_navigate_next(ctx: &mut UpdateContext<'_>) -> Task<Message> {
    let info = ctx.media_navigator.navigation_info();
    ctx.diagnostics.log_action(UserAction::NavigateNext {
        context: NavigationContext::Editor,
        filter_active: false, // Editor ignores filters
        position_in_filtered: None,
        position_in_total: info.current_index.unwrap_or(0),
    });

    // Set load origin for auto-skip on failure
    ctx.viewer.set_navigation_origin(NavigationDirection::Next);
    handle_navigation(
        ctx,
        NavigationDirection::Next,
        NavigationMode::ImagesOnly,
        Message::ImageEditorLoaded,
    )
}

/// Handles editor navigation to previous image (skips videos).
fn handle_editor_navigate_previous(ctx: &mut UpdateContext<'_>) -> Task<Message> {
    let info = ctx.media_navigator.navigation_info();
    ctx.diagnostics.log_action(UserAction::NavigatePrevious {
        context: NavigationContext::Editor,
        filter_active: false, // Editor ignores filters
        position_in_filtered: None,
        position_in_total: info.current_index.unwrap_or(0),
    });

    // Set load origin for auto-skip on failure
    ctx.viewer
        .set_navigation_origin(NavigationDirection::Previous);
    handle_navigation(
        ctx,
        NavigationDirection::Previous,
        NavigationMode::ImagesOnly,
        Message::ImageEditorLoaded,
    )
}

/// Handles navbar component messages.
pub fn handle_navbar_message(
    ctx: &mut UpdateContext<'_>,
    message: navbar::Message,
) -> Task<Message> {
    match navbar::update(message, ctx.menu_open) {
        NavbarEvent::None => Task::none(),
        NavbarEvent::OpenSettings => {
            ctx.diagnostics.log_action(UserAction::OpenSettings);
            *ctx.screen = Screen::Settings;
            Task::none()
        }
        NavbarEvent::OpenHelp => {
            ctx.diagnostics.log_action(UserAction::OpenHelp);
            *ctx.screen = Screen::Help;
            Task::none()
        }
        NavbarEvent::OpenAbout => {
            ctx.diagnostics.log_action(UserAction::OpenAbout);
            *ctx.screen = Screen::About;
            Task::none()
        }
        NavbarEvent::OpenDiagnostics => {
            ctx.diagnostics.log_action(UserAction::OpenDiagnostics);
            *ctx.screen = Screen::Diagnostics;
            Task::none()
        }
        NavbarEvent::EnterEditor => {
            ctx.diagnostics.log_action(UserAction::EnterEditor);
            handle_screen_switch(ctx, Screen::ImageEditor)
        }
        NavbarEvent::ToggleInfoPanel => {
            *ctx.info_panel_open = !*ctx.info_panel_open;
            Task::none()
        }
        NavbarEvent::FilterChanged(filter_msg) => {
            // Route filter messages: local ones to viewer, filter changes to handler
            match filter_msg {
                filter_dropdown::Message::ToggleDropdown
                | filter_dropdown::Message::CloseDropdown
                | filter_dropdown::Message::ConsumeClick
                | filter_dropdown::Message::DateSegmentChanged { .. } => {
                    // These are local dropdown state messages - forward to viewer component
                    let (effect, task) = ctx.viewer.handle_message(
                        component::Message::FilterDropdown(filter_msg),
                        ctx.i18n,
                        ctx.diagnostics,
                    );
                    // Handle any effects from the viewer
                    // Only FilterChanged and None are expected from FilterDropdown messages
                    let effect_task = match effect {
                        component::Effect::FilterChanged(msg) => handle_filter_changed(ctx, msg),
                        _ => Task::none(),
                    };
                    Task::batch([task.map(Message::Viewer), effect_task])
                }
                // All other messages are filter changes - handle directly
                _ => handle_filter_changed(ctx, filter_msg),
            }
        }
    }
}

/// Handles help screen messages.
pub fn handle_help_message(ctx: &mut UpdateContext<'_>, message: help::Message) -> Task<Message> {
    match help::update(ctx.help_state, message) {
        HelpEvent::None => Task::none(),
        HelpEvent::BackToViewer => {
            *ctx.screen = Screen::Viewer;
            Task::none()
        }
    }
}

/// Handles about screen messages.
pub fn handle_about_message(
    ctx: &mut UpdateContext<'_>,
    message: &about::Message,
) -> Task<Message> {
    match about::update(message) {
        AboutEvent::None => Task::none(),
        AboutEvent::BackToViewer => {
            *ctx.screen = Screen::Viewer;
            Task::none()
        }
    }
}

/// Handles diagnostics screen messages.
pub fn handle_diagnostics_message(
    ctx: &mut UpdateContext<'_>,
    message: &diagnostics_screen::Message,
) -> Task<Message> {
    match diagnostics_screen::update(message) {
        DiagnosticsEvent::None => Task::none(),
        DiagnosticsEvent::BackToViewer => {
            *ctx.screen = Screen::Viewer;
            Task::none()
        }
        DiagnosticsEvent::ToggleResourceCollection(_) => {
            // Toggle is handled directly in App::update() since it needs DiagnosticsCollector
            Task::none()
        }
        DiagnosticsEvent::ExportToFile | DiagnosticsEvent::ExportToClipboard => {
            // Export is handled directly in App::update() since it needs DiagnosticsCollector
            Task::none()
        }
    }
}

/// Handles metadata panel messages.
pub fn handle_metadata_panel_message(
    ctx: &mut UpdateContext<'_>,
    message: metadata_panel::Message,
) -> Task<Message> {
    // Use media_navigator as single source of truth for current path
    let current_path = ctx.media_navigator.current_media_path();
    let event = metadata_panel::update_with_state(
        ctx.metadata_editor_state.as_mut(),
        message,
        current_path,
    );

    match event {
        MetadataPanelEvent::None => Task::none(),
        MetadataPanelEvent::Close => {
            // Exit edit mode when closing panel
            *ctx.metadata_editor_state = None;
            *ctx.info_panel_open = false;
            Task::none()
        }
        MetadataPanelEvent::EnterEditModeRequested => {
            // Create editor state from current metadata
            if let Some(MediaMetadata::Image(image_meta)) = ctx.current_metadata.as_ref() {
                *ctx.metadata_editor_state =
                    Some(MetadataEditorState::from_image_metadata(image_meta));
            } else {
                // No image metadata - create empty editor state
                *ctx.metadata_editor_state = Some(MetadataEditorState::new_empty());
            }
            Task::none()
        }
        MetadataPanelEvent::ExitEditModeRequested => {
            *ctx.metadata_editor_state = None;
            Task::none()
        }
        MetadataPanelEvent::SaveRequested(path) => {
            // Validate all fields before saving
            if let Some(editor_state) = ctx.metadata_editor_state.as_mut() {
                if !editor_state.validate_all() {
                    // Validation failed - show error notification
                    ctx.notifications.push(
                        notifications::Notification::error(
                            "notification-metadata-validation-error",
                        )
                        .with_error_type(ErrorType::Other),
                    );
                    return Task::none();
                }

                // Write metadata using little_exif
                match crate::media::metadata_writer::write_exif(
                    &path,
                    editor_state.editable_metadata(),
                ) {
                    Ok(()) => {
                        // Refresh metadata display
                        *ctx.current_metadata = crate::media::metadata::extract_metadata(&path);

                        // Exit edit mode
                        *ctx.metadata_editor_state = None;

                        // Show success notification
                        ctx.notifications.push(notifications::Notification::success(
                            "notification-metadata-save-success",
                        ));
                    }
                    Err(_e) => {
                        // Show error notification
                        ctx.notifications.push(
                            notifications::Notification::error("notification-metadata-save-error")
                                .with_error_type(ErrorType::IoError),
                        );
                    }
                }
            }
            Task::none()
        }
        MetadataPanelEvent::SaveAsRequested => {
            // Validate all fields before showing dialog
            if let Some(editor_state) = ctx.metadata_editor_state.as_mut() {
                if !editor_state.validate_all() {
                    ctx.notifications.push(
                        notifications::Notification::error(
                            "notification-metadata-validation-error",
                        )
                        .with_error_type(ErrorType::Other),
                    );
                    return Task::none();
                }
            }

            // Open Save As dialog
            let mut dialog = rfd::AsyncFileDialog::new().set_title("Save Image As");
            for (name, exts) in crate::media::extensions::IMAGE_SAVE_FILTERS {
                dialog = dialog.add_filter(*name, exts);
            }

            // Set initial directory from app state
            let dialog = if let Some(dir) = ctx.persisted.last_save_directory.as_ref() {
                dialog.set_directory(dir)
            } else {
                dialog
            };

            // Set initial filename from current path (use media_navigator as source of truth)
            let dialog = if let Some(path) = ctx.media_navigator.current_media_path() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    dialog.set_file_name(filename)
                } else {
                    dialog
                }
            } else {
                dialog
            };

            Task::perform(
                async move {
                    dialog
                        .save_file()
                        .await
                        .map(|handle| handle.path().to_path_buf())
                },
                Message::MetadataSaveAsDialogResult,
            )
        }
    }
}

/// Unified navigation handler for viewer and editor.
///
/// This function consolidates all navigation logic (next/previous for viewer/editor)
/// into a single implementation, eliminating code duplication.
///
/// # Arguments
/// * `ctx` - Update context with mutable references to app state
/// * `direction` - Next or Previous
/// * `mode` - `AllMedia` (viewer) or `ImagesOnly` (editor)
/// * `skip_count` - Number of files to skip (0 for normal navigation, >0 for auto-skip retries)
/// * `on_loaded` - Message constructor for the load result
fn handle_navigation_with_skip<F>(
    ctx: &mut UpdateContext<'_>,
    direction: NavigationDirection,
    mode: NavigationMode,
    skip_count: usize,
    on_loaded: F,
) -> Task<Message>
where
    F: FnOnce(Result<MediaData, crate::error::Error>) -> Message + Send + 'static,
{
    // Rescan directory to handle added/removed media (single implementation)
    // Only rescan on initial navigation (skip_count == 0), not on retries
    if skip_count == 0 {
        if let Some(current_path) = ctx
            .media_navigator
            .current_media_path()
            .map(std::path::Path::to_path_buf)
        {
            let (config, _) = config::load();
            let sort_order = config.display.sort_order.unwrap_or_default();
            let _ = ctx
                .media_navigator
                .scan_directory(&current_path, sort_order);
        }
    }

    // Peek based on direction, mode, and skip_count (pessimistic update: don't change position yet)
    // For AllMedia mode, use filtered navigation which respects active filters
    // (automatically falls back to unfiltered when no filter is active)
    let next_path = match (direction, mode) {
        (NavigationDirection::Next, NavigationMode::AllMedia) => {
            ctx.media_navigator.peek_nth_next_filtered(skip_count)
        }
        (NavigationDirection::Previous, NavigationMode::AllMedia) => {
            ctx.media_navigator.peek_nth_previous_filtered(skip_count)
        }
        (NavigationDirection::Next, NavigationMode::ImagesOnly) => {
            ctx.media_navigator.peek_nth_next_image(skip_count)
        }
        (NavigationDirection::Previous, NavigationMode::ImagesOnly) => {
            ctx.media_navigator.peek_nth_previous_image(skip_count)
        }
    };

    if let Some(path) = next_path {
        // Set tentative path in viewer (for error handling and UI feedback).
        // Navigator position is only confirmed after successful load via ConfirmNavigation.
        ctx.viewer.current_media_path = Some(path.clone());

        // Check prefetch cache for images
        if matches!(
            media::detect_media_type(&path),
            Some(media::MediaType::Image)
        ) {
            if let Some(image_data) = ctx.prefetch_cache.get(&path) {
                // Cache hit - return immediately, file size will be read in MediaLoaded handler
                return Task::done(on_loaded(Ok(MediaData::Image(image_data))));
            }
        }

        // Set loading state via encapsulated method
        ctx.viewer.start_loading();

        // Log MediaLoadingStarted at handler level (R1: collect at handler level)
        log_media_loading_started(&path, ctx.diagnostics);

        // Load the media with the provided callback
        Task::perform(async move { media::load_media(&path) }, on_loaded)
    } else {
        Task::none()
    }
}

/// Wrapper for normal navigation (no skip).
fn handle_navigation<F>(
    ctx: &mut UpdateContext<'_>,
    direction: NavigationDirection,
    mode: NavigationMode,
    on_loaded: F,
) -> Task<Message>
where
    F: FnOnce(Result<MediaData, crate::error::Error>) -> Message + Send + 'static,
{
    handle_navigation_with_skip(ctx, direction, mode, 0, on_loaded)
}

/// Handles navigation to next media (images and videos).
pub fn handle_navigate_next(ctx: &mut UpdateContext<'_>) -> Task<Message> {
    let info = ctx.media_navigator.navigation_info();
    ctx.diagnostics.log_action(UserAction::NavigateNext {
        context: NavigationContext::Viewer,
        filter_active: info.filter_active,
        position_in_filtered: if info.filter_active {
            info.current_index
        } else {
            None
        },
        position_in_total: info.current_index.unwrap_or(0),
    });

    // Note: metadata edit mode is exited by MediaLoaded event handler (event-driven)
    // Set load origin for auto-skip on failure
    ctx.viewer.set_navigation_origin(NavigationDirection::Next);
    handle_navigation(
        ctx,
        NavigationDirection::Next,
        NavigationMode::AllMedia,
        |r| Message::Viewer(component::Message::MediaLoaded(r)),
    )
}

/// Handles navigation to previous media (images and videos).
pub fn handle_navigate_previous(ctx: &mut UpdateContext<'_>) -> Task<Message> {
    let info = ctx.media_navigator.navigation_info();
    ctx.diagnostics.log_action(UserAction::NavigatePrevious {
        context: NavigationContext::Viewer,
        filter_active: info.filter_active,
        position_in_filtered: if info.filter_active {
            info.current_index
        } else {
            None
        },
        position_in_total: info.current_index.unwrap_or(0),
    });

    // Note: metadata edit mode is exited by MediaLoaded event handler (event-driven)
    // Set load origin for auto-skip on failure
    ctx.viewer
        .set_navigation_origin(NavigationDirection::Previous);
    handle_navigation(
        ctx,
        NavigationDirection::Previous,
        NavigationMode::AllMedia,
        |r| Message::Viewer(component::Message::MediaLoaded(r)),
    )
}

/// Handles retry navigation after a failed load (auto-skip).
///
/// Continues navigation in the same direction, preserving skip context
/// for grouped notification when max attempts is reached.
///
/// Uses `peek_nth_*` with `skip_attempts` to find the next file without
/// modifying navigator state. The state is only updated via `ConfirmNavigation`
/// after a successful load.
pub fn handle_retry_navigation(
    ctx: &mut UpdateContext<'_>,
    direction: NavigationDirection,
    skip_attempts: u32,
    skipped_files: Vec<String>,
) -> Task<Message> {
    use crate::ui::viewer::LoadOrigin;

    // Set load origin with accumulated skip state
    ctx.viewer.set_load_origin(LoadOrigin::Navigation {
        direction,
        skip_attempts,
        skipped_files,
    });

    // Use skip_attempts as skip_count to peek ahead without modifying navigator state.
    // Navigator position is only confirmed after successful load via ConfirmNavigation.
    handle_navigation_with_skip(
        ctx,
        direction,
        NavigationMode::AllMedia,
        skip_attempts as usize,
        |r| Message::Viewer(component::Message::MediaLoaded(r)),
    )
}

/// Maximum length for a filename in notifications (characters).
const MAX_FILENAME_LEN: usize = 12;

/// Truncates a filename if it exceeds the maximum length.
fn truncate_filename(name: &str) -> String {
    if name.chars().count() <= MAX_FILENAME_LEN {
        name.to_string()
    } else {
        let truncated: String = name.chars().take(MAX_FILENAME_LEN - 1).collect();
        format!("{truncated}…")
    }
}

/// Formats the message for skipped files notification.
///
/// Uses compact format:
/// - 1-2 files: Show all names (truncated if too long)
/// - 3+ files: Show first name + "+X more"
pub fn format_skipped_files_message(i18n: &I18n, skipped_files: &[String]) -> String {
    match skipped_files.len() {
        0 => String::new(),
        1 => truncate_filename(&skipped_files[0]),
        2 => format!(
            "{}, {}",
            truncate_filename(&skipped_files[0]),
            truncate_filename(&skipped_files[1])
        ),
        n => {
            let others = n - 1;
            let others_str = others.to_string();
            let others_text = i18n.tr_with_args(
                "notification-skipped-and-others",
                &[("count", others_str.as_str())],
            );
            format!("{} {}", truncate_filename(&skipped_files[0]), others_text)
        }
    }
}

/// Handles deletion of the current media file.
///
/// Uses `media_navigator` to find the next media to display after deletion.
pub fn handle_delete_current_media(ctx: &mut UpdateContext<'_>) -> Task<Message> {
    let Some(current_path) = ctx
        .media_navigator
        .current_media_path()
        .map(std::path::Path::to_path_buf)
    else {
        return Task::none();
    };

    // Log user action at handler level (R1 principle)
    ctx.diagnostics.log_action(UserAction::DeleteMedia);

    // Get the next candidate before deletion (peek without changing position)
    let has_multiple = ctx.media_navigator.len() > 1;
    let next_candidate = if has_multiple {
        ctx.media_navigator
            .peek_next()
            .filter(|next| *next != current_path)
    } else {
        None
    };

    // Attempt to delete the file
    match std::fs::remove_file(&current_path) {
        Ok(()) => {
            ctx.notifications.push(notifications::Notification::success(
                "notification-delete-success",
            ));

            // Note: metadata edit mode is exited by MediaLoaded event handler (event-driven)

            // Rescan directory after deletion
            let scan_seed = next_candidate
                .clone()
                .unwrap_or_else(|| current_path.clone());

            let (config, _) = config::load();
            let sort_order = config.display.sort_order.unwrap_or_default();
            let _ = ctx.media_navigator.scan_directory(&scan_seed, sort_order);

            if let Some(next_path) = next_candidate {
                // Navigate to the next media
                ctx.media_navigator
                    .set_current_media_path(next_path.clone());
                ctx.viewer.current_media_path = Some(next_path.clone());

                // Set loading state via encapsulated method
                ctx.viewer.start_loading();

                // Log MediaLoadingStarted at handler level (R1: collect at handler level)
                log_media_loading_started(&next_path, ctx.diagnostics);

                Task::perform(async move { media::load_media(&next_path) }, |result| {
                    Message::Viewer(component::Message::MediaLoaded(result))
                })
            } else {
                // No more media in directory - send ClearMedia message to viewer
                // This is event-driven: the viewer handles its own state clearing
                *ctx.metadata_editor_state = None;
                *ctx.current_metadata = None;
                Task::done(Message::Viewer(component::Message::ClearMedia))
            }
        }
        Err(_err) => {
            ctx.notifications.push(
                notifications::Notification::error("notification-delete-error")
                    .with_error_type(ErrorType::IoError),
            );
            Task::none()
        }
    }
}

/// Handles frame capture: opens the editor with the captured frame.
pub fn handle_capture_frame(
    frame: ExportableFrame,
    video_path: PathBuf,
    position_secs: f64,
) -> Task<Message> {
    Task::done(Message::OpenImageEditorWithFrame {
        frame,
        video_path,
        position_secs,
    })
}

/// Toggles fullscreen mode.
/// When entering fullscreen, automatically closes the info panel if it's open.
fn toggle_fullscreen(
    fullscreen: &mut bool,
    window_id: Option<&window::Id>,
    info_panel_open: &mut bool,
) -> Task<Message> {
    let entering_fullscreen = !*fullscreen;
    if entering_fullscreen && *info_panel_open {
        *info_panel_open = false;
    }
    update_fullscreen_mode(fullscreen, window_id, entering_fullscreen)
}

/// Updates fullscreen mode to the desired state.
fn update_fullscreen_mode(
    fullscreen: &mut bool,
    window_id: Option<&window::Id>,
    desired: bool,
) -> Task<Message> {
    if *fullscreen == desired {
        return Task::none();
    }

    let Some(window_id) = window_id else {
        return Task::none();
    };

    *fullscreen = desired;
    let mode = if desired {
        window::Mode::Fullscreen
    } else {
        window::Mode::Windowed
    };
    window::set_mode(*window_id, mode)
}

/// Handles the open file dialog request from empty state.
pub fn handle_open_file_dialog(last_directory: Option<PathBuf>) -> Task<Message> {
    Task::perform(
        async move {
            let mut dialog = rfd::AsyncFileDialog::new()
                .add_filter("Media", crate::media::extensions::ALL_MEDIA_EXTENSIONS);

            if let Some(dir) = last_directory {
                if dir.exists() {
                    dialog = dialog.set_directory(&dir);
                }
            }

            dialog.pick_file().await.map(|h| h.path().to_path_buf())
        },
        Message::OpenFileDialogResult,
    )
}

/// Handles the result of the open file dialog.
pub fn handle_open_file_dialog_result(
    ctx: &mut UpdateContext<'_>,
    path: Option<PathBuf>,
) -> Task<Message> {
    let Some(path) = path else {
        // User cancelled the dialog
        return Task::none();
    };

    ctx.diagnostics.log_action(UserAction::LoadMedia {
        source: Some("file_dialog".to_string()),
    });

    // Load the media (last_open_directory is updated on successful load)
    load_media_from_path(ctx, path)
}

/// Handles a file dropped on the window.
///
/// Only accepts drops within the viewer area (excludes navbar, hamburger menu,
/// toolbars at top, and metadata panel on right). In fullscreen mode, drops are accepted anywhere.
pub fn handle_file_dropped(ctx: &mut UpdateContext<'_>, path: PathBuf) -> Task<Message> {
    // Validate drop position: only accept drops within the viewer area
    if let (Some(cursor), Some(window_size)) = (ctx.viewer.cursor_position(), ctx.window_size) {
        let params = ViewerAreaParams {
            is_fullscreen: *ctx.fullscreen,
            metadata_panel_visible: *ctx.info_panel_open,
            menu_open: *ctx.menu_open,
            is_video: ctx.viewer.is_video(),
            overflow_menu_open: ctx.viewer.is_overflow_menu_open(),
        };
        if !is_in_viewer_area(cursor, *window_size, &params) {
            // Drop occurred outside viewer area - ignore
            return Task::none();
        }
    }
    // If cursor position is unknown, accept the drop (better UX than silent rejection)

    ctx.diagnostics.log_action(UserAction::LoadMedia {
        source: Some("drag_drop".to_string()),
    });

    // Check if it's a directory
    if path.is_dir() {
        // Use async scan for directories
        let (config, _) = config::load();
        let sort_order = config.display.sort_order.unwrap_or_default();
        return Task::perform(
            crate::directory_scanner::scan_directory_direct_async(path, sort_order),
            |result| Message::DirectoryScanCompleted {
                result,
                load_path: None,
            },
        );
    }

    // Load the media file (uses async scan for directory)
    load_media_from_path(ctx, path)
}

/// Internal helper to load media from a path.
///
/// Uses async directory scanning to avoid blocking the UI.
fn load_media_from_path(_ctx: &mut UpdateContext<'_>, path: PathBuf) -> Task<Message> {
    // Use async scan for the directory
    let (config, _) = config::load();
    let sort_order = config.display.sort_order.unwrap_or_default();

    let load_path = path.clone();
    Task::perform(
        crate::directory_scanner::scan_directory_async(path, sort_order),
        move |result| Message::DirectoryScanCompleted {
            result,
            load_path: Some(load_path),
        },
    )
}

/// Handles filter dropdown messages from the viewer.
#[allow(clippy::needless_pass_by_value)] // Message is small and matched/destructured
fn handle_filter_changed(
    ctx: &mut UpdateContext<'_>,
    msg: filter_dropdown::Message,
) -> Task<Message> {
    use crate::media::filter::{DateRangeFilter, MediaFilter};
    use filter_dropdown::DateTarget;

    // === Capture state BEFORE change for diagnostics ===
    let previous_active = ctx.media_navigator.filter().is_active();
    let previous_media_type = ctx.media_navigator.filter().media_type;
    let previous_date_active = ctx.media_navigator.filter().date_range.is_some();

    // Clone current filter to modify
    let mut filter = ctx.media_navigator.filter().clone();

    // Determine filter_type for logging (None for local-only messages and ResetFilters)
    let filter_change_type: Option<FilterChangeType> = match msg {
        filter_dropdown::Message::ToggleDropdown
        | filter_dropdown::Message::CloseDropdown
        | filter_dropdown::Message::ConsumeClick
        | filter_dropdown::Message::DateSegmentChanged { .. } => {
            // These are handled locally in the viewer component
            unreachable!("Local messages should be handled in component")
        }
        filter_dropdown::Message::MediaTypeChanged(media_type) => {
            let from = format!("{previous_media_type:?}").to_lowercase();
            let to = format!("{media_type:?}").to_lowercase();
            filter.media_type = media_type;
            Some(FilterChangeType::MediaType { from, to })
        }
        filter_dropdown::Message::ToggleDateFilter(enabled) => {
            if enabled {
                // Enable date filter with default values (no bounds = filter by field only)
                filter.date_range = Some(DateRangeFilter::default());
                Some(FilterChangeType::DateRangeEnabled)
            } else {
                filter.date_range = None;
                Some(FilterChangeType::DateRangeDisabled)
            }
        }
        filter_dropdown::Message::DateFieldChanged(field) => {
            let field_str = format!("{field:?}").to_lowercase();
            if let Some(ref mut date_range) = filter.date_range {
                date_range.field = field;
            }
            Some(FilterChangeType::DateFieldChanged { field: field_str })
        }
        filter_dropdown::Message::DateSubmit(target) => {
            // Get the date from the viewer's dropdown state
            let date_state = ctx.viewer.filter_dropdown_state().date_state(target);
            let date = date_state.to_system_time();
            let target_str = match target {
                DateTarget::Start => "start".to_string(),
                DateTarget::End => "end".to_string(),
            };

            if let Some(ref mut date_range) = filter.date_range {
                match target {
                    DateTarget::Start => date_range.start = date,
                    DateTarget::End => date_range.end = date,
                }
            }
            Some(FilterChangeType::DateBoundSet { target: target_str })
        }
        filter_dropdown::Message::ClearDate(target) => {
            let target_str = match target {
                DateTarget::Start => "start".to_string(),
                DateTarget::End => "end".to_string(),
            };

            if let Some(ref mut date_range) = filter.date_range {
                match target {
                    DateTarget::Start => date_range.start = None,
                    DateTarget::End => date_range.end = None,
                }
            }
            Some(FilterChangeType::DateBoundCleared { target: target_str })
        }
        filter_dropdown::Message::ResetFilters => {
            // Handle separately as FilterCleared
            let had_media_type_filter = previous_media_type.is_active();
            let had_date_filter = previous_date_active;
            filter = MediaFilter::default();

            // Emit FilterCleared event
            ctx.diagnostics.log_state(AppStateEvent::FilterCleared {
                had_media_type_filter,
                had_date_filter,
            });
            None // Don't emit FilterChanged
        }
    };

    // Update the navigator's filter
    ctx.media_navigator.set_filter(filter);

    // === Emit FilterChanged diagnostic event ===
    if let Some(filter_type) = filter_change_type {
        ctx.diagnostics.log_state(AppStateEvent::FilterChanged {
            filter_type,
            previous_active,
            new_active: ctx.media_navigator.filter().is_active(),
            filtered_count: ctx.media_navigator.filtered_count(),
            total_count: ctx.media_navigator.len(),
        });
    }

    // Persist if filter persistence is enabled
    let (cfg, _) = config::load();
    let should_persist = cfg.display.persist_filters.unwrap_or(false);

    if should_persist {
        persistence::persist_preferences(&mut ctx.preferences_context())
    } else {
        Task::none()
    }
}

/// Triggers deferred AI model validation when the user first enters the image editor.
///
/// This function checks if any AI models are in the `NeedsValidation` state (meaning
/// they were downloaded but validation was deferred from startup) and launches the
/// validation tasks for those models.
///
/// Validation is CPU-intensive (loads ONNX model and runs test inference) so we defer
/// it until the user actually needs the editor, rather than doing it at every app startup.
pub fn trigger_deferred_ai_validation(ctx: &mut UpdateContext<'_>) -> Task<Message> {
    let mut tasks = Vec::new();

    // Check and trigger deblur validation if needed
    if matches!(
        ctx.settings.deblur_model_status(),
        media::deblur::ModelStatus::NeedsValidation
    ) {
        ctx.settings
            .set_deblur_model_status(media::deblur::ModelStatus::Validating);

        let cancel_token = ctx.cancellation_token.clone();
        let deblur_task = Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    let mut manager = media::deblur::DeblurManager::new();
                    manager.load_session(Some(&cancel_token))?;
                    media::deblur::validate_model(&mut manager, Some(&cancel_token))?;
                    Ok::<(), media::deblur::DeblurError>(())
                })
                .await
                .map_err(|e| media::deblur::DeblurError::InferenceFailed(e.to_string()))?
            },
            |result: media::deblur::DeblurResult<()>| match result {
                Ok(()) => Message::DeblurValidationCompleted {
                    result: Ok(()),
                    is_startup: true,
                },
                Err(e) => Message::DeblurValidationCompleted {
                    result: Err(e.to_string()),
                    is_startup: true,
                },
            },
        );
        tasks.push(deblur_task);
    }

    // Check and trigger upscale validation if needed
    if matches!(
        ctx.settings.upscale_model_status(),
        media::upscale::UpscaleModelStatus::NeedsValidation
    ) {
        ctx.settings
            .set_upscale_model_status(media::upscale::UpscaleModelStatus::Validating);

        let cancel_token = ctx.cancellation_token.clone();
        let upscale_task = Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    let mut manager = media::upscale::UpscaleManager::new();
                    manager.load_session(Some(&cancel_token))?;
                    media::upscale::validate_model(&mut manager, Some(&cancel_token))?;
                    Ok::<(), media::upscale::UpscaleError>(())
                })
                .await
                .map_err(|e| media::upscale::UpscaleError::InferenceFailed(e.to_string()))?
            },
            |result: media::upscale::UpscaleResult<()>| match result {
                Ok(()) => Message::UpscaleValidationCompleted {
                    result: Ok(()),
                    is_startup: true,
                },
                Err(e) => Message::UpscaleValidationCompleted {
                    result: Err(e.to_string()),
                    is_startup: true,
                },
            },
        );
        tasks.push(upscale_task);
    }

    if tasks.is_empty() {
        Task::none()
    } else {
        Task::batch(tasks)
    }
}

/// Triggers background prefetching of adjacent images for faster navigation.
///
/// Uses the navigator's `peek_nth_next_filtered` and `peek_nth_previous_filtered` methods
/// to find the N next and N previous images (respecting the active filter).
/// Only images not already in the cache are prefetched.
fn trigger_prefetch(ctx: &mut UpdateContext<'_>) -> Task<Message> {
    if !ctx.prefetch_cache.is_enabled() {
        return Task::none();
    }

    let prefetch_count = ctx.prefetch_cache.prefetch_count();

    // Collect paths to prefetch (N next + N previous images)
    let mut paths_to_check = Vec::with_capacity(prefetch_count * 2);

    // Get next N images (using filtered navigation to respect active filter)
    for i in 0..prefetch_count {
        if let Some(path) = ctx.media_navigator.peek_nth_next_filtered(i) {
            // Only prefetch images, not videos
            if matches!(
                media::detect_media_type(&path),
                Some(media::MediaType::Image)
            ) {
                paths_to_check.push(path);
            }
        }
    }

    // Get previous N images
    for i in 0..prefetch_count {
        if let Some(path) = ctx.media_navigator.peek_nth_previous_filtered(i) {
            // Only prefetch images, not videos
            if matches!(
                media::detect_media_type(&path),
                Some(media::MediaType::Image)
            ) {
                paths_to_check.push(path);
            }
        }
    }

    // Filter out paths already in cache
    let paths_to_prefetch = ctx.prefetch_cache.paths_to_prefetch(&paths_to_check);

    if paths_to_prefetch.is_empty() {
        return Task::none();
    }

    // Create prefetch tasks for each path
    let tasks: Vec<Task<Message>> = paths_to_prefetch
        .into_iter()
        .map(|path| {
            Task::perform(
                media::prefetch::load_image_for_prefetch(path),
                |(path, result)| Message::ImagePrefetched { path, result },
            )
        })
        .collect();

    Task::batch(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::{BufferCapacity, DiagnosticEventKind, DiagnosticsCollector};
    use crate::media::{ImageData, VideoData};
    use std::path::Path;

    /// Creates test image data.
    fn test_image_data() -> ImageData {
        let pixels = vec![255_u8; 4 * 100 * 100];
        ImageData::from_rgba(100, 100, pixels)
    }

    /// Creates test video data.
    fn test_video_data() -> VideoData {
        let pixels = vec![255_u8; 4];
        let thumbnail = ImageData::from_rgba(1, 1, pixels);
        VideoData {
            thumbnail,
            width: 1920,
            height: 1080,
            duration_secs: 60.0,
            fps: 30.0,
            has_audio: true,
        }
    }

    #[test]
    fn log_media_loaded_captures_image_dimensions() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        let media = MediaData::Image(test_image_data());
        log_media_loaded(&media, None, &handle);

        collector.process_pending();
        let events: Vec<_> = collector.iter().collect();
        assert_eq!(events.len(), 1);

        if let DiagnosticEventKind::AppState {
            state: AppStateEvent::MediaLoaded { dimensions, .. },
        } = &events[0].kind
        {
            assert!(dimensions.is_some());
            let dims = dimensions.as_ref().unwrap();
            assert_eq!(dims.width, 100);
            assert_eq!(dims.height, 100);
        } else {
            panic!("Expected MediaLoaded event");
        }
    }

    #[test]
    fn log_media_loaded_captures_video_dimensions() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        let media = MediaData::Video(test_video_data());
        log_media_loaded(&media, None, &handle);

        collector.process_pending();
        let events: Vec<_> = collector.iter().collect();
        assert_eq!(events.len(), 1);

        if let DiagnosticEventKind::AppState {
            state:
                AppStateEvent::MediaLoaded {
                    media_type,
                    dimensions,
                    ..
                },
        } = &events[0].kind
        {
            assert!(matches!(media_type, crate::diagnostics::MediaType::Video));
            assert!(dimensions.is_some());
            let dims = dimensions.as_ref().unwrap();
            assert_eq!(dims.width, 1920);
            assert_eq!(dims.height, 1080);
        } else {
            panic!("Expected MediaLoaded event");
        }
    }

    #[test]
    fn log_media_failed_captures_error_reason() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        let error = crate::error::Error::Config("test config error".to_string());
        log_media_failed(&error, None, &handle);

        collector.process_pending();
        let events: Vec<_> = collector.iter().collect();
        assert_eq!(events.len(), 1);

        if let DiagnosticEventKind::AppState {
            state: AppStateEvent::MediaFailed { reason, .. },
        } = &events[0].kind
        {
            assert!(reason.contains("test config error"));
        } else {
            panic!("Expected MediaFailed event");
        }
    }

    #[test]
    fn log_media_loading_started_detects_video_extension() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        let path = Path::new("/tmp/test_video.mp4");
        log_media_loading_started(path, &handle);

        collector.process_pending();
        let events: Vec<_> = collector.iter().collect();
        assert_eq!(events.len(), 1);

        if let DiagnosticEventKind::AppState {
            state: AppStateEvent::MediaLoadingStarted { media_type, .. },
        } = &events[0].kind
        {
            assert!(matches!(media_type, crate::diagnostics::MediaType::Video));
        } else {
            panic!("Expected MediaLoadingStarted event");
        }
    }

    #[test]
    fn log_media_loading_started_detects_image_extension() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        let path = Path::new("/tmp/test_image.jpg");
        log_media_loading_started(path, &handle);

        collector.process_pending();
        let events: Vec<_> = collector.iter().collect();
        assert_eq!(events.len(), 1);

        if let DiagnosticEventKind::AppState {
            state: AppStateEvent::MediaLoadingStarted { media_type, .. },
        } = &events[0].kind
        {
            assert!(matches!(media_type, crate::diagnostics::MediaType::Image));
        } else {
            panic!("Expected MediaLoadingStarted event");
        }
    }

    #[test]
    fn toggle_playback_logged_from_handler() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        let message = component::Message::VideoControls(video_controls::Message::TogglePlayback);
        let result = log_viewer_message_diagnostics(&message, None, None, &handle);

        assert!(!result); // TogglePlayback doesn't indicate successful load
        collector.process_pending();
        let events: Vec<_> = collector.iter().collect();
        assert_eq!(events.len(), 1);

        if let DiagnosticEventKind::UserAction {
            action: UserAction::TogglePlayback,
            ..
        } = &events[0].kind
        {
            // Test passes - correct action logged
        } else {
            panic!("Expected TogglePlayback action");
        }
    }

    #[test]
    fn seek_video_logged_from_handler() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        let message = component::Message::VideoControls(video_controls::Message::SeekCommit);
        let seek_position = Some(42.5);
        let result = log_viewer_message_diagnostics(&message, None, seek_position, &handle);

        assert!(!result); // SeekCommit doesn't indicate successful load
        collector.process_pending();
        let events: Vec<_> = collector.iter().collect();
        assert_eq!(events.len(), 1);

        if let DiagnosticEventKind::UserAction {
            action: UserAction::SeekVideo { position_secs },
            ..
        } = &events[0].kind
        {
            assert!((*position_secs - 42.5).abs() < f64::EPSILON);
        } else {
            panic!("Expected SeekVideo action");
        }
    }

    #[test]
    fn seek_video_not_logged_without_preview_position() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        let message = component::Message::VideoControls(video_controls::Message::SeekCommit);
        // No seek preview position set
        let result = log_viewer_message_diagnostics(&message, None, None, &handle);

        assert!(!result);
        collector.process_pending();
        let events: Vec<_> = collector.iter().collect();
        assert_eq!(events.len(), 0); // No event logged when no preview position
    }
}
