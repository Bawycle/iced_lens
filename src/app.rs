// SPDX-License-Identifier: MPL-2.0
use crate::config;
use crate::error::Error;
use crate::i18n::fluent::I18n;
use crate::image_handler::{self, ImageData};
use crate::ui::settings::{
    self, Event as SettingsEvent, State as SettingsState, ViewContext as SettingsViewContext,
};
use crate::ui::state::zoom::{
    DEFAULT_ZOOM_PERCENT, MAX_ZOOM_STEP_PERCENT, MIN_ZOOM_STEP_PERCENT, ZOOM_INPUT_INVALID_KEY,
};
use crate::ui::state::{DragState, ViewportState, ZoomState};
use crate::ui::viewer::{self, controls as viewer_controls};
use iced::widget::scrollable::{self, AbsoluteOffset, Id, RelativeOffset};
use iced::{
    event, keyboard, mouse,
    widget::{button, Container, Text},
    window, Element, Length, Point, Rectangle, Subscription, Task, Theme,
};
use std::fmt;
use unic_langid::LanguageIdentifier;

pub struct App {
    image: Option<ImageData>,
    error: Option<ErrorState>,
    pub i18n: I18n, // Made public
    mode: AppMode,
    zoom: ZoomState,
    settings: SettingsState,
    viewport: ViewportState,
    drag: DragState,
    modifiers: keyboard::Modifiers,
    cursor_position: Option<Point>,
}

const VIEWER_SCROLLABLE_ID: &str = "viewer-image-scrollable";

fn parse_number(input: &str) -> Option<f32> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let without_percent = trimmed.trim_end_matches('%').trim();
    if without_percent.is_empty() {
        return None;
    }

    let normalized = without_percent.replace(',', ".");
    let candidate = normalized.trim();
    let value = candidate.parse::<f32>().ok()?;

    if !value.is_finite() {
        return None;
    }

    Some(value)
}

fn clamp_zoom_step(value: f32) -> f32 {
    value.clamp(MIN_ZOOM_STEP_PERCENT, MAX_ZOOM_STEP_PERCENT)
}

fn scroll_steps(delta: &mouse::ScrollDelta) -> f32 {
    match delta {
        mouse::ScrollDelta::Lines { y, .. } => *y,
        mouse::ScrollDelta::Pixels { y, .. } => *y / 120.0,
    }
}

#[derive(Debug, Clone)]
struct ErrorState {
    friendly_key: &'static str,
    friendly_text: String,
    details: String,
    show_details: bool,
}

impl ErrorState {
    fn from_error(error: &Error, i18n: &I18n) -> Self {
        let friendly_key = match error {
            Error::Io(_) => "error-load-image-io",
            Error::Svg(_) => "error-load-image-svg",
            #[allow(unreachable_patterns)]
            _ => "error-load-image-general",
        };

        Self {
            friendly_key,
            friendly_text: i18n.tr(friendly_key),
            details: error.to_string(),
            show_details: false,
        }
    }

