// SPDX-License-Identifier: MPL-2.0
//! Application root state and orchestration between the viewer and settings views.
//!
//! The `App` struct wires together the domains (viewer, localization, settings)
//! and translates messages into side effects like config persistence or image
//! loading. This file intentionally keeps policy decisions (minimum window size,
//! persistence format, localization switching) close to the main update loop so
//! it is easy to audit user-facing behavior.
use crate::config;
use crate::error::Error;
use crate::i18n::fluent::I18n;
use crate::image_navigation::ImageNavigator;
use crate::media::{self, MediaData};
use crate::ui::editor::{self, Event as EditorEvent, State as EditorState};
use crate::ui::settings::{
    self, Event as SettingsEvent, State as SettingsState, StateConfig as SettingsConfig,
    ViewContext as SettingsViewContext,
};
use crate::ui::state::zoom::{MAX_ZOOM_STEP_PERCENT, MIN_ZOOM_STEP_PERCENT};
use crate::ui::theming::ThemeMode;
use crate::ui::viewer::component;
use crate::video_player::{create_lufs_cache, SharedLufsCache};
use iced::{
    event, time,
    widget::{Container, Text},
    window, Element, Length, Subscription, Task, Theme,
};
use std::fmt;
use std::path::PathBuf;
use unic_langid::LanguageIdentifier;

/// Root Iced application state that bridges UI components, localization, and
/// persisted preferences.
pub struct App {
    pub i18n: I18n,
    screen: Screen,
    settings: SettingsState,
    viewer: component::State,
    editor: Option<EditorState>,
    image_navigator: ImageNavigator,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Screens the user can navigate between.
pub enum Screen {
    Viewer,
    Settings,
    Editor,
}

impl fmt::Debug for App {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("App")
            .field("screen", &self.screen)
            .field("viewer_has_image", &self.viewer.has_media())
            .finish()
    }
}

/// Top-level messages consumed by [`App::update`]. The variants forward
/// lower-level component messages while keeping a single update entrypoint.
#[derive(Debug, Clone)]
pub enum Message {
    Viewer(component::Message),
    SwitchScreen(Screen),
    Settings(settings::Message),
    Editor(editor::Message),
    EditorImageLoaded(Result<MediaData, Error>),
    SaveAsDialogResult(Option<PathBuf>),
    FrameCaptureDialogResult {
        path: Option<PathBuf>,
        frame: Option<crate::media::frame_export::ExportableFrame>,
    },
    /// Open the editor with a captured video frame.
    OpenEditorWithFrame {
        frame: crate::media::frame_export::ExportableFrame,
        video_path: PathBuf,
        position_secs: f64,
    },
    Tick(std::time::Instant), // Periodic tick for overlay auto-hide
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
    iced::application(|state: &App| state.title(), App::update, App::view)
        .theme(App::theme)
        .window(window_settings_with_locale())
        .subscription(App::subscription)
        .run_with(move || App::new(flags))
}

impl Default for App {
    fn default() -> Self {
        Self {
            i18n: I18n::default(),
            screen: Screen::Viewer,
            settings: SettingsState::default(),
            viewer: component::State::new(),
            editor: None,
            image_navigator: ImageNavigator::new(),
            fullscreen: false,
            window_id: None,
            theme_mode: ThemeMode::System,
            video_autoplay: false,
            audio_normalization: true, // Enabled by default - normalizes audio volume between media files
            lufs_cache: create_lufs_cache(),
            frame_cache_mb: config::DEFAULT_FRAME_CACHE_MB,
        }
    }
}

