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
use crate::media::{self, MediaData, MediaNavigator};
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

/// Root Iced application state that bridges UI components, localization, and
/// persisted preferences.
pub struct App {
    pub i18n: I18n,
    screen: Screen,
    settings: SettingsState,
    viewer: component::State,
    image_editor: Option<ImageEditorState>,
    media_navigator: MediaNavigator,
    fullscreen: bool,
    window_id: Option<window::Id>,
    theme_mode: ThemeMode,
    /// Whether videos should auto-play when loaded.
    video_autoplay: bool,
    /// Whether audio normalization is enabled for consistent volume levels.
    audio_normalization: bool,
    /// Shared cache for LUFS measurements to avoid re-analyzing files.
    lufs_cache: SharedLufsCache,
    /// Frame cache size in MB for video seek optimization.
    frame_cache_mb: u32,
    /// Frame history size in MB for backward frame stepping.
    frame_history_mb: u32,
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
    app_state: persisted_state::AppState,
    /// Toast notification manager for user feedback.
    notifications: notifications::Manager,
}

impl fmt::Debug for App {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("App")
            .field("screen", &self.screen)
            .field("viewer_has_image", &self.viewer.has_media())
            .finish()
    }
}

pub const WINDOW_DEFAULT_HEIGHT: u32 = 650;
pub const WINDOW_DEFAULT_WIDTH: u32 = 800;
pub const MIN_WINDOW_HEIGHT: u32 = 650;
pub const MIN_WINDOW_WIDTH: u32 = 650;

/// Ensures zoom step values stay inside the supported range so persisted
/// configs cannot request nonsensical increments.
fn clamp_zoom_step(value: f32) -> f32 {
    value.clamp(MIN_ZOOM_STEP_PERCENT, MAX_ZOOM_STEP_PERCENT)
}

/// Builds the window settings
pub fn window_settings_with_locale() -> window::Settings {
    let icon = crate::icon::load_window_icon();

    window::Settings {
        size: iced::Size::new(WINDOW_DEFAULT_WIDTH as f32, WINDOW_DEFAULT_HEIGHT as f32),
        min_size: Some(iced::Size::new(
            MIN_WINDOW_WIDTH as f32,
            MIN_WINDOW_HEIGHT as f32,
        )),
        icon,
        ..window::Settings::default()
    }
}

/// Entry point used by `main.rs` to launch the Iced application loop.
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
            theme_mode: ThemeMode::System,
            video_autoplay: false,
            audio_normalization: true, // Enabled by default - normalizes audio volume between media files
            lufs_cache: create_lufs_cache(),
            frame_cache_mb: config::DEFAULT_FRAME_CACHE_MB,
            frame_history_mb: config::DEFAULT_FRAME_HISTORY_MB,
            menu_open: false,
            info_panel_open: false,
            current_metadata: None,
            metadata_editor_state: None,
            help_state: help::State::new(),
            app_state: persisted_state::AppState::default(),
            notifications: notifications::Manager::new(),
        }
    }
}