    fn refresh_translation(&mut self, i18n: &I18n) {
        self.friendly_text = i18n.tr(self.friendly_key);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Viewer,
    Settings,
}

impl fmt::Debug for App {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("App")
            .field("image", &self.image)
            .field("error", &self.error)
            .field("i18n", &"I18n instance (omitted for brevity)")
            .field("mode", &self.mode)
            .finish()
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            image: None,
            error: None,
            i18n: I18n::default(),
            mode: AppMode::Viewer,
            zoom: ZoomState::default(),
            settings: SettingsState::default(),
            viewport: ViewportState::default(),
            drag: DragState::default(),
            modifiers: keyboard::Modifiers::default(),
            cursor_position: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ImageLoaded(Result<ImageData, Error>),
    SwitchMode(AppMode),
    ToggleErrorDetails,
    ViewerControls(viewer_controls::Message),
    Settings(settings::Message),
    ViewportChanged {
        bounds: Rectangle,
        offset: AbsoluteOffset,
    },
    MouseButtonPressed {
        button: mouse::Button,
        position: Point,
    },
    MouseButtonReleased {
        button: mouse::Button,
    },
    CursorMovedDuringDrag {
        position: Point,
    },
    RawEvent(event::Event),
}

#[derive(Debug, Default)]
pub struct Flags {
    pub lang: Option<String>,
    pub file_path: Option<String>,
    pub i18n_dir: Option<String>,
}

pub const MIN_WINDOW_HEIGHT: u32 = 480; // Provide comfortable viewport area

fn compute_min_window_width(i18n: &I18n) -> u32 {
    // Approximate average character width for UI font.
    const CHAR_W: f32 = 8.0;
    let zoom_input_w = 90.0; // fixed text input width
    let parts = [
        i18n.tr("viewer-zoom-label"),
        i18n.tr("viewer-zoom-out-button"),
        i18n.tr("viewer-zoom-reset-button"),
        i18n.tr("viewer-zoom-in-button"),
        i18n.tr("viewer-fit-to-window-toggle"),
    ];
    let text_total: f32 = parts.iter().map(|s| s.len() as f32 * CHAR_W).sum();
    let button_padding = (6.0 * 2.0) * 4.0; // each of 4 buttons
    let extra_label_padding = 12.0; // label + checkbox overhead
    let gaps = 10.0 * 6.0; // spacing between 6 joins
    let breathing_room = 40.0; // avoid cramped layout
    let total =
        text_total + zoom_input_w + button_padding + extra_label_padding + gaps + breathing_room;
    total.ceil() as u32
}

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

pub fn run(flags: Flags) -> iced::Result {
    iced::application(|state: &App| state.title(), App::update, App::view)
        .theme(App::theme)
        .window(window_settings_with_locale(&flags))
        .subscription(App::subscription)
        .run_with(move || App::new(flags))
}

impl App {
    fn new(flags: Flags) -> (Self, Task<Message>) {
        let config = config::load().unwrap_or_default();
        let i18n = I18n::new(flags.lang.clone(), flags.i18n_dir.clone(), &config);

        let task = if let Some(path) = flags.file_path {
            Task::perform(
                async move { image_handler::load_image(&path) },
                Message::ImageLoaded,
            )
        } else {
            Task::none()
        };

        let mut app = App {
            i18n,
            mode: AppMode::Viewer,
            ..Self::default()
        };

        if let Some(fit) = config.fit_to_window {
            if fit {
                app.zoom.enable_fit_to_window();
            } else {
                app.zoom.disable_fit_to_window();
            }
        }

        if let Some(step) = config.zoom_step {
            let clamped = clamp_zoom_step(step);
            app.zoom.zoom_step_percent = clamped;
        }

        let theme = config
            .background_theme
            .unwrap_or_else(config::BackgroundTheme::default);
        app.settings = SettingsState::new(app.zoom.zoom_step_percent, theme);

        if app.zoom.fit_to_window {
            app.refresh_fit_zoom();
        } else {
            app.zoom.disable_fit_to_window();
        }

        (app, task)
    }

    fn title(&self) -> String {
        self.i18n.tr("window-title")
    }