impl App {
    /// Initializes application state and optionally kicks off asynchronous image
    /// loading based on `Flags` received from the launcher.
    fn new(flags: Flags) -> (Self, Task<Message>) {
        let config = config::load().unwrap_or_default();
        let i18n = I18n::new(flags.lang.clone(), flags.i18n_dir.clone(), &config);

        let mut app = App {
            i18n,
            ..Self::default()
        };

        app.theme_mode = config.theme_mode;

        if let Some(step) = config.zoom_step {
            let clamped = clamp_zoom_step(step);
            app.viewer.set_zoom_step_percent(clamped);
        }

        match config.fit_to_window {
            Some(true) | None => app.viewer.enable_fit_to_window(),
            Some(false) => app.viewer.disable_fit_to_window(),
        }

        let theme = config.background_theme.unwrap_or_default();
        let sort_order = config.sort_order.unwrap_or_default();
        let overlay_timeout_secs = config
            .overlay_timeout_secs
            .unwrap_or(config::DEFAULT_OVERLAY_TIMEOUT_SECS);
        let video_autoplay = config.video_autoplay.unwrap_or(false);
        let audio_normalization = config.audio_normalization.unwrap_or(true);
        let frame_cache_mb = config
            .frame_cache_mb
            .unwrap_or(config::DEFAULT_FRAME_CACHE_MB);
        app.frame_cache_mb = frame_cache_mb;
        app.settings = SettingsState::new(SettingsConfig {
            zoom_step_percent: app.viewer.zoom_step_percent(),
            background_theme: theme,
            sort_order,
            overlay_timeout_secs,
            theme_mode: config.theme_mode,
            video_autoplay,
            audio_normalization,
            frame_cache_mb,
        });
        app.video_autoplay = video_autoplay;
        app.audio_normalization = audio_normalization;
        app.viewer.set_video_autoplay(video_autoplay);

        let task = if let Some(path_str) = flags.file_path {
            let path = std::path::PathBuf::from(&path_str);

            // Initialize ImageNavigator with the initial image
            let config = config::load().unwrap_or_default();
            let sort_order = config.sort_order.unwrap_or_default();
            if let Err(err) = app.image_navigator.scan_directory(&path, sort_order) {
                eprintln!("Failed to scan directory: {:?}", err);
            }

            // Synchronize viewer state
            app.viewer.current_image_path = Some(path.clone());

            // Set loading state directly (before first render)
            app.viewer.is_loading_media = true;
            app.viewer.loading_started_at = Some(std::time::Instant::now());

            // Load the media
            Task::perform(async move { media::load_media(&path_str) }, |result| {
                Message::Viewer(component::Message::ImageLoaded(result))
            })
        } else {
            Task::none()
        };

        (app, task)
    }

    fn title(&self) -> String {
        self.i18n.tr("window-title")
    }

