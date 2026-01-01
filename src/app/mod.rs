// SPDX-License-Identifier: MPL-2.0
//! Application root state and orchestration between the viewer and settings views.
//!
//! The `App` struct wires together the domains (viewer, localization, settings)
//! and translates messages into side effects like config persistence or image
//! loading. This file intentionally keeps policy decisions (minimum window size,
//! persistence format, localization switching) close to the main update loop so
//! it is easy to audit user-facing behavior.

pub mod config;
pub mod i18n;
mod message;
pub mod paths;
pub mod persisted_state;
mod persistence;
mod screen;
mod subscription;
mod update;
mod view;

pub use message::{Flags, Message};
pub use screen::Screen;

use crate::media::metadata::MediaMetadata;
use crate::media::{self, MaxSkipAttempts, MediaData, MediaNavigator};
use crate::ui::help;
use crate::ui::image_editor::{self, State as ImageEditorState};
use crate::ui::metadata_panel::MetadataEditorState;
use crate::ui::notifications;
use crate::ui::settings::{State as SettingsState, StateConfig as SettingsConfig};
use crate::ui::state::zoom::{MAX_ZOOM_STEP_PERCENT, MIN_ZOOM_STEP_PERCENT};
use crate::ui::theming::ThemeMode;
use crate::ui::viewer::component;
use crate::video_player::{create_lufs_cache, SharedLufsCache};
use i18n::fluent::I18n;
use iced::{window, Element, Subscription, Task, Theme};
use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};

// Diagnostic counters for startup hang investigation
static VIEW_CALL_COUNT: AtomicU32 = AtomicU32::new(0);
static UPDATE_CALL_COUNT: AtomicU32 = AtomicU32::new(0);
static SUBSCRIPTION_CALL_COUNT: AtomicU32 = AtomicU32::new(0);

/// Root Iced application state that bridges UI components, localization, and
/// persisted preferences.
// Allow excessive bools: these represent orthogonal application states
// (fullscreen, autoplay, normalization, menu state, info panel, shutdown flag).
// They are independent concerns, not mutually exclusive states for an enum.
#[allow(clippy::struct_excessive_bools)]
pub struct App {
    pub i18n: I18n,
    screen: Screen,
    settings: SettingsState,
    viewer: component::State,
    image_editor: Option<ImageEditorState>,
    media_navigator: MediaNavigator,
    fullscreen: bool,
    window_id: Option<window::Id>,
    /// Current window size for drop zone calculations.
    window_size: Option<iced::Size>,
    theme_mode: ThemeMode,
    /// Whether videos should auto-play when loaded.
    video_autoplay: bool,
    /// Whether audio normalization is enabled for consistent volume levels.
    audio_normalization: bool,
    /// Shared cache for LUFS measurements to avoid re-analyzing files.
    lufs_cache: SharedLufsCache,
    /// Frame cache size in MB for video seek optimization.
    frame_cache_mb: crate::video_player::FrameCacheMb,
    /// Frame history size in MB for backward frame stepping.
    frame_history_mb: crate::video_player::FrameHistoryMb,
    /// Whether the hamburger menu is open.
    menu_open: bool,
    /// Whether the info panel is open.
    info_panel_open: bool,
    /// Current media metadata for the info panel.
    current_metadata: Option<MediaMetadata>,
    /// State for metadata editing mode.
    metadata_editor_state: Option<MetadataEditorState>,
    /// Help screen state (tracks expanded sections).
    help_state: help::State,
    /// Persisted application state (last save directory, etc.).
    persisted: persisted_state::AppState,
    /// Toast notification manager for user feedback.
    notifications: notifications::Manager,
    /// Whether the application is shutting down (used to cancel background tasks).
    shutting_down: bool,
    /// Cancellation token for background tasks (shared with async tasks).
    cancellation_token: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl fmt::Debug for App {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("App")
            .field("screen", &self.screen)
            .field("viewer_has_image", &self.viewer.has_media())
            .finish_non_exhaustive()
    }
}

pub const WINDOW_DEFAULT_HEIGHT: f32 = 650.0;
pub const WINDOW_DEFAULT_WIDTH: f32 = 800.0;
pub const MIN_WINDOW_HEIGHT: f32 = 650.0;
pub const MIN_WINDOW_WIDTH: f32 = 650.0;

/// Ensures zoom step values stay inside the supported range so persisted
/// configs cannot request nonsensical increments.
fn clamp_zoom_step(value: f32) -> f32 {
    value.clamp(MIN_ZOOM_STEP_PERCENT, MAX_ZOOM_STEP_PERCENT)
}

/// Builds the window settings
#[must_use] 
pub fn window_settings_with_locale() -> window::Settings {
    let icon = crate::icon::load_window_icon();

    window::Settings {
        size: iced::Size::new(WINDOW_DEFAULT_WIDTH, WINDOW_DEFAULT_HEIGHT),
        min_size: Some(iced::Size::new(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT)),
        icon,
        ..window::Settings::default()
    }
}

