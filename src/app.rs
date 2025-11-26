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
use crate::image_handler::{self, ImageData};
use crate::ui::editor::{self, Event as EditorEvent, State as EditorState};
use crate::ui::settings::{
    self, Event as SettingsEvent, State as SettingsState, ViewContext as SettingsViewContext,
};
use crate::ui::state::zoom::{MAX_ZOOM_STEP_PERCENT, MIN_ZOOM_STEP_PERCENT};
use crate::ui::viewer::component;
use iced::{
    event,
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
    mode: AppMode,
    settings: SettingsState,
    viewer: component::State,
    editor: Option<EditorState>,
    fullscreen: bool,
    window_id: Option<window::Id>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Screens the user can navigate between.
pub enum AppMode {
    Viewer,
    Settings,
    Editor,
}

impl fmt::Debug for App {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("App")
            .field("mode", &self.mode)
            .field("viewer_has_image", &self.viewer.has_image())
            .finish()
    }
}

/// Top-level messages consumed by [`App::update`]. The variants forward
/// lower-level component messages while keeping a single update entrypoint.
#[derive(Debug, Clone)]
pub enum Message {
    Viewer(component::Message),
    SwitchMode(AppMode),
    Settings(settings::Message),
    Editor(editor::Message),
    EditorImageLoaded(Result<ImageData, Error>),
    SaveAsDialogResult(Option<PathBuf>),
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

pub const WINDOW_DEFAULT_HEIGHT: u32 = 600;
pub const WINDOW_DEFAULT_WIDTH: u32 = 800;
pub const MIN_WINDOW_HEIGHT: u32 = 480;
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
            mode: AppMode::Viewer,
            settings: SettingsState::default(),
            viewer: component::State::new(),
            editor: None,
            fullscreen: false,
            window_id: None,
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
        app.settings = SettingsState::new(app.viewer.zoom_step_percent(), theme, sort_order);

        let task = if let Some(path_str) = flags.file_path {
            let path = std::path::PathBuf::from(&path_str);
            app.viewer.current_image_path = Some(path.clone());
            Task::perform(
                async move { image_handler::load_image(&path_str) },
                |result| Message::Viewer(component::Message::ImageLoaded(result)),
            )
        } else {
            Task::none()
        };