    fn theme(&self) -> Theme {
        Theme::default()
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, status, _window| {
            // Capture all wheel scroll events to prevent Scrollable from processing them
            // We handle wheel events ourselves for zoom only
            if matches!(
                event,
                event::Event::Mouse(mouse::Event::WheelScrolled { .. })
            ) {
                // Always capture wheel events, regardless of status
                // This prevents the Scrollable widget from receiving them
                return Some(Message::RawEvent(event.clone()));
            }

            // For other events, only process if not already handled
            match status {
                event::Status::Ignored => Some(Message::RawEvent(event.clone())),
                event::Status::Captured => None,
            }
        })
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ImageLoaded(Ok(image_data)) => {
                self.image = Some(image_data);
                self.error = None;
                self.refresh_fit_zoom();
                Task::none()
            }
            Message::ImageLoaded(Err(error)) => {
                self.image = None;
                self.error = Some(ErrorState::from_error(&error, &self.i18n));
                Task::none()
            }
            Message::SwitchMode(mode) => {
                if matches!(mode, AppMode::Viewer) && matches!(self.mode, AppMode::Settings) {
                    match self.settings.ensure_zoom_step_committed() {
                        Ok(Some(value)) => {
                            self.zoom.zoom_step_percent = value;
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

                self.mode = mode;
                Task::none()
            }
            Message::ToggleErrorDetails => {
                if let Some(error_state) = &mut self.error {
                    error_state.show_details = !error_state.show_details;
                }
                Task::none()
            }
            Message::ViewerControls(controls_message) => {
                self.handle_viewer_controls_message(controls_message)
            }
            Message::Settings(settings_message) => self.handle_settings_message(settings_message),
            Message::ViewportChanged { bounds, offset } => {
                self.viewport.update(bounds, offset);
                self.refresh_fit_zoom();
                Task::none()
            }
            Message::MouseButtonPressed { button, position } => {
                self.handle_mouse_button_pressed(button, position)
            }
            Message::MouseButtonReleased { button } => self.handle_mouse_button_released(button),
            Message::CursorMovedDuringDrag { position } => {
                self.handle_cursor_moved_during_drag(position)
            }
            Message::RawEvent(event) => self.handle_raw_event(event),
        }
    }

    fn handle_settings_message(&mut self, message: settings::Message) -> Task<Message> {
        match self.settings.update(message) {
            SettingsEvent::None => Task::none(),
            SettingsEvent::LanguageSelected(locale) => self.apply_language_change(locale),
            SettingsEvent::ZoomStepChanged(value) => {
                self.zoom.zoom_step_percent = value;
                self.persist_preferences()
            }
            SettingsEvent::BackgroundThemeSelected(_) => self.persist_preferences(),
        }
    }

    fn handle_viewer_controls_message(
        &mut self,
        message: viewer_controls::Message,
    ) -> Task<Message> {
        match message {
            viewer_controls::Message::ZoomInputChanged(value) => {
                self.zoom.zoom_input = value;
                self.zoom.zoom_input_dirty = true;
                self.zoom.zoom_input_error_key = None;
                Task::none()
            }
            viewer_controls::Message::ZoomInputSubmitted => {
                self.zoom.zoom_input_dirty = false;

                if let Some(value) = parse_number(&self.zoom.zoom_input) {
                    self.zoom.apply_manual_zoom(value);
                    self.persist_preferences()
                } else {
                    self.zoom.zoom_input_error_key = Some(ZOOM_INPUT_INVALID_KEY);
                    Task::none()
                }
            }
            viewer_controls::Message::ResetZoom => {
                self.zoom.apply_manual_zoom(DEFAULT_ZOOM_PERCENT);
                self.persist_preferences()
            }
            viewer_controls::Message::ZoomIn => {
                self.zoom
                    .apply_manual_zoom(self.zoom.zoom_percent + self.zoom.zoom_step_percent);
                self.persist_preferences()
            }
            viewer_controls::Message::ZoomOut => {
                self.zoom
                    .apply_manual_zoom(self.zoom.zoom_percent - self.zoom.zoom_step_percent);
                self.persist_preferences()
            }
            viewer_controls::Message::SetFitToWindow(fit) => {
                if fit {
                    self.zoom.enable_fit_to_window();
                    self.refresh_fit_zoom();
                } else {
                    self.zoom.disable_fit_to_window();
                }
                self.persist_preferences()
            }
        }
    }

    fn apply_language_change(&mut self, locale: LanguageIdentifier) -> Task<Message> {
        self.i18n.set_locale(locale.clone());

        let mut config = config::load().unwrap_or_default();
        config.language = Some(locale.to_string());

        if let Err(e) = config::save(&config) {
            eprintln!("Failed to save config: {:?}", e);
        }

        if let Some(error_state) = &mut self.error {
            error_state.refresh_translation(&self.i18n);
        }

        Task::none()
    }

    fn refresh_fit_zoom(&mut self) {
        if self.zoom.fit_to_window {
            if let Some(fit_zoom) = self.viewer_state().compute_fit_zoom_percent() {
                self.zoom.update_zoom_display(fit_zoom);
                self.zoom.zoom_input_dirty = false;
                self.zoom.zoom_input_error_key = None;
            }
        }
    }

    fn viewer_state(&self) -> viewer::state::ViewerState<'_> {
        viewer::state::ViewerState::new(
            self.image.as_ref(),
            &self.viewport,
            self.zoom.zoom_percent,
            self.cursor_position,
        )
    }

    fn persist_preferences(&self) -> Task<Message> {
        if cfg!(test) {
            return Task::none();
        }

        let mut config = config::load().unwrap_or_default();
        config.fit_to_window = Some(self.zoom.fit_to_window);
        config.zoom_step = Some(self.zoom.zoom_step_percent);
        config.background_theme = Some(self.settings.background_theme());

        if let Err(error) = config::save(&config) {
            eprintln!("Failed to save config: {:?}", error);
        }

        Task::none()
    }

    fn handle_mouse_button_pressed(
        &mut self,
        button: mouse::Button,
        position: Point,
    ) -> Task<Message> {
        // Only start drag on left button press over the image
        if button == mouse::Button::Left && self.viewer_state().is_cursor_over_image() {
            self.drag.start(position, self.viewport.offset);
        }
        Task::none()
    }

    fn handle_mouse_button_released(&mut self, button: mouse::Button) -> Task<Message> {
        // Stop dragging on left button release
        if button == mouse::Button::Left {
            self.drag.stop();
        }
        Task::none()
    }

    fn handle_cursor_moved_during_drag(&mut self, position: Point) -> Task<Message> {
        // Only update offset if currently dragging
        let proposed_offset = match self.drag.calculate_offset(position) {
            Some(offset) => offset,
            None => return Task::none(),
        };

        // Convert absolute offset to relative offset (0.0-1.0 range)
        let viewer_state = self.viewer_state();
        if let (Some(viewport), Some(size)) = (self.viewport.bounds, viewer_state.scaled_image_size()) {
            let max_offset_x = (size.width - viewport.width).max(0.0);
            let max_offset_y = (size.height - viewport.height).max(0.0);

            let clamped_offset = AbsoluteOffset {
                x: if max_offset_x > 0.0 {
                    proposed_offset.x.clamp(0.0, max_offset_x)
                } else {
                    0.0
                },
                y: if max_offset_y > 0.0 {
                    proposed_offset.y.clamp(0.0, max_offset_y)
                } else {
                    0.0
                },
            };

            self.viewport.offset = clamped_offset;

            let relative_x = if max_offset_x > 0.0 {
                clamped_offset.x / max_offset_x
            } else {
                0.0
            };

            let relative_y = if max_offset_y > 0.0 {
                clamped_offset.y / max_offset_y
            } else {
                0.0
            };

            // Snap scrollable to the new offset
            scrollable::snap_to(
                Id::new(VIEWER_SCROLLABLE_ID),
                RelativeOffset {
                    x: relative_x,
                    y: relative_y,
                },
            )
        } else {
            self.viewport.offset = proposed_offset;
            Task::none()
        }
    }

