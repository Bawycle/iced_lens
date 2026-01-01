// SPDX-License-Identifier: MPL-2.0
//! Update logic and message handlers for the application.
//!
//! This module contains the main `update` function and all specialized
//! message handlers for different parts of the application.

use super::{notifications, persistence, Message, Screen};
use crate::config;
use crate::i18n::fluent::I18n;
use crate::media::metadata::MediaMetadata;
use crate::media::{
    self, frame_export::ExportableFrame, MaxSkipAttempts, MediaData, MediaNavigator,
};
use crate::ui::about::{self, Event as AboutEvent};
use crate::ui::design_tokens::sizing;
use crate::ui::help::{self, Event as HelpEvent};
use crate::ui::image_editor::{self, Event as ImageEditorEvent, State as ImageEditorState};
use crate::ui::metadata_panel::{self, Event as MetadataPanelEvent, MetadataEditorState};
use crate::ui::navbar::{self, Event as NavbarEvent};
use crate::ui::settings::{self, Event as SettingsEvent, State as SettingsState};
use crate::ui::theming::ThemeMode;
use crate::ui::viewer::{component, filter_dropdown};
use crate::video_player::KeyboardSeekStep;
// Re-export NavigationDirection from viewer component (single source of truth)
pub use crate::ui::viewer::NavigationDirection;
use iced::{window, Point, Size, Task};
use std::path::PathBuf;

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