        (app, task)
    }

    fn title(&self) -> String {
        self.i18n.tr("window-title")
    }

    fn theme(&self) -> Theme {
        Theme::default()
    }

    fn subscription(&self) -> Subscription<Message> {
        // Route events based on current mode
        match self.mode {
            AppMode::Editor => {
                event::listen_with(|event, status, window_id| {
                    if let event::Event::Window(window::Event::Resized(_)) = &event {
                        return Some(Message::Viewer(component::Message::RawEvent {
                            window: window_id,
                            event: event.clone(),
                        }));
                    }

                    // In editor mode, route keyboard events to editor
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
                })
            }
            _ => {
                // In viewer or settings mode, route to viewer
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
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Viewer(viewer_message) => self.handle_viewer_message(viewer_message),
            Message::SwitchMode(mode) => self.handle_mode_switch(mode),
            Message::Settings(settings_message) => self.handle_settings_message(settings_message),
            Message::Editor(editor_message) => self.handle_editor_message(editor_message),
            Message::EditorImageLoaded(result) => {
                match result {
                    Ok(image_data) => {
                        // Create a new EditorState with the loaded image
                        if let Some(current_image_path) = self.viewer.current_image_path.clone() {
                            match editor::State::new(current_image_path, image_data) {
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
                self.mode = AppMode::Settings;
                Task::none()
            }
            component::Effect::EnterEditor => self.handle_mode_switch(AppMode::Editor),
            component::Effect::None => Task::none(),
        };
        Task::batch([viewer_task, side_effect])
    }

    fn handle_mode_switch(&mut self, mode: AppMode) -> Task<Message> {
        // Handle Settings → Viewer transition
        if matches!(mode, AppMode::Viewer) && matches!(self.mode, AppMode::Settings) {
            match self.settings.ensure_zoom_step_committed() {
                Ok(Some(value)) => {
                    self.viewer.set_zoom_step_percent(value);
                    self.mode = mode;
                    return self.persist_preferences();
                }
                Ok(None) => {
                    self.mode = mode;
                    return Task::none();
                }
                Err(_) => {
                    self.mode = AppMode::Settings;
                    return Task::none();
                }
            }
        }

        // Handle Viewer → Editor transition
        if matches!(mode, AppMode::Editor) && matches!(self.mode, AppMode::Viewer) {
            if let (Some(image_path), Some(image_data)) = (
                self.viewer.current_image_path.clone(),
                self.viewer.image().cloned(),
            ) {
                match EditorState::new(image_path, image_data) {
                    Ok(state) => {
                        self.editor = Some(state);
                        self.mode = mode;
                    }
                    Err(err) => {
                        eprintln!("Failed to enter editor mode: {err:?}");
                    }
                }
                return Task::none();
            } else {
                // Can't enter editor mode without an image
                return Task::none();
            }
        }

        // Handle Editor → Viewer transition
        if matches!(mode, AppMode::Viewer) && matches!(self.mode, AppMode::Editor) {
            self.editor = None;
            self.mode = mode;
            return Task::none();
        }

        self.mode = mode;
        Task::none()
    }

    fn handle_settings_message(&mut self, message: settings::Message) -> Task<Message> {
        match self.settings.update(message) {
            SettingsEvent::None => Task::none(),
            SettingsEvent::BackToViewer => {
                self.mode = AppMode::Viewer;
                Task::none()
            }
            SettingsEvent::BackToViewerWithZoomChange(value) => {
                self.viewer.set_zoom_step_percent(value);
                self.mode = AppMode::Viewer;
                self.persist_preferences()
            }
            SettingsEvent::LanguageSelected(locale) => self.apply_language_change(locale),
            SettingsEvent::ZoomStepChanged(value) => {
                self.viewer.set_zoom_step_percent(value);
                self.persist_preferences()
            }
            SettingsEvent::BackgroundThemeSelected(_) => self.persist_preferences(),
            SettingsEvent::SortOrderSelected(_) => self.persist_preferences(),
        }
    }

    fn handle_editor_message(&mut self, message: editor::Message) -> Task<Message> {
        let Some(editor_state) = self.editor.as_mut() else {
            return Task::none();
        };

        match editor_state.update(message) {
            EditorEvent::None => Task::none(),
            EditorEvent::ExitEditor => {
                // Get the current image path before dropping the editor
                let current_image_path = editor_state.image_path().to_path_buf();

                self.editor = None;
                self.mode = AppMode::Viewer;

                // Reload the image in the viewer to show any saved changes
                Task::perform(
                    async move { crate::image_handler::load_image(&current_image_path) },
                    |result| Message::Viewer(component::Message::ImageLoaded(result)),
                )
            }
            EditorEvent::NavigateNext => {
                // Rescan directory to handle added/removed images
                let _ = self.viewer.scan_directory();

                // Navigate to next image in the list
                if let Some(next_path) = self.viewer.image_list.next() {
                    let path = next_path.to_path_buf();
                    self.viewer.current_image_path = Some(path.clone());
                    self.viewer.image_list.set_current(&path);

                    // Load the next image and create a new EditorState
                    Task::perform(
                        async move { crate::image_handler::load_image(&path) },
                        Message::EditorImageLoaded,
                    )
                } else {
                    Task::none()
                }
            }
            EditorEvent::NavigatePrevious => {
                // Rescan directory to handle added/removed images
                let _ = self.viewer.scan_directory();

                // Navigate to previous image in the list
                if let Some(prev_path) = self.viewer.image_list.previous() {
                    let path = prev_path.to_path_buf();
                    self.viewer.current_image_path = Some(path.clone());
                    self.viewer.image_list.set_current(&path);

                    // Load the previous image and create a new EditorState
                    Task::perform(
                        async move { crate::image_handler::load_image(&path) },
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
                let current_path = editor_state.image_path().to_path_buf();
                Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .set_file_name(
                                current_path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("image.png"),
                            )
                            .add_filter(
                                "Image Files",
                                &["png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp", "ico"],
                            )
                            .save_file()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    Message::SaveAsDialogResult,
                )
            }
        }
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
        cfg.fit_to_window = Some(self.viewer.fit_to_window());
        cfg.zoom_step = Some(self.viewer.zoom_step_percent());
        cfg.background_theme = Some(self.settings.background_theme());
        cfg.sort_order = Some(self.settings.sort_order());

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
        let current_view: Element<'_, Message> = match self.mode {
            AppMode::Viewer => self
                .viewer
                .view(component::ViewEnv {
                    i18n: &self.i18n,
                    background_theme: self.settings.background_theme(),
                    show_controls: !self.fullscreen,
                })
                .map(Message::Viewer),
            AppMode::Settings => self
                .settings
                .view(SettingsViewContext { i18n: &self.i18n })
                .map(Message::Settings),
            AppMode::Editor => {
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
    use crate::image_handler::ImageData;
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

    fn build_image(width: u32, height: u32) -> ImageData {
        let pixel_count = (width * height * 4) as usize;
        let pixels = vec![255; pixel_count];
        ImageData {
            handle: Handle::from_rgba(width, height, pixels),
            width,
            height,
        }
    }

    #[test]
    fn new_starts_in_viewer_mode_without_image() {
        with_temp_config_dir(|_| {
            let (app, _command) = App::new(Flags {
                lang: None,
                file_path: None,
                i18n_dir: None,
            });
            assert_eq!(app.mode, AppMode::Viewer);
            assert!(!app.viewer.has_image());
        });
    }

    #[test]
    fn update_image_loaded_ok_sets_state() {
        let mut app = App::default();
        let data = sample_image_data();

        let _ = app.update(Message::Viewer(component::Message::ImageLoaded(Ok(
            data.clone()
        ))));

        assert!(app.viewer.has_image());
        assert_eq!(app.viewer.image().unwrap().width, data.width);
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
                mode: AppMode::Settings,
                ..App::default()
            };
            let _ = app.update(Message::Settings(settings::Message::ZoomStepInputChanged(
                "25".into(),
            )));

            let _ = app.update(Message::SwitchMode(AppMode::Viewer));

            assert_eq!(app.mode, AppMode::Viewer);
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
                mode: AppMode::Settings,
                ..App::default()
            };
            let _ = app.update(Message::Settings(settings::Message::ZoomStepInputChanged(
                "not-a-number".into(),
            )));

            let _ = app.update(Message::SwitchMode(AppMode::Viewer));

            assert_eq!(app.mode, AppMode::Settings);
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
                mode: AppMode::Settings,
                ..App::default()
            };
            let _ = app.update(Message::Settings(settings::Message::ZoomStepInputChanged(
                "500".into(),
            )));

            let _ = app.update(Message::SwitchMode(AppMode::Viewer));

            assert_eq!(app.mode, AppMode::Settings);
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
            sample_image_data(),
        ))));

        let _ = app.update(Message::Viewer(component::Message::ImageLoaded(Err(
            Error::Io("boom".into()),
        ))));

        assert!(!app.viewer.has_image());
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
            component::Message::ImageLoaded(Ok(build_image(2000, 1000))),
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
            component::Message::ImageLoaded(Ok(build_image(800, 600))),
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
            component::Message::ImageLoaded(Ok(build_image(800, 600))),
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
            sample_image_data(),
        ))));
        app.viewer.current_image_path = Some(img1_path.clone());
        app.viewer
            .scan_directory()
            .expect("failed to scan directory");

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
            sample_image_data(),
        ))));
        app.viewer.current_image_path = Some(img2_path.clone());
        app.viewer
            .scan_directory()
            .expect("failed to scan directory");

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
            sample_image_data(),
        ))));
        app.viewer.current_image_path = Some(img2_path.clone());
        app.viewer
            .scan_directory()
            .expect("failed to scan directory");

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
                sample_image_data(),
            ))));
            app.viewer.current_image_path = Some(img1_path.clone());
            app.viewer
                .scan_directory()
                .expect("failed to scan directory");

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
        let img1_data = image_handler::load_image(&img1_path).expect("failed to load img1");
        let _ = app.update(Message::Viewer(component::Message::ImageLoaded(Ok(
            img1_data.clone(),
        ))));
        app.viewer.current_image_path = Some(img1_path.clone());
        app.viewer
            .scan_directory()
            .expect("failed to scan directory");

        // Switch to editor mode
        let _ = app.update(Message::SwitchMode(AppMode::Editor));

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
        let img2_data = image_handler::load_image(&img2_path).expect("failed to load img2");
        let _ = app.update(Message::EditorImageLoaded(Ok(img2_data)));

        // Verify editor has loaded the second image
        assert!(app.editor.is_some(), "Editor should still be active");
        if let Some(editor) = &app.editor {
            assert_eq!(
                editor.image_path(),
                img2_path.as_path(),
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
        let img2_data = image_handler::load_image(&img2_path).expect("failed to load img2");
        let _ = app.update(Message::Viewer(component::Message::ImageLoaded(Ok(
            img2_data.clone(),
        ))));
        app.viewer.current_image_path = Some(img2_path.clone());
        app.viewer
            .scan_directory()
            .expect("failed to scan directory");

        // Switch to editor mode
        let _ = app.update(Message::SwitchMode(AppMode::Editor));

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
        let img1_data = image_handler::load_image(&img1_path).expect("failed to load img1");
        let _ = app.update(Message::EditorImageLoaded(Ok(img1_data)));

        // Verify editor has loaded the first image
        assert!(app.editor.is_some(), "Editor should still be active");
        if let Some(editor) = &app.editor {
            assert_eq!(
                editor.image_path(),
                img1_path.as_path(),
                "Editor should have loaded a.png"
            );
        }
    }
}