    fn handle_wheel_zoom(&mut self, delta: mouse::ScrollDelta) -> Task<Message> {
        // Zoom only when cursor is over the image
        if !self.viewer_state().is_cursor_over_image() {
            return Task::none();
        }

        let steps = scroll_steps(&delta);
        if steps.abs() < f32::EPSILON {
            return Task::none();
        }

        let new_zoom = self.zoom.zoom_percent + steps * self.zoom.zoom_step_percent;
        self.zoom.apply_manual_zoom(new_zoom);
        self.persist_preferences()
    }

    fn handle_raw_event(&mut self, event: event::Event) -> Task<Message> {
        match event {
            event::Event::Window(window_event) => {
                if let window::Event::Resized(size) = window_event {
                    let bounds = Rectangle::new(Point::new(0.0, 0.0), size);
                    self.viewport.update(bounds, self.viewport.offset);
                    self.refresh_fit_zoom();
                }
                Task::none()
            }
            event::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::WheelScrolled { delta } => {
                    // Mouse wheel is now used for zoom (not scroll)
                    // The Scrollable is wrapped in a wheel-blocking widget, so it won't receive this event
                    self.handle_wheel_zoom(delta)
                }
                mouse::Event::ButtonPressed(button) => {
                    if let Some(position) = self.cursor_position {
                        self.handle_mouse_button_pressed(button, position)
                    } else {
                        Task::none()
                    }
                }
                mouse::Event::ButtonReleased(button) => self.handle_mouse_button_released(button),
                mouse::Event::CursorMoved { position } => {
                    self.cursor_position = Some(position);
                    if self.drag.is_dragging {
                        self.handle_cursor_moved_during_drag(position)
                    } else {
                        Task::none()
                    }
                }
                mouse::Event::CursorLeft => {
                    self.cursor_position = None;
                    // Stop dragging if cursor leaves window
                    if self.drag.is_dragging {
                        self.drag.stop();
                    }
                    Task::none()
                }
                _ => Task::none(),
            },
            event::Event::Keyboard(keyboard_event) => {
                if let keyboard::Event::ModifiersChanged(modifiers) = keyboard_event {
                    self.modifiers = modifiers;
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let current_view: Element<'_, Message> = match self.mode {
            AppMode::Viewer => {
                let viewer_state = self.viewer_state();
                viewer::view(viewer::ViewContext {
                    i18n: &self.i18n,
                    error: self.error.as_ref().map(|error_state| viewer::ErrorContext {
                        friendly_text: &error_state.friendly_text,
                        details: &error_state.details,
                        show_details: error_state.show_details,
                    }),
                    image: self.image.as_ref().map(|image_data| viewer::ImageContext {
                        controls_context: viewer_controls::ViewContext { i18n: &self.i18n },
                        zoom: &self.zoom,
                        pane_context: viewer::pane::ViewContext {
                            background_theme: self.settings.background_theme(),
                            scroll_position: viewer_state.scroll_position_percentage(),
                            scrollable_id: VIEWER_SCROLLABLE_ID,
                        },
                        pane_model: viewer::pane::ViewModel {
                            image: image_data,
                            zoom_percent: self.zoom.zoom_percent,
                            padding: viewer_state.image_padding(),
                            is_dragging: self.drag.is_dragging,
                            cursor_over_image: viewer_state.is_cursor_over_image(),
                        },
                    }),
                })
            }
            AppMode::Settings => self
                .settings
                .view(SettingsViewContext { i18n: &self.i18n })
                .map(Message::Settings),
        };

        let switch_button = if self.mode == AppMode::Viewer {
            button(Text::new(self.i18n.tr("open-settings-button")))
                .on_press(Message::SwitchMode(AppMode::Settings))
        } else {
            button(Text::new(self.i18n.tr("back-to-viewer-button")))
                .on_press(Message::SwitchMode(AppMode::Viewer))
        };

        let final_layout = Container::new(
            iced::widget::column![
                Container::new(switch_button)
                    .width(Length::Shrink)
                    .padding(10)
                    .align_x(iced::alignment::Horizontal::Left),
                Container::new(current_view)
                    .width(Length::Fill)
                    .height(Length::Fill)
            ]
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill);
        final_layout.into()
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
#[allow(clippy::assertions_on_constants)]
mod tests {
    use super::*;
    use crate::config::DEFAULT_ZOOM_STEP_PERCENT;
    use crate::image_handler::ImageData;
    use crate::ui::state::zoom::{
        format_number, MAX_ZOOM_PERCENT, MIN_ZOOM_PERCENT, ZOOM_STEP_INVALID_KEY, ZOOM_STEP_RANGE_KEY,
    };
    use iced::widget::image::Handle;
    use std::fs;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;
    use unic_langid::LanguageIdentifier;

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

    #[test]
    fn new_starts_in_viewer_mode_without_image() {
        with_temp_config_dir(|_| {
            let (app, _command) = App::new(Flags {
                lang: None,
                file_path: None,
                i18n_dir: None,
            });
            assert_eq!(app.mode, AppMode::Viewer);
            assert!(app.image.is_none());
            assert!(app.error.is_none());
        });
    }

    #[test]
    fn update_image_loaded_ok_sets_state() {
        let mut app = App::default();
        let data = sample_image_data();

        let _ = app.update(Message::ImageLoaded(Ok(data.clone())));

        assert!(app.image.is_some());
        assert!(app.error.is_none());
        assert_eq!(app.image.as_ref().unwrap().width, data.width);
    }

    #[test]
    fn default_zoom_state_is_consistent() {
        let app = App::default();

        assert!(app.zoom.fit_to_window);
        assert_eq!(app.zoom.zoom_percent, DEFAULT_ZOOM_PERCENT);
        assert_eq!(app.zoom.zoom_input, format_number(DEFAULT_ZOOM_PERCENT));
        assert!(!app.zoom.zoom_input_dirty);
        assert!(app.zoom.zoom_input_error_key.is_none());

        assert_eq!(app.zoom.zoom_step_percent, DEFAULT_ZOOM_STEP_PERCENT);
        assert_eq!(
            app.settings.zoom_step_input_value(),
            format_number(DEFAULT_ZOOM_STEP_PERCENT)
        );
        assert!(!app.settings.zoom_step_input_dirty());
        assert!(app.settings.zoom_step_error_key().is_none());
        assert!(MIN_ZOOM_STEP_PERCENT <= app.zoom.zoom_step_percent);
        assert!(MAX_ZOOM_STEP_PERCENT >= app.zoom.zoom_step_percent);
        assert_eq!(app.zoom.manual_zoom_percent, DEFAULT_ZOOM_PERCENT);
        assert_eq!(app.modifiers, keyboard::Modifiers::default());

        assert_eq!(app.viewport.offset.x, 0.0);
        assert_eq!(app.viewport.offset.y, 0.0);
        assert_eq!(app.viewport.previous_offset.x, 0.0);
        assert_eq!(app.viewport.previous_offset.y, 0.0);

        assert!(MIN_ZOOM_PERCENT < DEFAULT_ZOOM_PERCENT);
        assert!(MAX_ZOOM_PERCENT > DEFAULT_ZOOM_PERCENT);
        assert_eq!(
            app.settings.background_theme(),
            config::BackgroundTheme::default()
        );
    }

    #[test]
    fn zoom_step_changes_commit_when_leaving_settings() {
        with_temp_config_dir(|_| {
            let mut app = App::default();
            app.mode = AppMode::Settings;
            let _ = app.update(Message::Settings(settings::Message::ZoomStepInputChanged(
                "25".into(),
            )));

            let _ = app.update(Message::SwitchMode(AppMode::Viewer));

            assert_eq!(app.mode, AppMode::Viewer);
            assert_eq!(app.zoom.zoom_step_percent, 25.0);
            assert_eq!(app.settings.zoom_step_input_value(), "25");
            assert!(!app.settings.zoom_step_input_dirty());
            assert!(app.settings.zoom_step_error_key().is_none());
        });
    }

    #[test]
    fn invalid_zoom_step_prevents_leaving_settings() {
        with_temp_config_dir(|_| {
            let mut app = App::default();
            app.mode = AppMode::Settings;
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
            assert_eq!(app.zoom.zoom_step_percent, DEFAULT_ZOOM_STEP_PERCENT);
        });
    }

    #[test]
    fn out_of_range_zoom_step_shows_error_and_stays_in_settings() {
        with_temp_config_dir(|_| {
            let mut app = App::default();
            app.mode = AppMode::Settings;
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
            assert_eq!(app.zoom.zoom_step_percent, DEFAULT_ZOOM_STEP_PERCENT);
        });
    }

    #[test]
    fn update_image_loaded_err_clears_image_and_sets_error() {
        let mut app = App::default();
        app.image = Some(sample_image_data());

        let _ = app.update(Message::ImageLoaded(Err(Error::Io("boom".into()))));

        assert!(app.image.is_none());
        assert!(app
            .error
            .as_ref()
            .map(|state| state.details.contains("boom"))
            .unwrap_or(false));
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
    fn submitting_valid_zoom_input_updates_zoom() {
        let mut app = App::default();
        app.zoom.zoom_input = "150".into();
        app.zoom.fit_to_window = true;

        let _ = app.update(Message::ViewerControls(
            viewer_controls::Message::ZoomInputSubmitted,
        ));

        assert_eq!(app.zoom.zoom_percent, 150.0);
        assert_eq!(app.zoom.manual_zoom_percent, 150.0);
        assert_eq!(app.zoom.zoom_input, format_number(150.0));
        assert!(!app.zoom.fit_to_window);
        assert!(app.zoom.zoom_input_error_key.is_none());
        assert!(!app.zoom.zoom_input_dirty);
    }

    #[test]
    fn submitting_out_of_range_zoom_clamps_value() {
        let mut app = App::default();
        app.zoom.zoom_input = "9999".into();

        let _ = app.update(Message::ViewerControls(
            viewer_controls::Message::ZoomInputSubmitted,
        ));

        assert_eq!(app.zoom.zoom_percent, MAX_ZOOM_PERCENT);
        assert_eq!(app.zoom.zoom_input, format_number(MAX_ZOOM_PERCENT));
        assert_eq!(app.zoom.manual_zoom_percent, MAX_ZOOM_PERCENT);
        assert!(!app.zoom.fit_to_window);
        assert!(app.zoom.zoom_input_error_key.is_none());
    }

    #[test]
    fn submitting_invalid_zoom_sets_error() {
        let mut app = App::default();
        app.zoom.fit_to_window = true;
        app.zoom.zoom_input = "oops".into();

        let _ = app.update(Message::ViewerControls(
            viewer_controls::Message::ZoomInputSubmitted,
        ));

        assert_eq!(app.zoom.zoom_percent, DEFAULT_ZOOM_PERCENT);
        assert!(app.zoom.fit_to_window);
        assert_eq!(app.zoom.zoom_input_error_key, Some(ZOOM_INPUT_INVALID_KEY));
        assert!(!app.zoom.zoom_input_dirty);
    }

    #[test]
    fn reset_zoom_restores_defaults() {
        let mut app = App::default();
        app.zoom.zoom_percent = 250.0;
        app.zoom.manual_zoom_percent = 250.0;
        app.zoom.fit_to_window = false;
        app.zoom.zoom_input = "250".into();

        let _ = app.update(Message::ViewerControls(viewer_controls::Message::ResetZoom));

        assert_eq!(app.zoom.zoom_percent, DEFAULT_ZOOM_PERCENT);
        assert_eq!(app.zoom.manual_zoom_percent, DEFAULT_ZOOM_PERCENT);
        assert_eq!(app.zoom.zoom_input, format_number(DEFAULT_ZOOM_PERCENT));
        assert!(!app.zoom.fit_to_window);
        assert!(app.zoom.zoom_input_error_key.is_none());
    }

    #[test]
    fn toggling_fit_to_window_updates_zoom() {
        let mut app = App::default();
        app.image = Some(build_image(2000, 1000));
        let bounds = Rectangle::new(Point::new(0.0, 0.0), iced::Size::new(1000.0, 500.0));
        let _ = app.update(Message::ViewportChanged {
            bounds,
            offset: AbsoluteOffset { x: 0.0, y: 0.0 },
        });

        app.zoom.fit_to_window = false;
        app.zoom.manual_zoom_percent = 160.0;

        let _ = app.update(Message::ViewerControls(
            viewer_controls::Message::SetFitToWindow(true),
        ));

        assert!(app.zoom.fit_to_window);
        let fit_zoom = app
            .viewer_state()
            .compute_fit_zoom_percent()
            .expect("fit zoom should exist");
        assert_eq!(app.zoom.zoom_percent, fit_zoom);
        assert!(fit_zoom <= DEFAULT_ZOOM_PERCENT);
        assert_eq!(app.zoom.zoom_input, format_number(fit_zoom));

        let _ = app.update(Message::ViewerControls(
            viewer_controls::Message::SetFitToWindow(false),
        ));

        assert!(!app.zoom.fit_to_window);
        assert_eq!(app.zoom.zoom_percent, fit_zoom);
        assert_eq!(app.zoom.manual_zoom_percent, fit_zoom);
        assert_eq!(app.zoom.zoom_input, format_number(fit_zoom));
    }

    #[test]
    fn viewport_change_updates_offset_tracking() {
        let mut app = App::default();
        let first = AbsoluteOffset { x: 10.0, y: 5.0 };
        let second = AbsoluteOffset { x: 4.0, y: 2.0 };
        let bounds = Rectangle::new(Point::new(32.0, 48.0), iced::Size::new(800.0, 600.0));

        let _ = app.update(Message::ViewportChanged {
            bounds,
            offset: first,
        });
        let _ = app.update(Message::ViewportChanged {
            bounds,
            offset: second,
        });

        assert_eq!(app.viewport.previous_offset, first);
        assert_eq!(app.viewport.offset, second);
        assert_eq!(app.viewport.bounds, Some(bounds));
    }

    #[test]
    fn wheel_scroll_applies_zoom_step_when_over_image() {
        let mut app = App::default();
        app.zoom.zoom_step_percent = 15.0;
        app.zoom.zoom_percent = 100.0;
        app.image = Some(build_image(800, 600));
        app.viewport.bounds = Some(Rectangle::new(
            Point::new(10.0, 10.0),
            iced::Size::new(400.0, 300.0),
        ));
        app.cursor_position = Some(Point::new(210.0, 160.0));

        // Simulate wheel scroll (no Ctrl needed anymore)
        let delta = mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 };
        let _ = app.handle_wheel_zoom(delta);

        assert_eq!(app.zoom.zoom_percent, 115.0);
        assert_eq!(app.zoom.manual_zoom_percent, 115.0);
        assert!(!app.zoom.fit_to_window);
    }

    #[test]
    fn wheel_scroll_ignored_when_cursor_not_over_image() {
        let mut app = App::default();
        app.zoom.zoom_step_percent = 20.0;
        app.zoom.zoom_percent = 150.0;
        app.zoom.manual_zoom_percent = 150.0;
        app.zoom.fit_to_window = false;
        app.image = Some(build_image(800, 600));
        app.viewport.bounds = Some(Rectangle::new(
            Point::new(0.0, 0.0),
            iced::Size::new(400.0, 300.0),
        ));
        app.cursor_position = Some(Point::new(1000.0, 1000.0));

        let delta = mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 };
        let _ = app.handle_wheel_zoom(delta);

        assert_eq!(app.zoom.zoom_percent, 150.0);
        assert_eq!(app.zoom.manual_zoom_percent, 150.0);
        assert!(!app.zoom.fit_to_window);
    }

    #[test]
    fn wheel_scroll_ignored_when_cursor_over_scrollbar_area() {
        let mut app = App::default();
        app.zoom.zoom_step_percent = 10.0;
        app.zoom.zoom_percent = 120.0;
        app.zoom.manual_zoom_percent = 120.0;
        app.zoom.fit_to_window = false;
        app.image = Some(build_image(1600, 2000));

        let viewport = Rectangle::new(Point::new(0.0, 0.0), iced::Size::new(400.0, 300.0));
        app.viewport.bounds = Some(viewport);
        app.cursor_position = Some(Point::new(
            viewport.x + viewport.width - 1.0,
            viewport.y + viewport.height / 2.0,
        ));

        let delta = mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 };
        let _ = app.handle_wheel_zoom(delta);

        assert_eq!(app.zoom.zoom_percent, 120.0);
        assert_eq!(app.zoom.manual_zoom_percent, 120.0);
        assert!(!app.zoom.fit_to_window);
    }

    #[test]
    fn zoom_step_submission_updates_config() {
        let mut app = App::default();
        let _ = app.update(Message::Settings(settings::Message::ZoomStepInputChanged(
            "5".into(),
        )));
        let _ = app.update(Message::Settings(settings::Message::ZoomStepSubmitted));

        assert_eq!(app.zoom.zoom_step_percent, 5.0);
        assert_eq!(app.settings.zoom_step_input_value(), "5");
        assert!(app.settings.zoom_step_error_key().is_none());
    }

    #[test]
    fn zoom_step_submission_rejects_invalid() {
        let mut app = App::default();
        let original = app.zoom.zoom_step_percent;
        let _ = app.update(Message::Settings(settings::Message::ZoomStepInputChanged(
            "0".into(),
        )));
        let _ = app.update(Message::Settings(settings::Message::ZoomStepSubmitted));

        assert_eq!(app.zoom.zoom_step_percent, original);
        assert_eq!(app.settings.zoom_step_input_value(), "0");
        assert_eq!(
            app.settings.zoom_step_error_key(),
            Some(ZOOM_STEP_RANGE_KEY)
        );

        let _ = app.update(Message::Settings(settings::Message::ZoomStepInputChanged(
            "abc".into(),
        )));
        let _ = app.update(Message::Settings(settings::Message::ZoomStepSubmitted));
        assert_eq!(
            app.settings.zoom_step_error_key(),
            Some(ZOOM_STEP_INVALID_KEY)
        );
    }

    #[test]
    fn switch_mode_changes_active_view() {
        let mut app = App::default();
        app.mode = AppMode::Viewer;

        let _ = app.update(Message::SwitchMode(AppMode::Settings));
        assert_eq!(app.mode, AppMode::Settings);
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
    fn view_renders_error_message_when_present() {
        let mut app = App::default();
        let error = Error::Io("failure".into());
        app.error = Some(ErrorState::from_error(&error, &app.i18n));
        let _ = app.view();
    }

    #[test]
    fn view_renders_image_when_available() {
        let mut app = App::default();
        app.image = Some(sample_image_data());
        let _ = app.view();
    }

    #[test]
    fn view_renders_settings_when_in_settings_mode() {
        let mut app = App::default();
        app.mode = AppMode::Settings;
        let _ = app.view();
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
    fn grab_and_drag_state_defaults_to_not_dragging() {
        let app = App::default();
        assert!(!app.drag.is_dragging);
        assert!(app.drag.start_position.is_none());
        assert!(app.drag.start_offset.is_none());
    }

    #[test]
    fn left_mouse_button_press_over_image_starts_drag() {
        let mut app = App::default();
        app.image = Some(build_image(800, 600));
        app.viewport.bounds = Some(Rectangle::new(
            Point::new(0.0, 0.0),
            iced::Size::new(400.0, 300.0),
        ));
        app.cursor_position = Some(Point::new(200.0, 150.0));
        app.viewport.offset = AbsoluteOffset { x: 50.0, y: 30.0 };

        let _ = app.update(Message::MouseButtonPressed {
            button: mouse::Button::Left,
            position: Point::new(200.0, 150.0),
        });

        assert!(app.drag.is_dragging);
        assert_eq!(app.drag.start_position, Some(Point::new(200.0, 150.0)));
        assert_eq!(
            app.drag.start_offset,
            Some(AbsoluteOffset { x: 50.0, y: 30.0 })
        );
    }

    #[test]
    fn left_mouse_button_press_outside_image_does_not_start_drag() {
        let mut app = App::default();
        app.image = Some(build_image(100, 100));
        app.viewport.bounds = Some(Rectangle::new(
            Point::new(0.0, 0.0),
            iced::Size::new(400.0, 300.0),
        ));
        app.cursor_position = Some(Point::new(500.0, 500.0));

        let _ = app.update(Message::MouseButtonPressed {
            button: mouse::Button::Left,
            position: Point::new(500.0, 500.0),
        });

        assert!(!app.drag.is_dragging);
        assert!(app.drag.start_position.is_none());
    }

    #[test]
    fn cursor_moved_while_dragging_updates_viewport_offset() {
        let mut app = App::default();
        app.image = Some(build_image(800, 600));
        app.viewport.bounds = Some(Rectangle::new(
            Point::new(0.0, 0.0),
            iced::Size::new(400.0, 300.0),
        ));
        app.drag.is_dragging = true;
        app.drag.start_position = Some(Point::new(200.0, 150.0));
        app.drag.start_offset = Some(AbsoluteOffset { x: 50.0, y: 30.0 });

        let _ = app.update(Message::CursorMovedDuringDrag {
            position: Point::new(180.0, 130.0),
        });

        // Cursor moved left/up: (200,150) -> (180,130), delta = (-20, -20)
        // In grab-and-drag, cursor moving left means dragging image right, so offset should increase
        // New offset: 50 - (-20) = 70, 30 - (-20) = 50
        assert_eq!(app.viewport.offset.x, 70.0);
        assert_eq!(app.viewport.offset.y, 50.0);
    }

    #[test]
    fn mouse_button_release_stops_dragging() {
        let mut app = App::default();
        app.drag.is_dragging = true;
        app.drag.start_position = Some(Point::new(200.0, 150.0));
        app.drag.start_offset = Some(AbsoluteOffset { x: 50.0, y: 30.0 });

        let _ = app.update(Message::MouseButtonReleased {
            button: mouse::Button::Left,
        });

        assert!(!app.drag.is_dragging);
        assert!(app.drag.start_position.is_none());
        assert!(app.drag.start_offset.is_none());
    }

    #[test]
    fn drag_offset_clamps_to_scroll_bounds() {
        let mut app = App::default();
        app.image = Some(build_image(2000, 2000));
        app.viewport.bounds = Some(Rectangle::new(
            Point::new(0.0, 0.0),
            iced::Size::new(400.0, 400.0),
        ));

        let max_offset = 1600.0; // image_height - viewport_height
        app.drag.start(
            Point::new(0.0, 0.0),
            AbsoluteOffset {
                x: 0.0,
                y: max_offset,
            },
        );

        // Cursor moves up, which would normally push the offset beyond the maximum
        let _ = app.handle_cursor_moved_during_drag(Point::new(0.0, -200.0));

        assert_eq!(app.viewport.offset.y, max_offset);
    }

    #[test]
    fn cursor_moved_while_not_dragging_does_not_update_offset() {
        let mut app = App::default();
        app.image = Some(build_image(800, 600));
        app.viewport.bounds = Some(Rectangle::new(
            Point::new(0.0, 0.0),
            iced::Size::new(400.0, 300.0),
        ));
        app.viewport.offset = AbsoluteOffset { x: 50.0, y: 30.0 };
        app.drag.is_dragging = false;

        let initial_offset = app.viewport.offset;

        let _ = app.update(Message::CursorMovedDuringDrag {
            position: Point::new(180.0, 130.0),
        });

        assert_eq!(app.viewport.offset, initial_offset);
    }
}