/// Entry point used by `main.rs` to launch the Iced application loop.
///
/// # Errors
///
/// Returns an error if the Iced application fails to initialize or run.
///
/// # Panics
///
/// Panics if called more than once (Iced's boot function is consumed on first call).
pub fn run(flags: Flags) -> iced::Result {
    use std::cell::RefCell;

    // Wrap flags in RefCell<Option<_>> to satisfy Fn trait requirement
    // while only consuming flags once (iced 0.14 requires Fn, not FnOnce)
    let boot_state = RefCell::new(Some(flags));
    let boot = move || {
        let flags = boot_state
            .borrow_mut()
            .take()
            .expect("Boot function called more than once");
        App::new(flags)
    };

    iced::application(boot, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .font(iced_aw::ICED_AW_FONT_BYTES)
        .window(window_settings_with_locale())
        .subscription(App::subscription)
        .run()
}

impl Default for App {
    fn default() -> Self {
        Self {
            i18n: I18n::default(),
            screen: Screen::Viewer,
            settings: SettingsState::default(),
            viewer: component::State::new(),
            image_editor: None,
            media_navigator: MediaNavigator::new(),
            fullscreen: false,
            window_id: None,
            window_size: None,
            theme_mode: ThemeMode::System,
            video_autoplay: false,
            audio_normalization: true, // Enabled by default - normalizes audio volume between media files
            lufs_cache: create_lufs_cache(),
            frame_cache_mb: crate::video_player::FrameCacheMb::default(),
            frame_history_mb: crate::video_player::FrameHistoryMb::default(),
            menu_open: false,
            info_panel_open: false,
            current_metadata: None,
            metadata_editor_state: None,
            help_state: help::State::new(),
            persisted: persisted_state::AppState::default(),
            notifications: notifications::Manager::new(),
            shutting_down: false,
            cancellation_token: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
}

impl App {
    /// Initializes application state and optionally kicks off asynchronous image
    /// loading based on `Flags` received from the launcher.
    // Allow too_many_lines: initialization function with ordered setup steps.
    // Refactoring would risk breaking initialization order and add indirection.
    #[allow(clippy::too_many_lines)]
    fn new(flags: Flags) -> (Self, Task<Message>) {
        let startup_time = std::time::Instant::now();
        eprintln!("[STARTUP] App::new() started");

        let (config, config_warning) = config::load();
        eprintln!("[STARTUP] Config loaded in {:?}", startup_time.elapsed());

        let i18n = I18n::new(flags.lang.clone(), flags.i18n_dir.clone(), &config);
        eprintln!("[STARTUP] I18n loaded in {:?}", startup_time.elapsed());

        let mut app = App {
            i18n,
            ..Self::default()
        };

        app.theme_mode = config.general.theme_mode;

        if let Some(step) = config.display.zoom_step {
            let clamped = clamp_zoom_step(step);
            app.viewer.set_zoom_step_percent(clamped);
        }

        match config.display.fit_to_window {
            Some(true) | None => app.viewer.enable_fit_to_window(),
            Some(false) => app.viewer.disable_fit_to_window(),
        }

        let theme = config.display.background_theme.unwrap_or_default();
        let sort_order = config.display.sort_order.unwrap_or_default();
        let overlay_timeout_secs = config
            .fullscreen
            .overlay_timeout_secs
            .unwrap_or(config::DEFAULT_OVERLAY_TIMEOUT_SECS);
        let video_autoplay = config.video.autoplay.unwrap_or(false);
        let audio_normalization = config.video.audio_normalization.unwrap_or(true);
        let keyboard_seek_step_secs = config
            .video
            .keyboard_seek_step_secs
            .unwrap_or(config::DEFAULT_KEYBOARD_SEEK_STEP_SECS);
        let frame_cache_mb = crate::video_player::FrameCacheMb::new(
            config
                .video
                .frame_cache_mb
                .unwrap_or(config::DEFAULT_FRAME_CACHE_MB),
        );
        let frame_history_mb = crate::video_player::FrameHistoryMb::new(
            config
                .video
                .frame_history_mb
                .unwrap_or(config::DEFAULT_FRAME_HISTORY_MB),
        );
        app.frame_cache_mb = frame_cache_mb;
        app.frame_history_mb = frame_history_mb;
        // Load application state (last save directory, deblur enabled, etc.)
        eprintln!("[STARTUP] Loading app state...");
        let (app_state, state_warning) = persisted_state::AppState::load();
        eprintln!("[STARTUP] App state loaded in {:?}", startup_time.elapsed());

        // Read AI settings before moving app_state (enable flags come from persisted state)
        let enable_deblur = app_state.enable_deblur;
        let enable_upscale = app_state.enable_upscale;

        // Move app_state (no clone needed since we've already extracted the values we need)
        app.persisted = app_state;
        let deblur_model_url = config
            .ai
            .deblur_model_url
            .clone()
            .unwrap_or_else(|| config::DEFAULT_DEBLUR_MODEL_URL.to_string());

        let upscale_model_url = config
            .ai
            .upscale_model_url
            .clone()
            .unwrap_or_else(|| config::DEFAULT_UPSCALE_MODEL_URL.to_string());

        // Check if the deblur model needs validation at startup
        // If enable_deblur is true and model exists, we need to validate it before making it available
        let (deblur_model_status, needs_deblur_startup_validation) =
            if enable_deblur && media::deblur::is_model_downloaded() {
                // Model exists but needs validation - set to Validating, not Ready
                (crate::media::deblur::ModelStatus::Validating, true)
            } else {
                (crate::media::deblur::ModelStatus::NotDownloaded, false)
            };

        // Check if the upscale model needs validation at startup
        let (upscale_model_status, needs_upscale_startup_validation) =
            if enable_upscale && media::upscale::is_model_downloaded() {
                (crate::media::upscale::UpscaleModelStatus::Validating, true)
            } else {
                (
                    crate::media::upscale::UpscaleModelStatus::NotDownloaded,
                    false,
                )
            };

        let max_skip_attempts = config
            .display
            .max_skip_attempts
            .unwrap_or(config::DEFAULT_MAX_SKIP_ATTEMPTS);
        let persist_filters = config.display.persist_filters.unwrap_or(false);
        app.settings = SettingsState::new(SettingsConfig {
            zoom_step_percent: app.viewer.zoom_step_percent(),
            background_theme: theme,
            sort_order,
            overlay_timeout_secs,
            theme_mode: config.general.theme_mode,
            video_autoplay,
            audio_normalization,
            frame_cache_mb: frame_cache_mb.value(),
            frame_history_mb: frame_history_mb.value(),
            keyboard_seek_step_secs,
            max_skip_attempts,
            enable_deblur,
            deblur_model_url,
            deblur_model_status,
            enable_upscale,
            upscale_model_url,
            upscale_model_status,
            persist_filters,
        });
        app.video_autoplay = video_autoplay;
        app.audio_normalization = audio_normalization;
        app.viewer.set_video_autoplay(video_autoplay);
        app.viewer
            .set_keyboard_seek_step(crate::video_player::KeyboardSeekStep::new(
                keyboard_seek_step_secs,
            ));

        // Apply video playback preferences from config
        if let Some(volume) = config.video.volume {
            app.viewer.set_video_volume(volume);
        }
        if let Some(muted) = config.video.muted {
            app.viewer.set_video_muted(muted);
        }
        if let Some(loop_enabled) = config.video.loop_enabled {
            app.viewer.set_video_loop(loop_enabled);
        }

        // Apply display preferences from config
        if let Some(max_skip) = config.display.max_skip_attempts {
            app.viewer
                .set_max_skip_attempts(MaxSkipAttempts::new(max_skip));
        }

        // Restore persisted filter if enabled
        if persist_filters {
            if let Some(filter) = config.display.filter {
                app.media_navigator.set_filter(filter);
            }
        }

        // Show warnings for config/state loading issues
        if let Some(key) = config_warning {
            app.notifications
                .push(notifications::Notification::warning(&key));
        }
        if let Some(key) = state_warning {
            app.notifications
                .push(notifications::Notification::warning(&key));
        }

        eprintln!("[STARTUP] Settings applied in {:?}", startup_time.elapsed());

        let task = if let Some(path_str) = flags.file_path {
            let path = std::path::PathBuf::from(&path_str);
            eprintln!("[STARTUP] Scanning directory for: {}", path.display());

            // Determine if path is a directory or a file and resolve the media path
            let resolved_path = if path.is_dir() {
                // Directory path: scan for media files and select the first one
                match app.media_navigator.scan_from_directory(&path, sort_order) {
                    Ok(Some(first_media)) => Some(first_media),
                    Ok(None) => {
                        // No media files found in directory - start without media
                        None
                    }
                    Err(_) => {
                        app.notifications.push(notifications::Notification::warning(
                            "notification-scan-dir-error",
                        ));
                        None
                    }
                }
            } else {
                // File path: use existing behavior
                if app
                    .media_navigator
                    .scan_directory(&path, sort_order)
                    .is_err()
                {
                    app.notifications.push(notifications::Notification::warning(
                        "notification-scan-dir-error",
                    ));
                }
                Some(path)
            };

            eprintln!(
                "[STARTUP] Directory scanned in {:?}",
                startup_time.elapsed()
            );

            if let Some(media_path) = resolved_path {
                // Synchronize navigator state (single source of truth for current media)
                app.media_navigator.set_current_media_path(media_path.clone());

                // Synchronize viewer state
                app.viewer.current_media_path = Some(media_path.clone());

                // Set loading state via encapsulated method
                app.viewer.start_loading();

                // Load the media
                let path_string = media_path.to_string_lossy().into_owned();
                Task::perform(async move { media::load_media(&path_string) }, |result| {
                    Message::Viewer(component::Message::MediaLoaded(result))
                })
            } else {
                Task::none()
            }
        } else {
            Task::none()
        };

        eprintln!(
            "[STARTUP] Media task prepared in {:?}, deblur_validation={}, upscale_validation={}",
            startup_time.elapsed(),
            needs_deblur_startup_validation,
            needs_upscale_startup_validation
        );

        // If deblur was enabled and model exists, start validation in background
        // Use spawn_blocking to avoid blocking the tokio runtime during CPU-intensive ONNX inference
        let deblur_validation_task = if needs_deblur_startup_validation {
            let cancel_token = app.cancellation_token.clone();
            Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        eprintln!("[STARTUP] Deblur validation: starting...");
                        let mut manager = media::deblur::DeblurManager::new();
                        eprintln!("[STARTUP] Deblur validation: loading session...");
                        manager.load_session(Some(&cancel_token))?;
                        eprintln!("[STARTUP] Deblur validation: validating model...");
                        media::deblur::validate_model(&mut manager, Some(&cancel_token))?;
                        eprintln!("[STARTUP] Deblur validation: completed");
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
            )
        } else {
            Task::none()
        };

        // If upscale was enabled and model exists, start validation in background
        let upscale_validation_task = if needs_upscale_startup_validation {
            let cancel_token = app.cancellation_token.clone();
            Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        eprintln!("[STARTUP] Upscale validation: starting...");
                        let mut manager = media::upscale::UpscaleManager::new();
                        eprintln!("[STARTUP] Upscale validation: loading session...");
                        manager.load_session(Some(&cancel_token))?;
                        eprintln!("[STARTUP] Upscale validation: validating model...");
                        media::upscale::validate_model(&mut manager, Some(&cancel_token))?;
                        eprintln!("[STARTUP] Upscale validation: completed");
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
            )
        } else {
            Task::none()
        };

        // Combine tasks
        let combined_task = Task::batch([task, deblur_validation_task, upscale_validation_task]);

        eprintln!(
            "[STARTUP] App::new() completed in {:?}",
            startup_time.elapsed()
        );
        (app, combined_task)
    }

    fn title(&self) -> String {
        let app_name = self.i18n.tr("window-title");

        match self.get_display_title() {
            Some(title) => {
                if self.has_any_unsaved_changes() {
                    format!("*{title} - {app_name}")
                } else {
                    format!("{title} - {app_name}")
                }
            }
            None => app_name,
        }
    }

    /// Gets the display title for the current context.
    ///
    /// Priority order:
    /// 1. Captured frame → "New Image" (i18n)
    /// 2. Dublin Core title (dc:title) from metadata
    /// 3. Filename from media navigator
    fn get_display_title(&self) -> Option<String> {
        // Captured frame: use localized "New Image" title
        if self.is_editing_captured_frame() {
            return Some(self.i18n.tr("new-image-title"));
        }

        // Try dc:title from Dublin Core metadata
        if let Some(media::metadata::MediaMetadata::Image(image_meta)) =
            self.current_metadata.as_ref()
        {
            if let Some(ref dc_title) = image_meta.dc_title {
                if !dc_title.is_empty() {
                    return Some(dc_title.clone());
                }
            }
        }

        // Fall back to filename (media_navigator as single source of truth)
        self.media_navigator.current_media_path().and_then(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(String::from)
        })
    }

    /// Checks if currently editing a captured video frame (no source file).
    fn is_editing_captured_frame(&self) -> bool {
        self.image_editor
            .as_ref()
            .is_some_and(crate::ui::image_editor::State::is_captured_frame)
    }

    /// Checks if any domain has unsaved changes.
    ///
    /// Aggregates unsaved state from:
    /// - Image editor (transformations)
    /// - Metadata editor (metadata changes)
    ///
    /// Note: Captured frames never show the unsaved indicator since they
    /// are conceptually new documents, not modified existing files.
    fn has_any_unsaved_changes(&self) -> bool {
        // Captured frames don't show unsaved indicator
        if self.is_editing_captured_frame() {
            return false;
        }

        // Check image editor
        let image_editor_changes = self
            .image_editor
            .as_ref()
            .is_some_and(crate::ui::image_editor::State::has_unsaved_changes);

        // Check metadata editor
        let metadata_editor_changes = self
            .metadata_editor_state
            .as_ref()
            .is_some_and(crate::ui::metadata_panel::MetadataEditorState::has_changes);

        image_editor_changes || metadata_editor_changes
    }

    fn theme(&self) -> Theme {
        match self.theme_mode {
            ThemeMode::Light => Theme::Light,
            ThemeMode::Dark | ThemeMode::System => Theme::Dark,
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let count = SUBSCRIPTION_CALL_COUNT.fetch_add(1, Ordering::Relaxed);
        if count < 5 {
            eprintln!("[STARTUP] subscription() call #{}", count + 1);
        }

        let event_sub = subscription::create_event_subscription(self.screen);
        let tick_sub = subscription::create_tick_subscription(
            self.fullscreen,
            self.viewer.is_loading_media(),
            self.notifications.has_notifications(),
        );
        let video_sub = subscription::create_video_subscription(
            &self.viewer,
            Some(self.lufs_cache.clone()),
            self.audio_normalization,
            self.frame_cache_mb.value(),
            self.settings.frame_history_mb(),
        );

        // Editor subscription for spinner animation during deblur processing
        let editor_sub = self
            .image_editor
            .as_ref()
            .map_or_else(Subscription::none, |editor| {
                editor.subscription().map(Message::ImageEditor)
            });

        Subscription::batch([event_sub, tick_sub, video_sub, editor_sub])
    }

    // Allow too_many_lines: match dispatcher inherent to Elm architecture.
    // Length comes from number of message variants, not from complexity.
    #[allow(clippy::too_many_lines)]
    fn update(&mut self, message: Message) -> Task<Message> {
        let count = UPDATE_CALL_COUNT.fetch_add(1, Ordering::Relaxed);
        if count < 10 {
            eprintln!("[STARTUP] update() call #{}: {:?}", count + 1, std::mem::discriminant(&message));
        }

        // Track window size from resize events before creating context
        // (must be done before borrowing self.window_size)
        if let Message::Viewer(component::Message::RawEvent {
            event: iced::event::Event::Window(iced::window::Event::Resized(size)),
            ..
        }) = &message
        {
            self.window_size = Some(*size);
        }

        let mut ctx = update::UpdateContext {
            i18n: &mut self.i18n,
            screen: &mut self.screen,
            settings: &mut self.settings,
            viewer: &mut self.viewer,
            image_editor: &mut self.image_editor,
            media_navigator: &mut self.media_navigator,
            fullscreen: &mut self.fullscreen,
            window_id: &mut self.window_id,
            window_size: &self.window_size,
            theme_mode: &mut self.theme_mode,
            video_autoplay: &mut self.video_autoplay,
            audio_normalization: &mut self.audio_normalization,
            menu_open: &mut self.menu_open,
            info_panel_open: &mut self.info_panel_open,
            current_metadata: &mut self.current_metadata,
            metadata_editor_state: &mut self.metadata_editor_state,
            help_state: &mut self.help_state,
            persisted: &mut self.persisted,
            notifications: &mut self.notifications,
        };

        match message {
            Message::Viewer(viewer_message) => {
                update::handle_viewer_message(&mut ctx, viewer_message)
            }
            Message::SwitchScreen(target) => update::handle_screen_switch(&mut ctx, target),
            Message::Settings(settings_message) => {
                update::handle_settings_message(&mut ctx, settings_message)
            }
            Message::ImageEditor(editor_message) => {
                update::handle_editor_message(&mut ctx, editor_message)
            }
            Message::Navbar(navbar_message) => {
                update::handle_navbar_message(&mut ctx, navbar_message)
            }
            Message::Help(help_message) => update::handle_help_message(&mut ctx, help_message),
            Message::About(about_message) => update::handle_about_message(&mut ctx, &about_message),
            Message::MetadataPanel(panel_message) => {
                update::handle_metadata_panel_message(&mut ctx, panel_message)
            }
            Message::Notification(notification_message) => {
                self.notifications.handle_message(&notification_message);
                Task::none()
            }
            Message::ImageEditorLoaded(result) => self.handle_image_editor_loaded(result),
            Message::Tick(_instant) => {
                // Periodic tick for overlay auto-hide - just trigger a view refresh
                // The view() function will check elapsed time and hide controls if needed

                // Also check for loading timeout
                if self.viewer.check_loading_timeout() {
                    self.notifications.push(notifications::Notification::error(
                        "notification-load-error-timeout",
                    ));
                }

                // Tick notification manager to handle auto-dismiss
                self.notifications.tick();

                Task::none()
            }
            Message::SaveAsDialogResult(path_opt) => {
                if let Some(path) = path_opt {
                    // User selected a path, save the image there
                    if let Some(editor) = self.image_editor.as_mut() {
                        match editor.save_image(&path) {
                            Ok(()) => {
                                self.notifications
                                    .push(notifications::Notification::success(
                                        "notification-save-success",
                                    ));

                                // Remember the save directory for next time
                                self.persisted.set_last_save_directory_from_file(&path);
                                if let Some(key) = self.persisted.save() {
                                    self.notifications
                                        .push(notifications::Notification::warning(&key));
                                }

                                // Rescan directory if saved in the same folder as current media
                                persistence::rescan_directory_if_same(
                                    &mut self.media_navigator,
                                    &path,
                                );
                            }
                            Err(_err) => {
                                self.notifications.push(notifications::Notification::error(
                                    "notification-save-error",
                                ));
                            }
                        }
                    }
                }
                // User cancelled or error occurred, do nothing
                Task::none()
            }
            Message::FrameCaptureDialogResult { path, frame } => {
                if let (Some(path), Some(frame)) = (path, frame) {
                    // Determine export format from file extension
                    let format = crate::media::frame_export::ExportFormat::from_path(&path);

                    match frame.save_to_file(&path, format) {
                        Ok(()) => {
                            self.notifications
                                .push(notifications::Notification::success(
                                    "notification-frame-capture-success",
                                ));

                            // Remember the save directory for next time
                            self.persisted.set_last_save_directory_from_file(&path);
                            if let Some(key) = self.persisted.save() {
                                self.notifications
                                    .push(notifications::Notification::warning(&key));
                            }
                        }
                        Err(_err) => {
                            self.notifications.push(notifications::Notification::error(
                                "notification-frame-capture-error",
                            ));
                        }
                    }
                }
                Task::none()
            }
            Message::OpenImageEditorWithFrame {
                frame,
                video_path,
                position_secs,
            } => {
                match ImageEditorState::from_captured_frame(&frame, video_path, position_secs) {
                    Ok(state) => {
                        self.image_editor = Some(state);
                        self.screen = Screen::ImageEditor;
                    }
                    Err(_) => {
                        self.notifications.push(notifications::Notification::error(
                            "notification-editor-frame-error",
                        ));
                    }
                }
                Task::none()
            }
            Message::OpenFileDialog => {
                update::handle_open_file_dialog(self.persisted.last_open_directory.clone())
            }
            Message::OpenFileDialogResult(path) => {
                update::handle_open_file_dialog_result(&mut ctx, path)
            }
            Message::FileDropped(path) => update::handle_file_dropped(&mut ctx, path),
            Message::MetadataSaveAsDialogResult(path_opt) => {
                if let Some(path) = path_opt {
                    self.handle_metadata_save_as(&path)
                } else {
                    Task::none()
                }
            }
            Message::DeblurDownloadProgress(progress) => {
                self.settings
                    .set_deblur_model_status(media::deblur::ModelStatus::Downloading { progress });
                Task::none()
            }
            Message::DeblurDownloadCompleted(result) => {
                self.handle_deblur_download_completed(result)
            }
            Message::DeblurValidationCompleted { result, is_startup } => {
                self.handle_deblur_validation_completed(result, is_startup)
            }
            Message::DeblurApplyCompleted(result) => self.handle_deblur_apply_completed(result),
            Message::UpscaleDownloadProgress(progress) => {
                self.settings.set_upscale_model_status(
                    media::upscale::UpscaleModelStatus::Downloading { progress },
                );
                Task::none()
            }
            Message::UpscaleDownloadCompleted(result) => {
                self.handle_upscale_download_completed(result)
            }
            Message::UpscaleValidationCompleted { result, is_startup } => {
                self.handle_upscale_validation_completed(result, is_startup)
            }
            Message::UpscaleResizeCompleted(result) => self.handle_upscale_resize_completed(result),
            Message::WindowCloseRequested(id) => {
                // Mark app as shutting down to cancel background tasks
                self.shutting_down = true;
                // Signal cancellation to background tasks
                self.cancellation_token
                    .store(true, std::sync::atomic::Ordering::SeqCst);
                // Close the window
                window::close(id)
            }
        }
    }

    /// Handles the result of applying AI deblur to an image.
    fn handle_deblur_apply_completed(
        &mut self,
        result: Result<Box<image_rs::DynamicImage>, String>,
    ) -> Task<Message> {
        // Ignore results if shutting down
        if self.shutting_down {
            return Task::none();
        }

        if let Some(editor) = self.image_editor.as_mut() {
            match result {
                Ok(deblurred_image) => {
                    editor.apply_deblur_result(*deblurred_image);
                    self.notifications
                        .push(notifications::Notification::success(
                            "notification-deblur-apply-success",
                        ));
                }
                Err(e) => {
                    editor.deblur_failed();
                    self.notifications.push(
                        notifications::Notification::error("notification-deblur-apply-error")
                            .with_arg("error", e),
                    );
                }
            }
        }
        Task::none()
    }

    /// Handles the result of applying AI upscale resize to an image.
    fn handle_upscale_resize_completed(
        &mut self,
        result: Result<Box<image_rs::DynamicImage>, String>,
    ) -> Task<Message> {
        // Ignore results if shutting down
        if self.shutting_down {
            return Task::none();
        }

        if let Some(editor) = self.image_editor.as_mut() {
            match result {
                Ok(upscaled_image) => {
                    // apply_upscale_resize_result clears the processing state
                    editor.apply_upscale_resize_result(*upscaled_image);
                    self.notifications
                        .push(notifications::Notification::success(
                            "notification-upscale-resize-success",
                        ));
                }
                Err(e) => {
                    // Clear processing state on error
                    editor.clear_upscale_processing();
                    self.notifications.push(
                        notifications::Notification::error("notification-upscale-resize-error")
                            .with_arg("error", e),
                    );
                }
            }
        }
        Task::none()
    }

    /// Handles the metadata Save As dialog result.
    fn handle_metadata_save_as(&mut self, path: &std::path::Path) -> Task<Message> {
        use crate::media::metadata_writer;

        // First, copy the original file to the new location
        // Use media_navigator as single source of truth for current path
        if let Some(source_path) = self.media_navigator.current_media_path() {
            if let Err(_e) = std::fs::copy(source_path, path) {
                self.notifications.push(notifications::Notification::error(
                    "notification-metadata-save-error",
                ));
                return Task::none();
            }
        } else {
            self.notifications.push(notifications::Notification::error(
                "notification-metadata-save-error",
            ));
            return Task::none();
        }

        // Then write metadata to the new file
        if let Some(editor_state) = self.metadata_editor_state.as_ref() {
            match metadata_writer::write_exif(path, editor_state.editable_metadata()) {
                Ok(()) => {
                    // Remember the save directory
                    self.persisted.set_last_save_directory_from_file(path);
                    if let Some(key) = self.persisted.save() {
                        self.notifications
                            .push(notifications::Notification::warning(&key));
                    }

                    // Refresh metadata display
                    self.current_metadata = media::metadata::extract_metadata(path);

                    // Exit edit mode
                    self.metadata_editor_state = None;

                    // Show success notification
                    self.notifications
                        .push(notifications::Notification::success(
                            "notification-metadata-save-success",
                        ));
                }
                Err(_e) => {
                    // Clean up: remove the copied file if write failed
                    let _ = std::fs::remove_file(path);
                    self.notifications.push(notifications::Notification::error(
                        "notification-metadata-save-error",
                    ));
                }
            }
        }
        Task::none()
    }

    /// Handles the result of deblur model download.
    fn handle_deblur_download_completed(&mut self, result: Result<(), String>) -> Task<Message> {
        // Don't start validation if shutting down
        if self.shutting_down {
            return Task::none();
        }

        match result {
            Ok(()) => {
                // Download succeeded - start validation
                self.settings
                    .set_deblur_model_status(media::deblur::ModelStatus::Validating);

                // Start validation task using spawn_blocking for CPU-intensive ONNX inference
                let cancel_token = self.cancellation_token.clone();
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            // Create manager and try to load + validate the model
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
                            is_startup: false,
                        },
                        Err(e) => Message::DeblurValidationCompleted {
                            result: Err(e.to_string()),
                            is_startup: false,
                        },
                    },
                )
            }
            Err(e) => {
                // Download failed
                self.settings
                    .set_deblur_model_status(media::deblur::ModelStatus::Error(e.clone()));
                self.notifications.push(
                    notifications::Notification::error("notification-deblur-download-error")
                        .with_arg("error", e),
                );
                Task::none()
            }
        }
    }

    /// Handles the result of deblur model validation.
    ///
    /// When `is_startup` is true, success notifications are suppressed (the user expects
    /// the feature to work from previous sessions). Failure notifications are always shown.
    /// If the app is shutting down, the result is ignored.
    fn handle_deblur_validation_completed(
        &mut self,
        result: Result<(), String>,
        is_startup: bool,
    ) -> Task<Message> {
        // Ignore validation results if the app is shutting down
        if self.shutting_down {
            return Task::none();
        }

        match result {
            Ok(()) => {
                // Validation succeeded - enable deblur and persist state
                self.settings
                    .set_deblur_model_status(media::deblur::ModelStatus::Ready);
                self.settings.set_enable_deblur(true);
                self.persisted.enable_deblur = true;
                if let Some(key) = self.persisted.save() {
                    self.notifications
                        .push(notifications::Notification::warning(&key));
                }
                // Only show success notification for user-initiated activation, not startup
                if !is_startup {
                    self.notifications
                        .push(notifications::Notification::success(
                            "notification-deblur-ready",
                        ));
                }
            }
            Err(e) => {
                // Validation failed - reset enable_deblur, delete the model and show error
                self.settings
                    .set_deblur_model_status(media::deblur::ModelStatus::Error(e.clone()));
                self.settings.set_enable_deblur(false);
                self.persisted.enable_deblur = false;
                if let Some(key) = self.persisted.save() {
                    self.notifications
                        .push(notifications::Notification::warning(&key));
                }
                // Delete the invalid model file
                let _ = std::fs::remove_file(media::deblur::get_model_path());
                self.notifications.push(
                    notifications::Notification::error("notification-deblur-validation-error")
                        .with_arg("error", e),
                );
            }
        }
        Task::none()
    }

    /// Handles the result of upscale model download.
    fn handle_upscale_download_completed(&mut self, result: Result<(), String>) -> Task<Message> {
        // Don't start validation if shutting down
        if self.shutting_down {
            return Task::none();
        }

        match result {
            Ok(()) => {
                // Download succeeded - start validation
                self.settings
                    .set_upscale_model_status(media::upscale::UpscaleModelStatus::Validating);

                // Start validation task using spawn_blocking for CPU-intensive ONNX inference
                let cancel_token = self.cancellation_token.clone();
                Task::perform(
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
                            is_startup: false,
                        },
                        Err(e) => Message::UpscaleValidationCompleted {
                            result: Err(e.to_string()),
                            is_startup: false,
                        },
                    },
                )
            }
            Err(e) => {
                // Download failed
                self.settings
                    .set_upscale_model_status(media::upscale::UpscaleModelStatus::Error(e.clone()));
                self.notifications.push(
                    notifications::Notification::error("notification-upscale-download-error")
                        .with_arg("error", e),
                );
                Task::none()
            }
        }
    }

    /// Handles the result of upscale model validation.
    fn handle_upscale_validation_completed(
        &mut self,
        result: Result<(), String>,
        is_startup: bool,
    ) -> Task<Message> {
        // Ignore validation results if the app is shutting down
        if self.shutting_down {
            return Task::none();
        }

        match result {
            Ok(()) => {
                // Validation succeeded - enable upscale and persist state
                self.settings
                    .set_upscale_model_status(media::upscale::UpscaleModelStatus::Ready);
                self.settings.set_enable_upscale(true);
                self.persisted.enable_upscale = true;
                if let Some(key) = self.persisted.save() {
                    self.notifications
                        .push(notifications::Notification::warning(&key));
                }
                // Only show success notification for user-initiated activation, not startup
                if !is_startup {
                    self.notifications
                        .push(notifications::Notification::success(
                            "notification-upscale-ready",
                        ));
                }
            }
            Err(e) => {
                // Validation failed - reset enable_upscale, delete the model and show error
                self.settings
                    .set_upscale_model_status(media::upscale::UpscaleModelStatus::Error(e.clone()));
                self.settings.set_enable_upscale(false);
                self.persisted.enable_upscale = false;
                if let Some(key) = self.persisted.save() {
                    self.notifications
                        .push(notifications::Notification::warning(&key));
                }
                // Delete the invalid model file
                let _ = std::fs::remove_file(media::upscale::get_model_path());
                self.notifications.push(
                    notifications::Notification::error("notification-upscale-validation-error")
                        .with_arg("error", e),
                );
            }
        }
        Task::none()
    }

    /// Handles async image loading result for the editor.
    // Allow too_many_lines: sequential async result handling with navigation logic.
    // Marginal benefit from extraction (111 lines vs 100 limit).
    #[allow(clippy::too_many_lines)]
    fn handle_image_editor_loaded(
        &mut self,
        result: Result<MediaData, crate::error::Error>,
    ) -> Task<Message> {
        use crate::ui::viewer::{LoadOrigin, NavigationDirection};

        if let Ok(media_data) = result {
            // Editor only supports images - videos are skipped during navigation
            let MediaData::Image(image_data) = media_data else {
                // Should not happen: peek_*_image() only returns images
                return Task::none();
            };

            // Get the tentative path from viewer and confirm navigation
            let Some(path) = self.viewer.current_media_path.clone() else {
                return Task::none();
            };

            // Confirm navigation in MediaNavigator (pessimistic update)
            self.media_navigator.confirm_navigation(&path);

            // Check if we skipped any files during navigation
            let load_origin = std::mem::take(&mut self.viewer.load_origin);
            if let LoadOrigin::Navigation { skipped_files, .. } = load_origin {
                if !skipped_files.is_empty() {
                    let files_text =
                        update::format_skipped_files_message(&self.i18n, &skipped_files);
                    self.notifications.push(
                        notifications::Notification::warning(
                            "notification-skipped-corrupted-files",
                        )
                        .with_arg("files", files_text)
                        .auto_dismiss(std::time::Duration::from_secs(8)),
                    );
                }
            }

            // Create a new ImageEditorState with the loaded image
            match image_editor::State::new(path, &image_data) {
                Ok(new_editor_state) => {
                    self.image_editor = Some(new_editor_state);
                }
                Err(_) => {
                    self.notifications.push(notifications::Notification::error(
                        "notification-editor-create-error",
                    ));
                }
            }
            Task::none()
        } else {
            // Get the failed filename from viewer's tentative path
            let failed_filename = self
                .viewer
                .current_media_path
                .as_ref()
                .and_then(|p| p.file_name())
                .map_or_else(
                    || "unknown".to_string(),
                    |n| n.to_string_lossy().to_string(),
                );

            // Handle based on load origin
            let load_origin = std::mem::take(&mut self.viewer.load_origin);
            match load_origin {
                LoadOrigin::Navigation {
                    direction,
                    skip_attempts,
                    mut skipped_files,
                } => {
                    // Add failed file to the list
                    skipped_files.push(failed_filename);
                    let new_attempts = skip_attempts + 1;
                    let max_attempts = self.viewer.max_skip_attempts;

                    if new_attempts <= max_attempts.value() {
                        // Use peek_nth_*_image with skip_count to find the next file
                        // without modifying navigator state. State is only updated
                        // via confirm_navigation after successful load.
                        let next_path = match direction {
                            NavigationDirection::Next => self
                                .media_navigator
                                .peek_nth_next_image(new_attempts as usize),
                            NavigationDirection::Previous => self
                                .media_navigator
                                .peek_nth_previous_image(new_attempts as usize),
                        };

                        if let Some(path) = next_path {
                            // Set tentative path for next retry
                            self.viewer.current_media_path = Some(path.clone());

                            // Auto-skip: retry navigation in the same direction
                            self.viewer.set_load_origin(LoadOrigin::Navigation {
                                direction,
                                skip_attempts: new_attempts,
                                skipped_files,
                            });

                            Task::perform(
                                async move { media::load_media(&path) },
                                Message::ImageEditorLoaded,
                            )
                        } else {
                            // No more images to navigate to
                            let files_text = update::format_skipped_files_message(
                                &self.i18n,
                                &skipped_files,
                            );
                            self.notifications.push(
                                notifications::Notification::warning(
                                    "notification-skipped-corrupted-files",
                                )
                                .with_arg("files", files_text)
                                .auto_dismiss(std::time::Duration::from_secs(8)),
                            );
                            Task::none()
                        }
                    } else {
                        // Max attempts reached: show grouped notification
                        let files_text =
                            update::format_skipped_files_message(&self.i18n, &skipped_files);
                        self.notifications.push(
                            notifications::Notification::warning(
                                "notification-skipped-corrupted-files",
                            )
                            .with_arg("files", files_text)
                            .auto_dismiss(std::time::Duration::from_secs(8)),
                        );
                        Task::none()
                    }
                }
                LoadOrigin::DirectOpen => {
                    // This case should not happen in the editor since all loads
                    // come from navigation. Kept as defensive fallback.
                    #[cfg(debug_assertions)]
                    eprintln!("[WARN] Unexpected DirectOpen in image editor error handler");
                    self.notifications.push(notifications::Notification::error(
                        "notification-load-error",
                    ));
                    Task::none()
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let count = VIEW_CALL_COUNT.fetch_add(1, Ordering::Relaxed);
        if count < 5 {
            eprintln!("[STARTUP] view() call #{}, screen={:?}", count + 1, self.screen);
        }

        let is_dark_theme = self.theme_mode.is_dark();
        let is_image = matches!(
            self.current_metadata,
            Some(crate::media::metadata::MediaMetadata::Image(_))
        );

        view::view(view::ViewContext {
            i18n: &self.i18n,
            screen: self.screen,
            settings: &self.settings,
            viewer: &self.viewer,
            image_editor: self.image_editor.as_ref(),
            help_state: &self.help_state,
            fullscreen: self.fullscreen,
            menu_open: self.menu_open,
            info_panel_open: self.info_panel_open,
            navigation: self.media_navigator.navigation_info(),
            current_metadata: self.current_metadata.as_ref(),
            metadata_editor_state: self.metadata_editor_state.as_ref(),
            current_media_path: self.media_navigator.current_media_path(),
            is_image,
            notifications: &self.notifications,
            is_dark_theme,
            deblur_model_status: self.settings.deblur_model_status(),
            upscale_model_status: self.settings.upscale_model_status(),
            enable_upscale: self.persisted.enable_upscale,
            filter: self.media_navigator.filter(),
            total_count: self.media_navigator.navigation_info().total_count,
            filtered_count: self.media_navigator.navigation_info().filtered_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DEFAULT_ZOOM_STEP_PERCENT;
    use crate::error::Error;
    use crate::media::ImageData;
    use crate::ui::settings;
    use crate::ui::state::zoom::{
        format_number, DEFAULT_ZOOM_PERCENT, MAX_ZOOM_PERCENT, ZOOM_STEP_INVALID_KEY,
        ZOOM_STEP_RANGE_KEY,
    };
    use crate::ui::viewer::controls;
    use iced::widget::scrollable::AbsoluteOffset;
    use iced::{event, keyboard, mouse, window, Point, Rectangle, Size};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    fn config_env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn with_temp_config_dir<F>(test: F)
    where
        F: FnOnce(&std::path::Path),
    {
        let _guard = config_env_lock().lock().expect("failed to lock mutex");
        let temp_dir = tempdir().expect("failed to create temp dir");
        let previous = std::env::var("XDG_CONFIG_HOME").ok();
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());

        test(temp_dir.path());

        if let Some(value) = previous {
            std::env::set_var("XDG_CONFIG_HOME", value);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
    }

    fn sample_image_data() -> ImageData {
        let pixels = vec![255_u8; 4];
        ImageData::from_rgba(1, 1, pixels)
    }

    fn sample_media_data() -> MediaData {
        MediaData::Image(sample_image_data())
    }

    fn build_image(width: u32, height: u32) -> ImageData {
        let pixel_count = (width * height * 4) as usize;
        let pixels = vec![255; pixel_count];
        ImageData::from_rgba(width, height, pixels)
    }

    fn build_media(width: u32, height: u32) -> MediaData {
        MediaData::Image(build_image(width, height))
    }

    /// Creates a real PNG file for tests that require file I/O (like image editor).
    fn create_test_png(width: u32, height: u32) -> (tempfile::TempDir, PathBuf, ImageData) {
        use image_rs::{Rgba, RgbaImage};

        let temp_dir = tempdir().expect("temp dir");
        let path = temp_dir.path().join("test.png");
        let img = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 255]));
        img.save(&path).expect("write png");
        let pixels = vec![0; (width * height * 4) as usize];
        let image = ImageData::from_rgba(width, height, pixels);
        (temp_dir, path, image)
    }

    #[test]
    fn new_starts_in_viewer_mode_without_image() {
        with_temp_config_dir(|_| {
            let (app, _command) = App::new(Flags::default());
            assert_eq!(app.screen, Screen::Viewer);
            assert!(!app.viewer.has_media());
        });
    }

    #[test]
    fn update_image_loaded_ok_sets_state() {
        let mut app = App::default();
        let data = sample_image_data();

        let _ = app.update(Message::Viewer(component::Message::MediaLoaded(Ok(
            MediaData::Image(data.clone()),
        ))));

        assert!(app.viewer.has_media());
        assert_eq!(app.viewer.media().unwrap().width(), data.width);
    }

    #[test]
    fn default_zoom_state_is_consistent() {
        let app = App::default();

        let zoom = &app.viewer.zoom;
        assert!(zoom.fit_to_window);
        assert_eq!(zoom.zoom_percent, DEFAULT_ZOOM_PERCENT);
        assert_eq!(zoom.zoom_input, format_number(DEFAULT_ZOOM_PERCENT));
        assert_eq!(zoom.zoom_step.value(), DEFAULT_ZOOM_STEP_PERCENT);
        assert_eq!(
            app.settings.background_theme(),
            config::BackgroundTheme::default()
        );
    }

    #[test]
    fn zoom_step_changes_commit_when_leaving_settings() {
        with_temp_config_dir(|_| {
            let mut app = App {
                screen: Screen::Settings,
                ..App::default()
            };
            let _ = app.update(Message::Settings(settings::Message::ZoomStepInputChanged(
                "25".into(),
            )));

            let _ = app.update(Message::SwitchScreen(Screen::Viewer));

            assert_eq!(app.screen, Screen::Viewer);
            assert_eq!(app.viewer.zoom_step_percent(), 25.0);
            assert_eq!(app.settings.zoom_step_input_value(), "25");
            assert!(!app.settings.zoom_step_input_dirty());
            assert!(app.settings.zoom_step_error_key().is_none());
        });
    }

    #[test]
    fn invalid_zoom_step_prevents_leaving_settings() {
        with_temp_config_dir(|_| {
            let mut app = App {
                screen: Screen::Settings,
                ..App::default()
            };
            let _ = app.update(Message::Settings(settings::Message::ZoomStepInputChanged(
                "not-a-number".into(),
            )));

            let _ = app.update(Message::SwitchScreen(Screen::Viewer));

            assert_eq!(app.screen, Screen::Settings);
            assert_eq!(
                app.settings.zoom_step_error_key(),
                Some(ZOOM_STEP_INVALID_KEY)
            );
            assert!(app.settings.zoom_step_input_dirty());
            assert_eq!(app.viewer.zoom_step_percent(), DEFAULT_ZOOM_STEP_PERCENT);
        });
    }

    #[test]
    fn out_of_range_zoom_step_shows_error_and_stays_in_settings() {
        with_temp_config_dir(|_| {
            let mut app = App {
                screen: Screen::Settings,
                ..App::default()
            };
            let _ = app.update(Message::Settings(settings::Message::ZoomStepInputChanged(
                "500".into(),
            )));

            let _ = app.update(Message::SwitchScreen(Screen::Viewer));

            assert_eq!(app.screen, Screen::Settings);
            assert_eq!(
                app.settings.zoom_step_error_key(),
                Some(ZOOM_STEP_RANGE_KEY)
            );
            assert!(app.settings.zoom_step_input_dirty());
            assert_eq!(app.viewer.zoom_step_percent(), DEFAULT_ZOOM_STEP_PERCENT);
        });
    }

    #[test]
    fn update_image_loaded_err_shows_notification_and_preserves_media() {
        let mut app = App::default();
        let _ = app.update(Message::Viewer(component::Message::MediaLoaded(Ok(
            sample_media_data(),
        ))));

        // Verify media is loaded
        assert!(app.viewer.has_media());

        let _ = app.update(Message::Viewer(component::Message::MediaLoaded(Err(
            Error::Io("boom".into()),
        ))));

        // New behavior: media is preserved, error panel is NOT set
        // A notification is shown instead (non-blocking UX)
        assert!(
            app.viewer.has_media(),
            "media should be preserved on load error"
        );
        assert!(
            app.viewer.error().is_none(),
            "error panel should not be set - notifications are used instead"
        );
        // Notification was pushed (we can verify via notifications manager)
        assert!(
            app.notifications.has_notifications(),
            "a notification should be shown for the error"
        );
    }

    #[test]
    fn submitting_valid_zoom_input_updates_zoom() {
        let mut app = App::default();
        let zoom = app.viewer.zoom_state_mut();
        zoom.zoom_input = "150".into();
        zoom.fit_to_window = true;

        let _ = app.update(Message::Viewer(component::Message::Controls(
            controls::Message::ZoomInputSubmitted,
        )));

        let zoom = app.viewer.zoom_state();
        assert_eq!(zoom.zoom_percent, 150.0);
        assert_eq!(zoom.manual_zoom_percent, 150.0);
        assert_eq!(zoom.zoom_input, format_number(150.0));
        assert!(!zoom.fit_to_window);
        assert!(zoom.zoom_input_error_key.is_none());
    }

    #[test]
    fn submitting_out_of_range_zoom_clamps_value() {
        let mut app = App::default();
        let zoom = app.viewer.zoom_state_mut();
        zoom.zoom_input = "9999".into();

        let _ = app.update(Message::Viewer(component::Message::Controls(
            controls::Message::ZoomInputSubmitted,
        )));

        let zoom = app.viewer.zoom_state();
        assert_eq!(zoom.zoom_percent, MAX_ZOOM_PERCENT);
        assert_eq!(zoom.zoom_input, format_number(MAX_ZOOM_PERCENT));
        assert_eq!(zoom.manual_zoom_percent, MAX_ZOOM_PERCENT);
        assert!(!zoom.fit_to_window);
    }

    #[test]
    fn submitting_invalid_zoom_sets_error() {
        let mut app = App::default();
        let zoom = app.viewer.zoom_state_mut();
        zoom.fit_to_window = true;
        zoom.zoom_input = "oops".into();

        let _ = app.update(Message::Viewer(component::Message::Controls(
            controls::Message::ZoomInputSubmitted,
        )));

        let zoom = app.viewer.zoom_state();
        assert_eq!(zoom.zoom_percent, DEFAULT_ZOOM_PERCENT);
        assert!(zoom.fit_to_window);
        assert_eq!(
            zoom.zoom_input_error_key,
            Some(crate::ui::state::zoom::ZOOM_INPUT_INVALID_KEY)
        );
    }

    #[test]
    fn reset_zoom_restores_defaults() {
        let mut app = App::default();
        let zoom = app.viewer.zoom_state_mut();
        zoom.zoom_percent = 250.0;
        zoom.manual_zoom_percent = 250.0;
        zoom.fit_to_window = false;
        zoom.zoom_input = "250".into();

        let _ = app.update(Message::Viewer(component::Message::Controls(
            controls::Message::ResetZoom,
        )));

        let zoom = app.viewer.zoom_state();
        assert_eq!(zoom.zoom_percent, DEFAULT_ZOOM_PERCENT);
        assert_eq!(zoom.manual_zoom_percent, DEFAULT_ZOOM_PERCENT);
        assert_eq!(zoom.zoom_input, format_number(DEFAULT_ZOOM_PERCENT));
        assert!(!zoom.fit_to_window);
    }

    #[test]
    fn toggling_fit_to_window_updates_zoom() {
        let mut app = App::default();
        let _ = app.viewer.handle_message(
            component::Message::MediaLoaded(Ok(build_media(2000, 1000))),
            &app.i18n,
        );
        app.viewer.viewport_state_mut().bounds = Some(Rectangle::new(
            Point::new(0.0, 0.0),
            Size::new(1000.0, 500.0),
        ));

        let zoom = app.viewer.zoom_state_mut();
        zoom.fit_to_window = false;
        zoom.manual_zoom_percent = 160.0;

        let _ = app.update(Message::Viewer(component::Message::Controls(
            controls::Message::SetFitToWindow(true),
        )));

        let zoom = app.viewer.zoom_state();
        assert!(zoom.fit_to_window);
        let fit_zoom = app.viewer.compute_fit_zoom_percent().unwrap();
        assert_eq!(zoom.zoom_percent, fit_zoom);
        assert_eq!(zoom.zoom_input, format_number(fit_zoom));
    }

    #[test]
    fn viewport_change_updates_offset_tracking() {
        let mut app = App::default();
        let first = AbsoluteOffset { x: 10.0, y: 5.0 };
        let second = AbsoluteOffset { x: 4.0, y: 2.0 };
        let bounds = Rectangle::new(Point::new(32.0, 48.0), Size::new(800.0, 600.0));

        let _ = app.update(Message::Viewer(component::Message::ViewportChanged {
            bounds,
            offset: first,
        }));
        let _ = app.update(Message::Viewer(component::Message::ViewportChanged {
            bounds,
            offset: second,
        }));

        let viewport = app.viewer.viewport_state();
        assert_eq!(viewport.previous_offset, first);
        assert_eq!(viewport.offset, second);
        assert_eq!(viewport.bounds, Some(bounds));
    }

    #[test]
    fn wheel_scroll_applies_zoom_step_when_over_image() {
        let mut app = App::default();
        app.viewer.set_zoom_step_percent(15.0);
        let zoom = app.viewer.zoom_state_mut();
        zoom.zoom_percent = 100.0;
        let _ = app.viewer.handle_message(
            component::Message::MediaLoaded(Ok(build_media(800, 600))),
            &app.i18n,
        );
        app.viewer.viewport_state_mut().bounds = Some(Rectangle::new(
            Point::new(10.0, 10.0),
            Size::new(400.0, 300.0),
        ));
        app.viewer
            .set_cursor_position(Some(Point::new(210.0, 160.0)));

        let _ = app.update(Message::Viewer(component::Message::RawEvent {
            window: window::Id::unique(),
            event: event::Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
            }),
        }));

        let zoom = app.viewer.zoom_state();
        assert_eq!(zoom.zoom_percent, 115.0);
        assert_eq!(zoom.manual_zoom_percent, 115.0);
        assert!(!zoom.fit_to_window);
    }

    #[test]
    fn wheel_scroll_ignored_when_cursor_not_over_image() {
        let mut app = App::default();
        app.viewer.set_zoom_step_percent(20.0);

        // Load image first (this will reset zoom to 100% if fit_to_window is false)
        let _ = app.viewer.handle_message(
            component::Message::MediaLoaded(Ok(build_media(800, 600))),
            &app.i18n,
        );

        // Configure zoom after loading to set up test state
        let zoom = app.viewer.zoom_state_mut();
        zoom.zoom_percent = 150.0;
        zoom.manual_zoom_percent = 150.0;
        zoom.fit_to_window = false;

        app.viewer.viewport_state_mut().bounds = Some(Rectangle::new(
            Point::new(0.0, 0.0),
            Size::new(400.0, 300.0),
        ));
        app.viewer
            .set_cursor_position(Some(Point::new(1000.0, 1000.0)));

        let _ = app.update(Message::Viewer(component::Message::RawEvent {
            window: window::Id::unique(),
            event: event::Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
            }),
        }));

        let zoom = app.viewer.zoom_state();
        assert_eq!(zoom.zoom_percent, 150.0);
        assert_eq!(zoom.manual_zoom_percent, 150.0);
        assert!(!zoom.fit_to_window);
    }

    #[test]
    fn language_selected_updates_config_file() {
        with_temp_config_dir(|config_root| {
            let mut app = App::default();
            let target_locale: unic_langid::LanguageIdentifier = app
                .i18n
                .available_locales
                .iter()
                .find(|locale| locale.to_string() == "fr")
                .cloned()
                .unwrap_or_else(|| app.i18n.current_locale().clone());

            let _ = app.update(Message::Settings(settings::Message::LanguageSelected(
                target_locale.clone(),
            )));

            let config_path = config_root.join("IcedLens").join("settings.toml");
            assert!(config_path.exists());
            let contents = fs::read_to_string(config_path).expect("config should be readable");
            assert!(contents.contains(&target_locale.to_string()));
        });
    }

    #[test]
    fn persist_preferences_handles_save_errors() {
        with_temp_config_dir(|config_root| {
            // Create a directory where the config file should be, causing write to fail
            let settings_dir = config_root.join("IcedLens");
            fs::create_dir_all(&settings_dir).expect("dir");
            fs::create_dir_all(settings_dir.join("settings.toml"))
                .expect("create conflicting directory");

            // Call persist_preferences directly with default values
            // This should not panic even though the save will fail
            let viewer = component::State::new();
            let settings_state = SettingsState::default();
            let mut notifs = notifications::Manager::new();
            let nav = MediaNavigator::default();
            let mut ctx = persistence::PreferencesContext {
                viewer: &viewer,
                settings: &settings_state,
                theme_mode: crate::ui::theming::ThemeMode::System,
                video_autoplay: false,
                audio_normalization: true,
                frame_cache_mb: crate::video_player::FrameCacheMb::default().value(),
                frame_history_mb: crate::video_player::FrameHistoryMb::default().value(),
                keyboard_seek_step_secs: config::DEFAULT_KEYBOARD_SEEK_STEP_SECS,
                notifications: &mut notifs,
                media_navigator: &nav,
            };
            let _ = persistence::persist_preferences(&mut ctx);
            // Test passes if we reach here without panicking
        });
    }

    #[test]
    fn navigate_next_loads_next_image() {
        use std::io::Write;
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1_path = temp_dir.path().join("a.jpg");
        let img2_path = temp_dir.path().join("b.jpg");

        fs::File::create(&img1_path)
            .expect("failed to create img1")
            .write_all(b"fake")
            .expect("failed to write img1");
        fs::File::create(&img2_path)
            .expect("failed to create img2")
            .write_all(b"fake")
            .expect("failed to write img2");

        let mut app = App::default();
        let _ = app.update(Message::Viewer(component::Message::MediaLoaded(Ok(
            sample_media_data(),
        ))));
        app.viewer.current_media_path = Some(img1_path.clone());

        // Initialize media_navigator (single source of truth)
        let _ = app
            .media_navigator
            .scan_directory(&img1_path, crate::config::SortOrder::Alphabetical);

        let _ = app.update(Message::Viewer(component::Message::NavigateNext));

        assert!(app
            .viewer
            .current_media_path
            .as_ref()
            .map(|p| p.ends_with("b.jpg"))
            .unwrap_or(false));
    }

    #[test]
    fn navigate_previous_loads_previous_image() {
        use std::io::Write;
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1_path = temp_dir.path().join("a.jpg");
        let img2_path = temp_dir.path().join("b.jpg");

        fs::File::create(&img1_path)
            .expect("failed to create img1")
            .write_all(b"fake")
            .expect("failed to write img1");
        fs::File::create(&img2_path)
            .expect("failed to create img2")
            .write_all(b"fake")
            .expect("failed to write img2");

        let mut app = App::default();
        let _ = app.update(Message::Viewer(component::Message::MediaLoaded(Ok(
            sample_media_data(),
        ))));
        app.viewer.current_media_path = Some(img2_path.clone());

        // Initialize media_navigator (single source of truth)
        let _ = app
            .media_navigator
            .scan_directory(&img2_path, crate::config::SortOrder::Alphabetical);

        let _ = app.update(Message::Viewer(component::Message::NavigatePrevious));

        assert!(app
            .viewer
            .current_media_path
            .as_ref()
            .map(|p| p.ends_with("a.jpg"))
            .unwrap_or(false));
    }

    #[test]
    fn navigate_next_wraps_to_first() {
        use std::io::Write;
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1_path = temp_dir.path().join("a.jpg");
        let img2_path = temp_dir.path().join("b.jpg");

        fs::File::create(&img1_path)
            .expect("failed to create img1")
            .write_all(b"fake")
            .expect("failed to write img1");
        fs::File::create(&img2_path)
            .expect("failed to create img2")
            .write_all(b"fake")
            .expect("failed to write img2");

        let mut app = App::default();
        let _ = app.update(Message::Viewer(component::Message::MediaLoaded(Ok(
            sample_media_data(),
        ))));
        app.viewer.current_media_path = Some(img2_path.clone());

        // Initialize media_navigator (single source of truth)
        let _ = app
            .media_navigator
            .scan_directory(&img2_path, crate::config::SortOrder::Alphabetical);

        let _ = app.update(Message::Viewer(component::Message::NavigateNext));

        assert!(app
            .viewer
            .current_media_path
            .as_ref()
            .map(|p| p.ends_with("a.jpg"))
            .unwrap_or(false));
    }

    #[test]
    fn keyboard_right_arrow_navigates_next() {
        with_temp_config_dir(|_| {
            use std::io::Write;
            use tempfile::tempdir;

            let temp_dir = tempdir().expect("failed to create temp dir");
            let img1_path = temp_dir.path().join("a.jpg");
            let img2_path = temp_dir.path().join("b.jpg");

            fs::File::create(&img1_path)
                .expect("failed to create img1")
                .write_all(b"fake")
                .expect("failed to write img1");
            fs::File::create(&img2_path)
                .expect("failed to create img2")
                .write_all(b"fake")
                .expect("failed to write img2");

            let mut app = App::default();
            let _ = app.update(Message::Viewer(component::Message::MediaLoaded(Ok(
                sample_media_data(),
            ))));
            app.viewer.current_media_path = Some(img1_path.clone());

            // Initialize media_navigator (single source of truth)
            let _ = app
                .media_navigator
                .scan_directory(&img1_path, crate::config::SortOrder::Alphabetical);

            let _ = app.update(Message::Viewer(component::Message::RawEvent {
                window: window::Id::unique(),
                event: event::Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::ArrowRight),
                    modified_key: keyboard::Key::Named(keyboard::key::Named::ArrowRight),
                    physical_key: keyboard::key::Physical::Code(keyboard::key::Code::ArrowRight),
                    location: keyboard::Location::Standard,
                    modifiers: keyboard::Modifiers::default(),
                    text: None,
                    repeat: false,
                }),
            }));

            assert!(app
                .viewer
                .current_media_path
                .as_ref()
                .map(|p| p.ends_with("b.jpg"))
                .unwrap_or(false));
        });
    }

    /// Helper function to create a real PNG image for editor tests
    fn create_real_png_image(path: &Path, width: u32, height: u32) -> std::io::Result<()> {
        use image_rs::{DynamicImage, ImageBuffer};
        let buffer = ImageBuffer::from_pixel(width, height, image_rs::Rgba([255, 0, 0, 255]));
        let img = DynamicImage::ImageRgba8(buffer);
        img.save(path).map_err(std::io::Error::other)
    }

    #[test]
    fn editor_navigate_next_loads_next_image() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1_path = temp_dir.path().join("a.png");
        let img2_path = temp_dir.path().join("b.png");

        // Create real PNG images
        create_real_png_image(&img1_path, 10, 10).expect("failed to create img1");
        create_real_png_image(&img2_path, 10, 10).expect("failed to create img2");

        let mut app = App::default();

        // Load first image in viewer
        let img1_data = media::load_media(&img1_path).expect("failed to load img1");
        let _ = app.update(Message::Viewer(component::Message::MediaLoaded(Ok(
            img1_data.clone(),
        ))));
        app.viewer.current_media_path = Some(img1_path.clone());

        // Initialize media_navigator (single source of truth)
        let _ = app
            .media_navigator
            .scan_directory(&img1_path, crate::config::SortOrder::Alphabetical);

        // Switch to editor screen
        let _ = app.update(Message::SwitchScreen(Screen::ImageEditor));

        // Navigate to next image
        let _ = app.update(Message::ImageEditor(image_editor::Message::Sidebar(
            crate::ui::image_editor::SidebarMessage::NavigateNext,
        )));

        // Verify the viewer's current image path has changed to the next image
        assert!(app
            .viewer
            .current_media_path
            .as_ref()
            .map(|p| p.ends_with("b.png"))
            .unwrap_or(false));

        // Simulate the async image loading completing
        let img2_data = media::load_media(&img2_path).expect("failed to load img2");
        let _ = app.update(Message::ImageEditorLoaded(Ok(img2_data)));

        // Verify editor has loaded the second image
        assert!(app.image_editor.is_some(), "Editor should still be active");
        if let Some(editor) = &app.image_editor {
            assert_eq!(
                editor.image_path(),
                Some(img2_path.as_path()),
                "Editor should have loaded b.png"
            );
        }
    }

    #[test]
    fn editor_navigate_previous_loads_previous_image() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1_path = temp_dir.path().join("a.png");
        let img2_path = temp_dir.path().join("b.png");

        // Create real PNG images
        create_real_png_image(&img1_path, 10, 10).expect("failed to create img1");
        create_real_png_image(&img2_path, 10, 10).expect("failed to create img2");

        let mut app = App::default();

        // Load second image in viewer
        let img2_data = media::load_media(&img2_path).expect("failed to load img2");
        let _ = app.update(Message::Viewer(component::Message::MediaLoaded(Ok(
            img2_data.clone(),
        ))));
        app.viewer.current_media_path = Some(img2_path.clone());

        // Initialize media_navigator (single source of truth)
        let _ = app
            .media_navigator
            .scan_directory(&img2_path, crate::config::SortOrder::Alphabetical);

        // Switch to editor screen
        let _ = app.update(Message::SwitchScreen(Screen::ImageEditor));

        // Navigate to previous image
        let _ = app.update(Message::ImageEditor(image_editor::Message::Sidebar(
            crate::ui::image_editor::SidebarMessage::NavigatePrevious,
        )));

        // Verify the viewer's current image path has changed to the previous image
        assert!(app
            .viewer
            .current_media_path
            .as_ref()
            .map(|p| p.ends_with("a.png"))
            .unwrap_or(false));

        // Simulate the async image loading completing
        let img1_data = media::load_media(&img1_path).expect("failed to load img1");
        let _ = app.update(Message::ImageEditorLoaded(Ok(img1_data)));

        // Verify editor has loaded the first image
        assert!(app.image_editor.is_some(), "Editor should still be active");
        if let Some(editor) = &app.image_editor {
            assert_eq!(
                editor.image_path(),
                Some(img1_path.as_path()),
                "Editor should have loaded a.png"
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Dynamic window title tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn title_shows_app_name_when_no_media_loaded() {
        let app = App::default();
        let title = app.title();

        // Should just be the app name (from window-title translation)
        assert_eq!(title, "IcedLens");
    }

    #[test]
    fn title_shows_filename_when_media_loaded() {
        let mut app = App::default();

        // Load a media file
        let _ = app.update(Message::Viewer(component::Message::MediaLoaded(Ok(
            sample_media_data(),
        ))));
        // Set path in media_navigator (single source of truth) and viewer
        let path = PathBuf::from("/path/to/image.jpg");
        app.media_navigator.set_current_media_path(path.clone());
        app.viewer.current_media_path = Some(path);

        let title = app.title();

        // Should show "filename - AppName"
        assert_eq!(title, "image.jpg - IcedLens");
    }

    #[test]
    fn title_shows_asterisk_when_editor_has_unsaved_changes() {
        // Create a real PNG file for the image editor
        let (_temp_dir, img_path, img_data) = create_test_png(4, 3);

        let mut app = App::default();

        // Load image and set the path in media_navigator (single source of truth) and viewer
        let _ = app.update(Message::Viewer(component::Message::MediaLoaded(Ok(
            MediaData::Image(img_data.clone()),
        ))));
        app.media_navigator.set_current_media_path(img_path.clone());
        app.viewer.current_media_path = Some(img_path.clone());

        // Create editor state with actual PNG file
        let editor_state =
            image_editor::State::new(img_path, &img_data).expect("create editor state");
        app.image_editor = Some(editor_state);
        app.screen = Screen::ImageEditor;

        // Title without changes - file name is "test.png" from helper
        let title_clean = app.title();
        assert_eq!(
            title_clean, "test.png - IcedLens",
            "Should not have asterisk without changes"
        );

        // Apply a transformation to create unsaved changes
        let _ = app.update(Message::ImageEditor(image_editor::Message::Sidebar(
            crate::ui::image_editor::SidebarMessage::RotateRight,
        )));

        // Title with unsaved changes
        let title_dirty = app.title();
        assert_eq!(
            title_dirty, "*test.png - IcedLens",
            "Should have asterisk with unsaved changes"
        );
    }

    #[test]
    fn title_shows_new_image_for_captured_frame() {
        use crate::media::frame_export::ExportableFrame;
        use std::sync::Arc;

        // Create a captured frame (4x3 black pixels)
        let rgba_data = Arc::new(vec![0u8; 4 * 3 * 4]); // width * height * 4 channels
        let frame = ExportableFrame::new(rgba_data, 4, 3);
        let video_path = PathBuf::from("/path/to/video.mp4");

        let mut app = App::default();

        // Create editor state from captured frame
        let editor_state = image_editor::State::from_captured_frame(&frame, video_path, 5.0)
            .expect("create editor state from captured frame");
        app.image_editor = Some(editor_state);
        app.screen = Screen::ImageEditor;

        // Title should show "New Image" without asterisk
        let title = app.title();
        assert_eq!(
            title, "New Image - IcedLens",
            "Captured frame should show 'New Image' without asterisk"
        );
    }

    #[test]
    fn title_shows_new_image_for_captured_frame_even_with_changes() {
        use crate::media::frame_export::ExportableFrame;
        use std::sync::Arc;

        // Create a captured frame
        let rgba_data = Arc::new(vec![0u8; 4 * 3 * 4]);
        let frame = ExportableFrame::new(rgba_data, 4, 3);
        let video_path = PathBuf::from("/path/to/video.mp4");

        let mut app = App::default();

        // Create editor state from captured frame
        let editor_state = image_editor::State::from_captured_frame(&frame, video_path, 5.0)
            .expect("create editor state from captured frame");
        app.image_editor = Some(editor_state);
        app.screen = Screen::ImageEditor;

        // Apply a transformation
        let _ = app.update(Message::ImageEditor(image_editor::Message::Sidebar(
            crate::ui::image_editor::SidebarMessage::FlipHorizontal,
        )));

        // Title should still show "New Image" without asterisk
        // (captured frames are new documents, not modified existing files)
        let title = app.title();
        assert_eq!(
            title, "New Image - IcedLens",
            "Captured frame should show 'New Image' even with changes (no asterisk)"
        );
    }
}
