// SPDX-License-Identifier: MPL-2.0
//! Application root state and orchestration between the viewer and settings views.
//!
//! The `App` struct wires together the domains (viewer, localization, settings)
//! and translates messages into side effects like config persistence or image
//! loading. This file intentionally keeps policy decisions (minimum window size,
//! persistence format, localization switching) close to the main update loop so
//! it is easy to audit user-facing behavior.
use crate::config;
use crate::i18n::fluent::I18n;
use crate::image_handler;
use crate::ui::editor::{self, Event as EditorEvent, State as EditorState};
use crate::ui::settings::{
    self, Event as SettingsEvent, State as SettingsState, ViewContext as SettingsViewContext,
};
use crate::ui::state::zoom::{MAX_ZOOM_STEP_PERCENT, MIN_ZOOM_STEP_PERCENT};
use crate::ui::viewer::component;
use iced::{
    event,
    widget::{button, Container, Text},
    window, Element, Length, Subscription, Task, Theme,
};
use std::fmt;
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

pub const MIN_WINDOW_HEIGHT: u32 = 480;

/// Ensures zoom step values stay inside the supported range so persisted
/// configs cannot request nonsensical increments.
fn clamp_zoom_step(value: f32) -> f32 {
    value.clamp(MIN_ZOOM_STEP_PERCENT, MAX_ZOOM_STEP_PERCENT)
}

/// Computes a language-aware minimum window width.
///
/// Iced currently needs an explicit minimum size; we approximate the required
/// width by multiplying the localized control labels by an average monospace
/// character width and adding padding constants for spacing, button chrome,
/// and breathing room. The constants are empirical but centralized here so the
/// rationale is documented.
fn compute_min_window_width(i18n: &I18n) -> u32 {
    const CHAR_W: f32 = 8.0;
    let zoom_input_w = 90.0;
    let parts = [
        i18n.tr("viewer-zoom-label"),
        i18n.tr("viewer-zoom-out-button"),
        i18n.tr("viewer-zoom-reset-button"),
        i18n.tr("viewer-zoom-in-button"),
        i18n.tr("viewer-fit-to-window-toggle"),
    ];
    let text_total: f32 = parts.iter().map(|s| s.len() as f32 * CHAR_W).sum();
    let button_padding = (6.0 * 2.0) * 4.0;
    let extra_label_padding = 12.0;
    let gaps = 10.0 * 6.0;
    let breathing_room = 40.0;
    (text_total + zoom_input_w + button_padding + extra_label_padding + gaps + breathing_room)
        .ceil() as u32
}

/// Builds the window settings using the provided flags so translation data is
/// available during size calculation and icon loading.
pub fn window_settings_with_locale(flags: &Flags) -> window::Settings {
    let config = crate::config::load().unwrap_or_default();
    let i18n = I18n::new(flags.lang.clone(), flags.i18n_dir.clone(), &config);
    let computed_width = compute_min_window_width(&i18n);
    let icon = crate::icon::load_window_icon();

    window::Settings {
        size: iced::Size::new(computed_width as f32, 600.0),
        min_size: Some(iced::Size::new(
            computed_width as f32,
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
        .window(window_settings_with_locale(&flags))
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
                event::Status::Ignored => Some(Message::Viewer(component::Message::RawEvent {
                    window: window_id,
                    event: event.clone(),
                })),
                event::Status::Captured => None,
            }
        })
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Viewer(viewer_message) => self.handle_viewer_message(viewer_message),
            Message::SwitchMode(mode) => self.handle_mode_switch(mode),
            Message::Settings(settings_message) => self.handle_settings_message(settings_message),
            Message::Editor(editor_message) => self.handle_editor_message(editor_message),
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
                self.editor = Some(EditorState::new(image_path, image_data));
                self.mode = mode;
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
                self.editor = None;
                self.mode = AppMode::Viewer;
                Task::none()
            }
            EditorEvent::NavigateNext => {
                // TODO: Implement navigation in editor mode
                Task::none()
            }
            EditorEvent::NavigatePrevious => {
                // TODO: Implement navigation in editor mode
                Task::none()
            }
            EditorEvent::SaveRequested { path, overwrite } => {
                // TODO: Implement save logic
                eprintln!("Save requested: {:?} (overwrite: {})", path, overwrite);
                Task::none()
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
                        .view(editor::ViewContext { i18n: &self.i18n })
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

        let switch_button = match self.mode {
            AppMode::Viewer => button(Text::new(self.i18n.tr("open-settings-button")))
                .on_press(Message::SwitchMode(AppMode::Settings)),
            AppMode::Settings => button(Text::new(self.i18n.tr("back-to-viewer-button")))
                .on_press(Message::SwitchMode(AppMode::Viewer)),
            AppMode::Editor => button(Text::new(self.i18n.tr("editor-cancel")))
                .on_press(Message::SwitchMode(AppMode::Viewer)),
        };

        let mut column = iced::widget::Column::new();
        if !self.fullscreen {
            let mut top_bar = iced::widget::Row::new().spacing(10).padding(10);

            // Add Settings/Back button
            top_bar = top_bar.push(switch_button);

            // Add Edit button (only in Viewer mode and when an image is loaded)
            if self.mode == AppMode::Viewer && self.viewer.has_image() {
                let edit_button = button(Text::new("✏ Edit"))
                    .on_press(Message::SwitchMode(AppMode::Editor));
                top_bar = top_bar.push(edit_button);
            }

            column = column.push(
                Container::new(top_bar)
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Left),
            );
        }

        column = column.push(
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
    fn dynamic_min_width_french_not_smaller_than_english() {
        let english_flags = Flags {
            lang: Some("en-US".into()),
            file_path: None,
            i18n_dir: None,
        };
        let ws_en = window_settings_with_locale(&english_flags);
        let min_en = ws_en.min_size.expect("min size en").width;

        let french_flags = Flags {
            lang: Some("fr".into()),
            file_path: None,
            i18n_dir: None,
        };
        let ws_fr = window_settings_with_locale(&french_flags);
        let min_fr = ws_fr.min_size.expect("min size fr").width;

        assert!(min_fr >= min_en);
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
}