impl App {
    /// Initializes application state and optionally kicks off asynchronous image
    /// loading based on `Flags` received from the launcher.
    fn new(flags: Flags) -> (Self, Task<Message>) {
        let (config, config_warning) = config::load();
        let i18n = I18n::new(flags.lang.clone(), flags.i18n_dir.clone(), &config);

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
        let frame_cache_mb = config
            .video
            .frame_cache_mb
            .unwrap_or(config::DEFAULT_FRAME_CACHE_MB);
        let frame_history_mb = config
            .video
            .frame_history_mb
            .unwrap_or(config::DEFAULT_FRAME_HISTORY_MB);
        app.frame_cache_mb = frame_cache_mb;
        app.frame_history_mb = frame_history_mb;
        app.settings = SettingsState::new(SettingsConfig {
            zoom_step_percent: app.viewer.zoom_step_percent(),
            background_theme: theme,
            sort_order,
            overlay_timeout_secs,
            theme_mode: config.general.theme_mode,
            video_autoplay,
            audio_normalization,
            frame_cache_mb,
            frame_history_mb,
            keyboard_seek_step_secs,
        });
        app.video_autoplay = video_autoplay;
        app.audio_normalization = audio_normalization;
        app.viewer.set_video_autoplay(video_autoplay);
        app.viewer
            .set_keyboard_seek_step_secs(keyboard_seek_step_secs);

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

        // Load application state (last save directory, etc.)
        let (app_state, state_warning) = persisted_state::AppState::load();
        app.app_state = app_state;

        // Show warnings for config/state loading issues
        if let Some(key) = config_warning {
            app.notifications
                .push(notifications::Notification::warning(&key));
        }
        if let Some(key) = state_warning {
            app.notifications
                .push(notifications::Notification::warning(&key));
        }

        let task = if let Some(path_str) = flags.file_path {
            let path = std::path::PathBuf::from(&path_str);

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

            if let Some(media_path) = resolved_path {
                // Synchronize viewer state
                app.viewer.current_media_path = Some(media_path.clone());

                // Set loading state directly (before first render)
                app.viewer.is_loading_media = true;
                app.viewer.loading_started_at = Some(std::time::Instant::now());

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

        (app, task)
    }

    fn title(&self) -> String {
        let app_name = self.i18n.tr("window-title");

        // Special handling for image editor screen
        if self.screen == Screen::ImageEditor {
            if let Some(editor) = &self.image_editor {
                // Captured frame: show "New Image" without asterisk
                // (it's a new document, not a modified existing file)
                if editor.is_captured_frame() {
                    let new_image = self.i18n.tr("new-image-title");
                    return format!("{new_image} - {app_name}");
                }

                // Existing file: show filename with asterisk if unsaved changes
                if let Some(path) = editor.image_path() {
                    let file_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown");

                    return if editor.has_unsaved_changes() {
                        format!("*{file_name} - {app_name}")
                    } else {
                        format!("{file_name} - {app_name}")
                    };
                }
            }
        }

        // All other screens: use viewer's current media path
        let file_name = self.viewer.current_media_path.as_ref().and_then(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(String::from)
        });

        // Check for metadata editor unsaved changes
        let metadata_has_changes = self
            .metadata_editor_state
            .as_ref()
            .map(|editor| editor.has_changes())
            .unwrap_or(false);

        match file_name {
            Some(name) => {
                if metadata_has_changes {
                    format!("*{name} - {app_name}")
                } else {
                    format!("{name} - {app_name}")
                }
            }
            None => app_name,
        }
    }

    fn theme(&self) -> Theme {
        match self.theme_mode {
            ThemeMode::Light => Theme::Light,
            ThemeMode::Dark => Theme::Dark,
            ThemeMode::System => Theme::Dark,
        }
    }

    fn subscription(&self) -> Subscription<Message> {
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
            self.frame_cache_mb,
            self.settings.frame_history_mb(),
        );

        Subscription::batch([event_sub, tick_sub, video_sub])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        let mut ctx = update::UpdateContext {
            i18n: &mut self.i18n,
            screen: &mut self.screen,
            settings: &mut self.settings,
            viewer: &mut self.viewer,
            image_editor: &mut self.image_editor,
            media_navigator: &mut self.media_navigator,
            fullscreen: &mut self.fullscreen,
            window_id: &mut self.window_id,
            theme_mode: &mut self.theme_mode,
            video_autoplay: &mut self.video_autoplay,
            audio_normalization: &mut self.audio_normalization,
            menu_open: &mut self.menu_open,
            info_panel_open: &mut self.info_panel_open,
            current_metadata: &mut self.current_metadata,
            metadata_editor_state: &mut self.metadata_editor_state,
            help_state: &mut self.help_state,
            app_state: &mut self.app_state,
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
            Message::About(about_message) => update::handle_about_message(&mut ctx, about_message),
            Message::MetadataPanel(panel_message) => {
                update::handle_metadata_panel_message(&mut ctx, panel_message)
            }
            Message::Notification(notification_message) => {
                self.notifications.handle_message(notification_message);
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
                                self.app_state.set_last_save_directory_from_file(&path);
                                if let Some(key) = self.app_state.save() {
                                    self.notifications
                                        .push(notifications::Notification::warning(&key));
                                }

                                // Rescan directory if saved in the same folder as viewer
                                persistence::rescan_directory_if_same(
                                    &mut self.viewer,
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
                            self.app_state.set_last_save_directory_from_file(&path);
                            if let Some(key) = self.app_state.save() {
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
                match ImageEditorState::from_captured_frame(frame, video_path, position_secs) {
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
                update::handle_open_file_dialog(self.app_state.last_open_directory.clone())
            }
            Message::OpenFileDialogResult(path) => {
                update::handle_open_file_dialog_result(&mut ctx, path)
            }
            Message::FileDropped(path) => update::handle_file_dropped(&mut ctx, path),
            Message::MetadataSaveAsDialogResult(path_opt) => {
                if let Some(path) = path_opt {
                    self.handle_metadata_save_as(path)
                } else {
                    Task::none()
                }
            }
        }
    }

    /// Handles the metadata Save As dialog result.
    fn handle_metadata_save_as(&mut self, path: std::path::PathBuf) -> Task<Message> {
        use crate::media::metadata_writer;

        // First, copy the original file to the new location
        if let Some(source_path) = self.viewer.current_media_path.as_ref() {
            if let Err(_e) = std::fs::copy(source_path, &path) {
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
            match metadata_writer::write_exif(&path, editor_state.editable_metadata()) {
                Ok(()) => {
                    // Remember the save directory
                    self.app_state.set_last_save_directory_from_file(&path);
                    if let Some(key) = self.app_state.save() {
                        self.notifications
                            .push(notifications::Notification::warning(&key));
                    }

                    // Refresh metadata display
                    self.current_metadata = media::metadata::extract_metadata(&path);

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
                    let _ = std::fs::remove_file(&path);
                    self.notifications.push(notifications::Notification::error(
                        "notification-metadata-save-error",
                    ));
                }
            }
        }
        Task::none()
    }

    /// Handles async image loading result for the editor.
    fn handle_image_editor_loaded(
        &mut self,
        result: Result<MediaData, crate::error::Error>,
    ) -> Task<Message> {
        match result {
            Ok(media_data) => {
                // Editor only supports images - videos are skipped during navigation
                let MediaData::Image(image_data) = media_data else {
                    // Should not happen: navigate_*_image() only returns images
                    return Task::none();
                };

                // Create a new ImageEditorState with the loaded image
                if let Some(current_media_path) = self.media_navigator.current_media_path() {
                    let path = current_media_path.to_path_buf();
                    match image_editor::State::new(path, image_data) {
                        Ok(new_editor_state) => {
                            self.image_editor = Some(new_editor_state);
                        }
                        Err(_) => {
                            self.notifications.push(notifications::Notification::error(
                                "notification-editor-create-error",
                            ));
                        }
                    }
                }
            }
            Err(_) => {
                self.notifications.push(notifications::Notification::error(
                    "notification-editor-load-error",
                ));
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
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
            current_media_path: self.viewer.current_media_path.as_ref(),
            is_image,
            notifications: &self.notifications,
            is_dark_theme,
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
    use iced::widget::image::Handle;
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
        ImageData {
            handle: Handle::from_rgba(1, 1, pixels),
            width: 1,
            height: 1,
        }
    }

    fn sample_media_data() -> MediaData {
        MediaData::Image(sample_image_data())
    }

    fn build_image(width: u32, height: u32) -> ImageData {
        let pixel_count = (width * height * 4) as usize;
        let pixels = vec![255; pixel_count];
        ImageData {
            handle: Handle::from_rgba(width, height, pixels),
            width,
            height,
        }
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
        let image = ImageData {
            handle: Handle::from_rgba(width, height, pixels),
            width,
            height,
        };
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
        assert_eq!(zoom.zoom_step_percent, DEFAULT_ZOOM_STEP_PERCENT);
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
            let _ = persistence::persist_preferences(persistence::PreferencesContext {
                viewer: &viewer,
                settings: &settings_state,
                theme_mode: crate::ui::theming::ThemeMode::System,
                video_autoplay: false,
                audio_normalization: true,
                frame_cache_mb: config::DEFAULT_FRAME_CACHE_MB,
                frame_history_mb: config::DEFAULT_FRAME_HISTORY_MB,
                keyboard_seek_step_secs: config::DEFAULT_KEYBOARD_SEEK_STEP_SECS,
                notifications: &mut notifs,
            });
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
        app.viewer.current_media_path = Some(PathBuf::from("/path/to/image.jpg"));

        let title = app.title();

        // Should show "filename - AppName"
        assert_eq!(title, "image.jpg - IcedLens");
    }

    #[test]
    fn title_shows_asterisk_when_editor_has_unsaved_changes() {
        // Create a real PNG file for the image editor
        let (_temp_dir, img_path, img_data) = create_test_png(4, 3);

        let mut app = App::default();

        // Load image and set the path
        let _ = app.update(Message::Viewer(component::Message::MediaLoaded(Ok(
            MediaData::Image(img_data.clone()),
        ))));
        app.viewer.current_media_path = Some(img_path.clone());

        // Create editor state with actual PNG file
        let editor_state =
            image_editor::State::new(img_path, img_data).expect("create editor state");
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

        // Create a captured frame (4x3 black pixels)
        let rgba_data = vec![0u8; 4 * 3 * 4]; // width * height * 4 channels
        let frame = ExportableFrame::new(rgba_data, 4, 3);
        let video_path = PathBuf::from("/path/to/video.mp4");

        let mut app = App::default();

        // Create editor state from captured frame
        let editor_state = image_editor::State::from_captured_frame(frame, video_path, 5.0)
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

        // Create a captured frame
        let rgba_data = vec![0u8; 4 * 3 * 4];
        let frame = ExportableFrame::new(rgba_data, 4, 3);
        let video_path = PathBuf::from("/path/to/video.mp4");

        let mut app = App::default();

        // Create editor state from captured frame
        let editor_state = image_editor::State::from_captured_frame(frame, video_path, 5.0)
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