    fn theme(&self) -> Theme {
        match self.theme_mode {
            ThemeMode::Light => Theme::Light,
            ThemeMode::Dark => Theme::Dark,
            ThemeMode::System => Theme::Dark,
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let event_subscription = match self.screen {
            Screen::Editor => event::listen_with(|event, status, window_id| {
                if let event::Event::Window(window::Event::Resized(_)) = &event {
                    return Some(Message::Viewer(component::Message::RawEvent {
                        window: window_id,
                        event: event.clone(),
                    }));
                }

                // In editor screen, route keyboard events to editor
                if let event::Event::Keyboard(..) = &event {
                    match status {
                        event::Status::Ignored => {
                            Some(Message::Editor(crate::ui::editor::Message::RawEvent {
                                window: window_id,
                                event: event.clone(),
                            }))
                        }
                        event::Status::Captured => None,
                    }
                } else {
                    None
                }
            }),
            Screen::Viewer => {
                // In viewer screen, route all events including wheel scroll for zoom
                event::listen_with(|event, status, window_id| {
                    if matches!(
                        event,
                        event::Event::Mouse(iced::mouse::Event::WheelScrolled { .. })
                    ) {
                        return Some(Message::Viewer(component::Message::RawEvent {
                            window: window_id,
                            event: event.clone(),
                        }));
                    }

                    match status {
                        event::Status::Ignored => {
                            Some(Message::Viewer(component::Message::RawEvent {
                                window: window_id,
                                event: event.clone(),
                            }))
                        }
                        event::Status::Captured => None,
                    }
                })
            }
            Screen::Settings => {
                // In settings screen, only route non-wheel events to viewer
                // (wheel events are used by settings scrollable)
                event::listen_with(|event, status, window_id| {
                    // Don't route wheel scroll to viewer - it's used by settings scroll
                    if matches!(
                        event,
                        event::Event::Mouse(iced::mouse::Event::WheelScrolled { .. })
                    ) {
                        return None;
                    }

                    match status {
                        event::Status::Ignored => {
                            Some(Message::Viewer(component::Message::RawEvent {
                                window: window_id,
                                event: event.clone(),
                            }))
                        }
                        event::Status::Captured => None,
                    }
                })
            }
        };

        // Add periodic tick when in fullscreen to update overlay auto-hide
        // or when loading media to check for timeout
        let tick_subscription = if self.fullscreen || self.viewer.is_loading_media() {
            time::every(std::time::Duration::from_millis(100)).map(Message::Tick)
        } else {
            Subscription::none()
        };

        // Add video playback subscription with LUFS cache for audio normalization
        let video_subscription = self
            .viewer
            .subscription(
                Some(self.lufs_cache.clone()),
                self.audio_normalization,
                self.frame_cache_mb,
            )
            .map(Message::Viewer);

        Subscription::batch([event_subscription, tick_subscription, video_subscription])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Viewer(viewer_message) => self.handle_viewer_message(viewer_message),
            Message::SwitchScreen(target) => self.handle_screen_switch(target),
            Message::Settings(settings_message) => self.handle_settings_message(settings_message),
            Message::Editor(editor_message) => self.handle_editor_message(editor_message),
            Message::EditorImageLoaded(result) => {
                match result {
                    Ok(media_data) => {
                        // Editor only supports images in v0.2, not videos
                        // Extract ImageData from MediaData
                        let image_data = match media_data {
                            MediaData::Image(img) => img,
                            MediaData::Video(_) => {
                                // Video editing not supported in v0.2
                                eprintln!("Video editing is not supported in this version");
                                return Task::none();
                            }
                        };

                        // Create a new EditorState with the loaded image
                        if let Some(current_image_path) = self.image_navigator.current_image_path()
                        {
                            let path = current_image_path.to_path_buf();
                            match editor::State::new(path, image_data) {
                                Ok(new_editor_state) => {
                                    self.editor = Some(new_editor_state);
                                }
                                Err(err) => {
                                    eprintln!("Failed to create editor state: {:?}", err);
                                }
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("Failed to load image for editor: {:?}", err);
                    }
                }
                Task::none()
            }
            Message::Tick(_instant) => {
                // Periodic tick for overlay auto-hide - just trigger a view refresh
                // The view() function will check elapsed time and hide controls if needed

                // Also check for loading timeout
                self.viewer.check_loading_timeout(&self.i18n);

                Task::none()
            }
            Message::SaveAsDialogResult(path_opt) => {
                if let Some(path) = path_opt {
                    // User selected a path, save the image there
                    if let Some(editor) = self.editor.as_mut() {
                        match editor.save_image(&path) {
                            Ok(()) => {
                                eprintln!("Image saved successfully to: {:?}", path);
                                // TODO: Show success notification to user
                            }
                            Err(err) => {
                                eprintln!("Failed to save image: {:?}", err);
                                // TODO: Show error notification to user
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
                            eprintln!("Frame captured successfully to: {:?}", path);
                            // TODO: Show success notification to user
                        }
                        Err(err) => {
                            eprintln!("Failed to capture frame: {:?}", err);
                            // TODO: Show error notification to user
                        }
                    }
                }
                Task::none()
            }
            Message::OpenEditorWithFrame {
                frame,
                video_path,
                position_secs,
            } => {
                match EditorState::from_captured_frame(frame, video_path, position_secs) {
                    Ok(state) => {
                        self.editor = Some(state);
                        self.screen = Screen::Editor;
                    }
                    Err(err) => {
                        eprintln!("Failed to open editor with captured frame: {err:?}");
                    }
                }
                Task::none()
            }
        }
    }

    fn handle_viewer_message(&mut self, message: component::Message) -> Task<Message> {
        if let component::Message::RawEvent { window, .. } = &message {
            self.window_id = Some(*window);
        }

        let (effect, task) = self.viewer.handle_message(message, &self.i18n);
        let viewer_task = task.map(Message::Viewer);
        let side_effect = match effect {
            component::Effect::PersistPreferences => self.persist_preferences(),
            component::Effect::ToggleFullscreen => self.toggle_fullscreen_task(),
            component::Effect::ExitFullscreen => self.update_fullscreen_mode(false),
            component::Effect::OpenSettings => {
                self.screen = Screen::Settings;
                Task::none()
            }
            component::Effect::EnterEditor => self.handle_screen_switch(Screen::Editor),
            component::Effect::NavigateNext => self.handle_navigate_next(),
            component::Effect::NavigatePrevious => self.handle_navigate_previous(),
            component::Effect::CaptureFrame {
                frame,
                video_path,
                position_secs,
            } => self.handle_capture_frame(frame, video_path, position_secs),
            component::Effect::None => Task::none(),
        };
        Task::batch([viewer_task, side_effect])
    }

    fn handle_screen_switch(&mut self, target: Screen) -> Task<Message> {
        // Handle Settings → Viewer transition
        if matches!(target, Screen::Viewer) && matches!(self.screen, Screen::Settings) {
            match self.settings.ensure_zoom_step_committed() {
                Ok(Some(value)) => {
                    self.viewer.set_zoom_step_percent(value);
                    self.screen = target;
                    return self.persist_preferences();
                }
                Ok(None) => {
                    self.screen = target;
                    return Task::none();
                }
                Err(_) => {
                    self.screen = Screen::Settings;
                    return Task::none();
                }
            }
        }

        // Handle Viewer → Editor transition
        if matches!(target, Screen::Editor) && matches!(self.screen, Screen::Viewer) {
            if let (Some(image_path), Some(media_data)) = (
                self.viewer.current_image_path.clone(),
                self.viewer.media().cloned(),
            ) {
                // Editor only supports images in v0.2, not videos
                let image_data = match media_data {
                    MediaData::Image(img) => img,
                    MediaData::Video(_) => {
                        eprintln!("Video editing is not supported in this version");
                        return Task::none();
                    }
                };

                // Synchronize image_navigator with viewer state before entering editor
                let config = config::load().unwrap_or_default();
                let sort_order = config.sort_order.unwrap_or_default();
                if let Err(err) = self.image_navigator.scan_directory(&image_path, sort_order) {
                    eprintln!("Failed to scan directory: {:?}", err);
                }

                match EditorState::new(image_path, image_data) {
                    Ok(state) => {
                        self.editor = Some(state);
                        self.screen = target;
                    }
                    Err(err) => {
                        eprintln!("Failed to enter editor screen: {err:?}");
                    }
                }
                return Task::none();
            } else {
                // Can't enter editor screen without an image
                return Task::none();
            }
        }

        // Handle Editor → Viewer transition
        if matches!(target, Screen::Viewer) && matches!(self.screen, Screen::Editor) {
            self.editor = None;
            self.screen = target;
            return Task::none();
        }

        self.screen = target;
        Task::none()
    }

    fn handle_settings_message(&mut self, message: settings::Message) -> Task<Message> {
        match self.settings.update(message) {
            SettingsEvent::None => Task::none(),
            SettingsEvent::BackToViewer => {
                self.screen = Screen::Viewer;
                Task::none()
            }
            SettingsEvent::BackToViewerWithZoomChange(value) => {
                self.viewer.set_zoom_step_percent(value);
                self.screen = Screen::Viewer;
                self.persist_preferences()
            }
            SettingsEvent::LanguageSelected(locale) => self.apply_language_change(locale),
            SettingsEvent::ZoomStepChanged(value) => {
                self.viewer.set_zoom_step_percent(value);
                self.persist_preferences()
            }
            SettingsEvent::BackgroundThemeSelected(_) => self.persist_preferences(),
            SettingsEvent::ThemeModeSelected(mode) => {
                self.theme_mode = mode;
                self.persist_preferences()
            }
            SettingsEvent::SortOrderSelected(_) => self.persist_preferences(),
            SettingsEvent::OverlayTimeoutChanged(_) => self.persist_preferences(),
            SettingsEvent::VideoAutoplayChanged(enabled) => {
                self.video_autoplay = enabled;
                self.viewer.set_video_autoplay(enabled);
                self.persist_preferences()
            }
            SettingsEvent::AudioNormalizationChanged(enabled) => {
                self.audio_normalization = enabled;
                self.persist_preferences()
            }
            SettingsEvent::FrameCacheMbChanged(mb) => {
                self.frame_cache_mb = mb;
                self.persist_preferences()
            }
        }
    }

    fn handle_editor_message(&mut self, message: editor::Message) -> Task<Message> {
        let Some(editor_state) = self.editor.as_mut() else {
            return Task::none();
        };

        match editor_state.update(message) {
            EditorEvent::None => Task::none(),
            EditorEvent::ExitEditor => {
                // Get the image source before dropping the editor
                let image_source = editor_state.image_source().clone();

                self.editor = None;
                self.screen = Screen::Viewer;

                // For file mode: reload the image in the viewer to show any saved changes
                // For captured frame mode: just return to viewer without reloading
                match image_source {
                    editor::ImageSource::File(current_image_path) => {
                        // Set loading state directly (before render)
                        self.viewer.is_loading_media = true;
                        self.viewer.loading_started_at = Some(std::time::Instant::now());

                        // Reload the image in the viewer to show any saved changes
                        Task::perform(
                            async move { crate::media::load_media(&current_image_path) },
                            |result| Message::Viewer(component::Message::ImageLoaded(result)),
                        )
                    }
                    editor::ImageSource::CapturedFrame { .. } => {
                        // Just return to viewer, no need to reload anything
                        Task::none()
                    }
                }
            }
            EditorEvent::NavigateNext => {
                // Rescan directory to handle added/removed images
                if let Some(current_path) = self
                    .image_navigator
                    .current_image_path()
                    .map(|p| p.to_path_buf())
                {
                    let config = config::load().unwrap_or_default();
                    let sort_order = config.sort_order.unwrap_or_default();
                    let _ = self
                        .image_navigator
                        .scan_directory(&current_path, sort_order);
                }

                // Navigate to next image in the list
                if let Some(next_path) = self.image_navigator.navigate_next() {
                    // Synchronize viewer state immediately
                    self.viewer.current_image_path = Some(next_path.clone());
                    self.viewer.image_list.set_current(&next_path);

                    // Load the next image and create a new EditorState
                    Task::perform(
                        async move { crate::media::load_media(&next_path) },
                        Message::EditorImageLoaded,
                    )
                } else {
                    Task::none()
                }
            }
            EditorEvent::NavigatePrevious => {
                // Rescan directory to handle added/removed images
                if let Some(current_path) = self
                    .image_navigator
                    .current_image_path()
                    .map(|p| p.to_path_buf())
                {
                    let config = config::load().unwrap_or_default();
                    let sort_order = config.sort_order.unwrap_or_default();
                    let _ = self
                        .image_navigator
                        .scan_directory(&current_path, sort_order);
                }

                // Navigate to previous image in the list
                if let Some(prev_path) = self.image_navigator.navigate_previous() {
                    // Synchronize viewer state immediately
                    self.viewer.current_image_path = Some(prev_path.clone());
                    self.viewer.image_list.set_current(&prev_path);

                    // Load the previous image and create a new EditorState
                    Task::perform(
                        async move { crate::media::load_media(&prev_path) },
                        Message::EditorImageLoaded,
                    )
                } else {
                    Task::none()
                }
            }
            EditorEvent::SaveRequested { path, overwrite: _ } => {
                // Save the edited image
                if let Some(editor) = self.editor.as_mut() {
                    match editor.save_image(&path) {
                        Ok(()) => {
                            eprintln!("Image saved successfully to: {:?}", path);
                            // TODO: Show success notification to user
                        }
                        Err(err) => {
                            eprintln!("Failed to save image: {:?}", err);
                            // TODO: Show error notification to user
                        }
                    }
                }
                Task::none()
            }
            EditorEvent::SaveAsRequested => {
                // Open file picker dialog for "Save As"
                use crate::media::frame_export::{generate_default_filename, ExportFormat};

                let image_source = editor_state.image_source().clone();
                let export_format = editor_state.export_format();

                // Generate filename and filter based on image source
                let (filename, filter_name, filter_ext) = match &image_source {
                    editor::ImageSource::File(path) => {
                        let filename = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("image.png")
                            .to_string();
                        (
                            filename,
                            "Image Files",
                            vec!["png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp", "ico"],
                        )
                    }
                    editor::ImageSource::CapturedFrame {
                        video_path,
                        position_secs,
                    } => {
                        // Use selected export format for captured frames
                        let filename =
                            generate_default_filename(video_path, *position_secs, export_format);
                        let (filter_name, filter_ext) = match export_format {
                            ExportFormat::Png => ("PNG Image", vec!["png"]),
                            ExportFormat::Jpeg => ("JPEG Image", vec!["jpg", "jpeg"]),
                            ExportFormat::WebP => ("WebP Image", vec!["webp"]),
                        };
                        (filename, filter_name, filter_ext)
                    }
                };

                Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .set_file_name(&filename)
                            .add_filter(filter_name, &filter_ext)
                            .save_file()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    Message::SaveAsDialogResult,
                )
            }
        }
    }

    fn handle_navigate_next(&mut self) -> Task<Message> {
        // Rescan directory to handle added/removed images
        if let Some(current_path) = self
            .image_navigator
            .current_image_path()
            .map(|p| p.to_path_buf())
        {
            let config = config::load().unwrap_or_default();
            let sort_order = config.sort_order.unwrap_or_default();
            let _ = self
                .image_navigator
                .scan_directory(&current_path, sort_order);
        }

        // Navigate to next image
        if let Some(next_path) = self.image_navigator.navigate_next() {
            // Synchronize viewer state from navigator
            self.viewer.current_image_path = Some(next_path.clone());
            // Also sync viewer.image_list from viewer.current_image_path
            let _ = self.viewer.scan_directory();

            // Set loading state directly (before render)
            self.viewer.is_loading_media = true;
            self.viewer.loading_started_at = Some(std::time::Instant::now());

            // Load the next image
            Task::perform(
                async move { crate::media::load_media(&next_path) },
                |result| Message::Viewer(component::Message::ImageLoaded(result)),
            )
        } else {
            Task::none()
        }
    }

    fn handle_navigate_previous(&mut self) -> Task<Message> {
        // Rescan directory to handle added/removed images
        if let Some(current_path) = self
            .image_navigator
            .current_image_path()
            .map(|p| p.to_path_buf())
        {
            let config = config::load().unwrap_or_default();
            let sort_order = config.sort_order.unwrap_or_default();
            let _ = self
                .image_navigator
                .scan_directory(&current_path, sort_order);
        }

        // Navigate to previous image
        if let Some(prev_path) = self.image_navigator.navigate_previous() {
            // Synchronize viewer state from navigator
            self.viewer.current_image_path = Some(prev_path.clone());
            // Also sync viewer.image_list from viewer.current_image_path
            let _ = self.viewer.scan_directory();

            // Set loading state directly (before render)
            self.viewer.is_loading_media = true;
            self.viewer.loading_started_at = Some(std::time::Instant::now());

            // Load the previous image
            Task::perform(
                async move { crate::media::load_media(&prev_path) },
                |result| Message::Viewer(component::Message::ImageLoaded(result)),
            )
        } else {
            Task::none()
        }
    }

    /// Handles frame capture: opens the editor with the captured frame.
    fn handle_capture_frame(
        &self,
        frame: crate::media::frame_export::ExportableFrame,
        video_path: PathBuf,
        position_secs: f64,
    ) -> Task<Message> {
        Task::done(Message::OpenEditorWithFrame {
            frame,
            video_path,
            position_secs,
        })
    }

    /// Applies the newly selected locale, persists it to config, and refreshes
    /// any visible error strings that depend on localization.
    fn apply_language_change(&mut self, locale: LanguageIdentifier) -> Task<Message> {
        self.i18n.set_locale(locale.clone());

        let mut cfg = config::load().unwrap_or_default();
        cfg.language = Some(locale.to_string());

        if let Err(error) = config::save(&cfg) {
            eprintln!("Failed to save config: {:?}", error);
        }

        self.viewer.refresh_error_translation(&self.i18n);
        Task::none()
    }

    /// Persists the current viewer + settings preferences to disk.
    ///
    /// Guarded during tests to keep isolation: unit tests exercise the logic by
    /// calling the function directly rather than through `Effect`s.
    fn persist_preferences(&self) -> Task<Message> {
        if cfg!(test) {
            return Task::none();
        }

        let mut cfg = config::load().unwrap_or_default();
        // Use image_fit_to_window() to only persist the image setting, not video
        cfg.fit_to_window = Some(self.viewer.image_fit_to_window());
        cfg.zoom_step = Some(self.viewer.zoom_step_percent());
        cfg.background_theme = Some(self.settings.background_theme());
        cfg.sort_order = Some(self.settings.sort_order());
        cfg.overlay_timeout_secs = Some(self.settings.overlay_timeout_secs());
        cfg.theme_mode = self.theme_mode;
        cfg.video_autoplay = Some(self.video_autoplay);
        cfg.audio_normalization = Some(self.audio_normalization);
        cfg.frame_cache_mb = Some(self.frame_cache_mb);

        if let Err(error) = config::save(&cfg) {
            eprintln!("Failed to save config: {:?}", error);
        }

        Task::none()
    }

    fn toggle_fullscreen_task(&mut self) -> Task<Message> {
        self.update_fullscreen_mode(!self.fullscreen)
    }

    fn update_fullscreen_mode(&mut self, desired: bool) -> Task<Message> {
        if self.fullscreen == desired {
            return Task::none();
        }

        let Some(window_id) = self.window_id else {
            return Task::none();
        };

        self.fullscreen = desired;
        let mode = if desired {
            window::Mode::Fullscreen
        } else {
            window::Mode::Windowed
        };
        window::change_mode::<Message>(window_id, mode)
    }

    fn view(&self) -> Element<'_, Message> {
        let current_view: Element<'_, Message> = match self.screen {
            Screen::Viewer => {
                let config = config::load().unwrap_or_default();
                let overlay_timeout_secs = config
                    .overlay_timeout_secs
                    .unwrap_or(config::DEFAULT_OVERLAY_TIMEOUT_SECS);

                self.viewer
                    .view(component::ViewEnv {
                        i18n: &self.i18n,
                        background_theme: self.settings.background_theme(),
                        is_fullscreen: self.fullscreen,
                        overlay_hide_delay: std::time::Duration::from_secs(
                            overlay_timeout_secs as u64,
                        ),
                    })
                    .map(Message::Viewer)
            }
            Screen::Settings => self
                .settings
                .view(SettingsViewContext { i18n: &self.i18n })
                .map(Message::Settings),
            Screen::Editor => {
                if let Some(editor_state) = &self.editor {
                    editor_state
                        .view(editor::ViewContext {
                            i18n: &self.i18n,
                            background_theme: self.settings.background_theme(),
                        })
                        .map(Message::Editor)
                } else {
                    // Fallback if editor state is missing
                    Container::new(Text::new("Editor error"))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                }
            }
        };

        let column = iced::widget::Column::new().push(
            Container::new(current_view)
                .width(Length::Fill)
                .height(Length::Fill),
        );

        Container::new(column.width(Length::Fill).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DEFAULT_ZOOM_STEP_PERCENT;
    use crate::error::Error;
    use crate::media::ImageData;
    use crate::ui::state::zoom::{
        format_number, DEFAULT_ZOOM_PERCENT, MAX_ZOOM_PERCENT, ZOOM_STEP_INVALID_KEY,
        ZOOM_STEP_RANGE_KEY,
    };
    use crate::ui::viewer::controls;
    use iced::widget::image::Handle;
    use iced::widget::scrollable::AbsoluteOffset;
    use iced::{event, keyboard, mouse, window, Point, Rectangle, Size};
    use std::fs;
    use std::path::Path;
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

    #[test]
    fn new_starts_in_viewer_mode_without_image() {
        with_temp_config_dir(|_| {
            let (app, _command) = App::new(Flags {
                lang: None,
                file_path: None,
                i18n_dir: None,
            });
            assert_eq!(app.screen, Screen::Viewer);
            assert!(!app.viewer.has_media());
        });
    }

    #[test]
    fn update_image_loaded_ok_sets_state() {
        let mut app = App::default();
        let data = sample_image_data();

        let _ = app.update(Message::Viewer(component::Message::ImageLoaded(Ok(
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
    fn update_image_loaded_err_clears_image_and_sets_error() {
        let mut app = App::default();
        let _ = app.update(Message::Viewer(component::Message::ImageLoaded(Ok(
            sample_media_data(),
        ))));

        let _ = app.update(Message::Viewer(component::Message::ImageLoaded(Err(
            Error::Io("boom".into()),
        ))));

        assert!(!app.viewer.has_media());
        assert!(app
            .viewer
            .error()
            .map(|state| state.details().contains("boom"))
            .unwrap_or(false));
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
            component::Message::ImageLoaded(Ok(build_media(2000, 1000))),
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
            component::Message::ImageLoaded(Ok(build_media(800, 600))),
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
        let zoom = app.viewer.zoom_state_mut();
        zoom.zoom_percent = 150.0;
        zoom.manual_zoom_percent = 150.0;
        zoom.fit_to_window = false;
        let _ = app.viewer.handle_message(
            component::Message::ImageLoaded(Ok(build_media(800, 600))),
            &app.i18n,
        );
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
            let target_locale: LanguageIdentifier = app
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
            let app = App::default();
            let settings_dir = config_root.join("IcedLens");
            fs::create_dir_all(&settings_dir).expect("dir");
            fs::create_dir_all(settings_dir.join("settings.toml"))
                .expect("create conflicting directory");

            let _ = app.persist_preferences();
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
        let _ = app.update(Message::Viewer(component::Message::ImageLoaded(Ok(
            sample_media_data(),
        ))));
        app.viewer.current_image_path = Some(img1_path.clone());
        app.viewer
            .scan_directory()
            .expect("failed to scan directory");

        // Also initialize image_navigator
        let _ = app
            .image_navigator
            .scan_directory(&img1_path, crate::config::SortOrder::Alphabetical);

        let _ = app.update(Message::Viewer(component::Message::NavigateNext));

        assert!(app
            .viewer
            .current_image_path
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
        let _ = app.update(Message::Viewer(component::Message::ImageLoaded(Ok(
            sample_media_data(),
        ))));
        app.viewer.current_image_path = Some(img2_path.clone());
        app.viewer
            .scan_directory()
            .expect("failed to scan directory");

        // Also initialize image_navigator
        let _ = app
            .image_navigator
            .scan_directory(&img2_path, crate::config::SortOrder::Alphabetical);

        let _ = app.update(Message::Viewer(component::Message::NavigatePrevious));

        assert!(app
            .viewer
            .current_image_path
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
        let _ = app.update(Message::Viewer(component::Message::ImageLoaded(Ok(
            sample_media_data(),
        ))));
        app.viewer.current_image_path = Some(img2_path.clone());
        app.viewer
            .scan_directory()
            .expect("failed to scan directory");

        // Also initialize image_navigator
        let _ = app
            .image_navigator
            .scan_directory(&img2_path, crate::config::SortOrder::Alphabetical);

        let _ = app.update(Message::Viewer(component::Message::NavigateNext));

        assert!(app
            .viewer
            .current_image_path
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
            let _ = app.update(Message::Viewer(component::Message::ImageLoaded(Ok(
                sample_media_data(),
            ))));
            app.viewer.current_image_path = Some(img1_path.clone());
            app.viewer
                .scan_directory()
                .expect("failed to scan directory");

            // Also initialize image_navigator
            let _ = app
                .image_navigator
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
                }),
            }));

            assert!(app
                .viewer
                .current_image_path
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
        let _ = app.update(Message::Viewer(component::Message::ImageLoaded(Ok(
            img1_data.clone(),
        ))));
        app.viewer.current_image_path = Some(img1_path.clone());
        app.viewer
            .scan_directory()
            .expect("failed to scan directory");

        // Switch to editor screen
        let _ = app.update(Message::SwitchScreen(Screen::Editor));

        // Navigate to next image
        let _ = app.update(Message::Editor(editor::Message::Sidebar(
            crate::ui::editor::SidebarMessage::NavigateNext,
        )));

        // Verify the viewer's current image path has changed to the next image
        assert!(app
            .viewer
            .current_image_path
            .as_ref()
            .map(|p| p.ends_with("b.png"))
            .unwrap_or(false));

        // Simulate the async image loading completing
        let img2_data = media::load_media(&img2_path).expect("failed to load img2");
        let _ = app.update(Message::EditorImageLoaded(Ok(img2_data)));

        // Verify editor has loaded the second image
        assert!(app.editor.is_some(), "Editor should still be active");
        if let Some(editor) = &app.editor {
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
        let _ = app.update(Message::Viewer(component::Message::ImageLoaded(Ok(
            img2_data.clone(),
        ))));
        app.viewer.current_image_path = Some(img2_path.clone());
        app.viewer
            .scan_directory()
            .expect("failed to scan directory");

        // Switch to editor screen
        let _ = app.update(Message::SwitchScreen(Screen::Editor));

        // Navigate to previous image
        let _ = app.update(Message::Editor(editor::Message::Sidebar(
            crate::ui::editor::SidebarMessage::NavigatePrevious,
        )));

        // Verify the viewer's current image path has changed to the previous image
        assert!(app
            .viewer
            .current_image_path
            .as_ref()
            .map(|p| p.ends_with("a.png"))
            .unwrap_or(false));

        // Simulate the async image loading completing
        let img1_data = media::load_media(&img1_path).expect("failed to load img1");
        let _ = app.update(Message::EditorImageLoaded(Ok(img1_data)));

        // Verify editor has loaded the first image
        assert!(app.editor.is_some(), "Editor should still be active");
        if let Some(editor) = &app.editor {
            assert_eq!(
                editor.image_path(),
                Some(img1_path.as_path()),
                "Editor should have loaded a.png"
            );
        }
    }
}