/// Handles viewer component messages.
pub fn handle_viewer_message(
    ctx: &mut UpdateContext<'_>,
    message: component::Message,
) -> Task<Message> {
    if let component::Message::RawEvent { window, .. } = &message {
        *ctx.window_id = Some(*window);
    }

    // Check if this is a successful MediaLoaded message to extract metadata
    let is_successful_load = matches!(&message, component::Message::MediaLoaded(Ok(_)));

    let (effect, task) = ctx.viewer.handle_message(message, ctx.i18n);

    // Handle successful media load
    if is_successful_load {
        // Exit metadata edit mode (new media loaded = new context)
        *ctx.metadata_editor_state = None;

        // Use viewer.current_media_path as the source of truth for metadata extraction.
        // This is the path of the media that was just loaded, which is guaranteed to be
        // correct at this point. The navigator may not yet be synchronized (ConfirmNavigation
        // effect is processed later).
        if let Some(path) = ctx.viewer.current_media_path.as_ref() {
            // Extract metadata
            *ctx.current_metadata = media::metadata::extract_metadata(path);

            // Remember the directory for next time and persist
            ctx.persisted.set_last_open_directory_from_file(path);
            if let Some(key) = ctx.persisted.save() {
                ctx.notifications
                    .push(notifications::Notification::warning(&key));
            }
        } else {
            *ctx.current_metadata = None;
        }

        // Clear any stale load error notifications (UX: state consistency)
        ctx.notifications.clear_load_errors();
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
            let mut notification = notifications::Notification::error(key);
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
                        .with_arg("files", files_text)
                        .auto_dismiss(std::time::Duration::from_secs(8)),
                );
            }
            Task::none()
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
                    ctx.notifications.push(notifications::Notification::warning(
                        "notification-video-editing-unsupported",
                    ));
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
                ctx.notifications.push(notifications::Notification::warning(
                    "notification-scan-dir-error",
                ));
            }

            match ImageEditorState::new(image_path, &image_data) {
                Ok(state) => {
                    *ctx.image_editor = Some(state);
                    *ctx.screen = target;
                }
                Err(_) => {
                    ctx.notifications.push(notifications::Notification::error(
                        "notification-editor-create-error",
                    ));
                }
            }
            return Task::none();
        }
        // Can't enter editor screen without an image
        return Task::none();
    }

    // Handle Editor → Viewer transition
    if matches!(target, Screen::Viewer) && matches!(ctx.screen, Screen::ImageEditor) {
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
                ctx.notifications
                    .push(notifications::Notification::warning(&key));
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
                ctx.notifications
                    .push(notifications::Notification::warning(&key));
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

    match editor_state.update(message) {
        ImageEditorEvent::None => Task::none(),
        ImageEditorEvent::ExitEditor => {
            // Get the image source before dropping the editor
            let image_source = editor_state.image_source().clone();

            *ctx.image_editor = None;
            *ctx.screen = Screen::Viewer;

            // For file mode: reload the image in the viewer to show any saved changes
            // For captured frame mode: just return to viewer without reloading
            match image_source {
                image_editor::ImageSource::File(current_media_path) => {
                    // Set loading state via encapsulated method
                    ctx.viewer.start_loading();

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
                    Ok(()) => {
                        ctx.notifications.push(notifications::Notification::success(
                            "notification-save-success",
                        ));
                    }
                    Err(_err) => {
                        ctx.notifications.push(notifications::Notification::error(
                            "notification-save-error",
                        ));
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
            *ctx.screen = Screen::Settings;
            Task::none()
        }
        NavbarEvent::OpenHelp => {
            *ctx.screen = Screen::Help;
            Task::none()
        }
        NavbarEvent::OpenAbout => {
            *ctx.screen = Screen::About;
            Task::none()
        }
        NavbarEvent::EnterEditor => handle_screen_switch(ctx, Screen::ImageEditor),
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
pub fn handle_about_message(ctx: &mut UpdateContext<'_>, message: &about::Message) -> Task<Message> {
    match about::update(message) {
        AboutEvent::None => Task::none(),
        AboutEvent::BackToViewer => {
            *ctx.screen = Screen::Viewer;
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
                    ctx.notifications.push(notifications::Notification::error(
                        "notification-metadata-validation-error",
                    ));
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
                        ctx.notifications.push(notifications::Notification::error(
                            "notification-metadata-save-error",
                        ));
                    }
                }
            }
            Task::none()
        }
        MetadataPanelEvent::SaveAsRequested => {
            // Validate all fields before showing dialog
            if let Some(editor_state) = ctx.metadata_editor_state.as_mut() {
                if !editor_state.validate_all() {
                    ctx.notifications.push(notifications::Notification::error(
                        "notification-metadata-validation-error",
                    ));
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

        // Set loading state via encapsulated method
        ctx.viewer.start_loading();

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
            ctx.notifications.push(notifications::Notification::error(
                "notification-delete-error",
            ));
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

    // Check if it's a directory
    if path.is_dir() {
        // Scan directory for media and load the first file
        let (config, _) = config::load();
        let sort_order = config.display.sort_order.unwrap_or_default();
        if ctx
            .media_navigator
            .scan_from_directory(&path, sort_order)
            .is_ok()
        {
            if let Some(first_path) = ctx
                .media_navigator
                .current_media_path()
                .map(std::path::Path::to_path_buf)
            {
                return load_media_from_path(ctx, first_path);
            }
        }
        // No media found in directory
        ctx.notifications.push(notifications::Notification::warning(
            "notification-empty-dir",
        ));
        return Task::none();
    }

    // Load the media file (last_open_directory is updated on successful load)
    load_media_from_path(ctx, path)
}

/// Internal helper to load media from a path.
fn load_media_from_path(ctx: &mut UpdateContext<'_>, path: PathBuf) -> Task<Message> {
    // Scan the directory for navigation
    let (config, _) = config::load();
    let sort_order = config.display.sort_order.unwrap_or_default();
    let _ = ctx.media_navigator.scan_directory(&path, sort_order);

    // Set up viewer state
    ctx.viewer.current_media_path = Some(path.clone());

    // Set loading state via encapsulated method
    ctx.viewer.start_loading();

    // Load the media
    Task::perform(async move { media::load_media(&path) }, |result| {
        Message::Viewer(component::Message::MediaLoaded(result))
    })
}

/// Handles filter dropdown messages from the viewer.
#[allow(clippy::needless_pass_by_value)] // Message is small and matched/destructured
fn handle_filter_changed(
    ctx: &mut UpdateContext<'_>,
    msg: filter_dropdown::Message,
) -> Task<Message> {
    use crate::media::filter::{DateRangeFilter, MediaFilter};
    use filter_dropdown::DateTarget;

    // Clone current filter to modify
    let mut filter = ctx.media_navigator.filter().clone();

    match msg {
        filter_dropdown::Message::ToggleDropdown
        | filter_dropdown::Message::CloseDropdown
        | filter_dropdown::Message::ConsumeClick
        | filter_dropdown::Message::DateSegmentChanged { .. } => {
            // These are handled locally in the viewer component
            unreachable!("Local messages should be handled in component")
        }
        filter_dropdown::Message::MediaTypeChanged(media_type) => {
            filter.media_type = media_type;
        }
        filter_dropdown::Message::ToggleDateFilter(enabled) => {
            if enabled {
                // Enable date filter with default values (no bounds = filter by field only)
                filter.date_range = Some(DateRangeFilter::default());
            } else {
                filter.date_range = None;
            }
        }
        filter_dropdown::Message::DateFieldChanged(field) => {
            if let Some(ref mut date_range) = filter.date_range {
                date_range.field = field;
            }
        }
        filter_dropdown::Message::DateSubmit(target) => {
            // Get the date from the viewer's dropdown state
            let date_state = ctx.viewer.filter_dropdown_state().date_state(target);
            let date = date_state.to_system_time();

            if let Some(ref mut date_range) = filter.date_range {
                match target {
                    DateTarget::Start => date_range.start = date,
                    DateTarget::End => date_range.end = date,
                }
            }
        }
        filter_dropdown::Message::ClearDate(target) => {
            if let Some(ref mut date_range) = filter.date_range {
                match target {
                    DateTarget::Start => date_range.start = None,
                    DateTarget::End => date_range.end = None,
                }
            }
        }
        filter_dropdown::Message::ResetFilters => {
            filter = MediaFilter::default();
        }
    }

    // Update the navigator's filter
    ctx.media_navigator.set_filter(filter);

    // Persist if filter persistence is enabled
    let (cfg, _) = config::load();
    let should_persist = cfg.display.persist_filters.unwrap_or(false);

    if should_persist {
        persistence::persist_preferences(&mut ctx.preferences_context())
    } else {
        Task::none()
    }
}
