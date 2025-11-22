use crate::config::{self, DEFAULT_ZOOM_STEP_PERCENT};
use crate::error::Error;
use crate::i18n::fluent::I18n;
use crate::image_handler::{self, ImageData};
use crate::ui::settings;
use crate::ui::viewer;
use iced::{
    alignment::{Horizontal, Vertical},
    event,
    keyboard,
    mouse,
    widget::{button, checkbox, Column, Container, Row, Scrollable, Space, Text},
    window, Element, Length, Size, Subscription, Task, Theme,
};
use iced::widget::text_input;
use iced::widget::scrollable::{AbsoluteOffset, Direction, Scrollbar, Viewport};
use std::fmt;

pub struct App {
    image: Option<ImageData>,
    error: Option<ErrorState>,
    pub i18n: I18n, // Made public
    mode: AppMode,
    fit_to_window: bool,
    zoom_percent: f32,
    zoom_input: String,
    zoom_input_dirty: bool,
    zoom_input_error_key: Option<&'static str>,
    zoom_step_percent: f32,
    zoom_step_input: String,
    zoom_step_input_dirty: bool,
    zoom_step_error_key: Option<&'static str>,
    manual_zoom_percent: f32,
    modifiers: keyboard::Modifiers,
    viewport_offset: AbsoluteOffset,
    previous_viewport_offset: AbsoluteOffset,
    viewport_bounds: Option<Size>,
    fit_zoom_percent: Option<f32>,
}

const MIN_ZOOM_PERCENT: f32 = 10.0;
const MAX_ZOOM_PERCENT: f32 = 800.0;
const DEFAULT_ZOOM_PERCENT: f32 = 100.0;
const MIN_ZOOM_STEP_PERCENT: f32 = 1.0;
const MAX_ZOOM_STEP_PERCENT: f32 = 200.0;

const ZOOM_INPUT_INVALID_KEY: &str = "viewer-zoom-input-error-invalid";
const ZOOM_STEP_INVALID_KEY: &str = "viewer-zoom-step-error-invalid";
const ZOOM_STEP_RANGE_KEY: &str = "viewer-zoom-step-error-range";

fn default_offset() -> AbsoluteOffset {
    AbsoluteOffset { x: 0.0, y: 0.0 }
}

fn format_number(value: f32) -> String {
    if (value.fract()).abs() < f32::EPSILON {
        format!("{value:.0}")
    } else {
        format!("{value:.2}").trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

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

fn clamp_zoom(value: f32) -> f32 {
    value.clamp(MIN_ZOOM_PERCENT, MAX_ZOOM_PERCENT)
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
            fit_to_window: true,
            zoom_percent: DEFAULT_ZOOM_PERCENT,
            zoom_input: format_number(DEFAULT_ZOOM_PERCENT),
            zoom_input_dirty: false,
            zoom_input_error_key: None,
            zoom_step_percent: DEFAULT_ZOOM_STEP_PERCENT,
            zoom_step_input: format_number(DEFAULT_ZOOM_STEP_PERCENT),
            zoom_step_input_dirty: false,
            zoom_step_error_key: None,
            manual_zoom_percent: DEFAULT_ZOOM_PERCENT,
            modifiers: keyboard::Modifiers::default(),
            viewport_offset: default_offset(),
            previous_viewport_offset: default_offset(),
            viewport_bounds: None,
            fit_zoom_percent: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ImageLoaded(Result<ImageData, Error>),
    SwitchMode(AppMode),
    LanguageSelected(unic_langid::LanguageIdentifier),
    ToggleErrorDetails,
    ZoomInputChanged(String),
    ZoomInputSubmitted,
    ResetZoom,
    ZoomIn,
    ZoomOut,
    SetFitToWindow(bool),
    ZoomStepInputChanged(String),
    ZoomStepSubmitted,
    ViewportChanged {
        bounds: Size,
        offset: AbsoluteOffset,
    },
    CtrlZoom {
        delta: mouse::ScrollDelta,
        control: bool,
    },
    RawEvent(event::Event),
}

#[derive(Debug, Default)]
pub struct Flags {
    pub lang: Option<String>,
    pub file_path: Option<String>,
}

pub fn run(flags: Flags) -> iced::Result {
    iced::application(|state: &App| state.title(), App::update, App::view)
        .theme(App::theme)
        .window(window::Settings {
            size: iced::Size::new(800.0, 600.0),
            ..window::Settings::default()
        })
        .subscription(App::subscription)
        .run_with(move || App::new(flags))
}

impl App {
    fn new(flags: Flags) -> (Self, Task<Message>) {
        let config = config::load().unwrap_or_default();
        let i18n = I18n::new(flags.lang, &config);

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
                app.enable_fit_to_window();
            } else {
                app.disable_fit_to_window();
            }
        }

        if let Some(step) = config.zoom_step {
            let clamped = clamp_zoom_step(step);
            app.zoom_step_percent = clamped;
            app.zoom_step_input = format_number(clamped);
        }

        if app.fit_to_window {
            app.refresh_fit_zoom();
        } else {
            app.disable_fit_to_window();
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
        event::listen().map(Message::RawEvent)
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
                self.mode = mode;
                Task::none()
            }
            Message::LanguageSelected(locale) => {
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
            Message::ToggleErrorDetails => {
                if let Some(error_state) = &mut self.error {
                    error_state.show_details = !error_state.show_details;
                }
                Task::none()
            }
            Message::ZoomInputChanged(value) => {
                self.zoom_input = value;
                self.zoom_input_dirty = true;
                self.zoom_input_error_key = None;
                Task::none()
            }
            Message::ZoomInputSubmitted => {
                self.zoom_input_dirty = false;

                if let Some(value) = parse_number(&self.zoom_input) {
                    self.apply_manual_zoom(value);
                    return self.persist_zoom_preferences();
                } else {
                    self.zoom_input_error_key = Some(ZOOM_INPUT_INVALID_KEY);
                }

                Task::none()
            }
            Message::ResetZoom => {
                self.apply_manual_zoom(DEFAULT_ZOOM_PERCENT);
                self.persist_zoom_preferences()
            }
            Message::ZoomIn => {
                self.apply_manual_zoom(self.zoom_percent + self.zoom_step_percent);
                self.persist_zoom_preferences()
            }
            Message::ZoomOut => {
                self.apply_manual_zoom(self.zoom_percent - self.zoom_step_percent);
                self.persist_zoom_preferences()
            }
            Message::SetFitToWindow(fit) => {
                if fit {
                    self.enable_fit_to_window();
                } else {
                    self.disable_fit_to_window();
                }
                self.persist_zoom_preferences()
            }
            Message::ZoomStepInputChanged(value) => {
                self.zoom_step_input = value;
                self.zoom_step_input_dirty = true;
                self.zoom_step_error_key = None;
                Task::none()
            }
            Message::ZoomStepSubmitted => {
                self.zoom_step_input_dirty = false;

                if let Some(value) = parse_number(&self.zoom_step_input) {
                    if value < MIN_ZOOM_STEP_PERCENT || value > MAX_ZOOM_STEP_PERCENT {
                        self.zoom_step_error_key = Some(ZOOM_STEP_RANGE_KEY);
                    } else {
                        let clamped = clamp_zoom_step(value);
                        self.zoom_step_percent = clamped;
                        self.zoom_step_input = format_number(clamped);
                        self.zoom_step_error_key = None;
                        return self.persist_zoom_preferences();
                    }
                } else {
                    self.zoom_step_error_key = Some(ZOOM_STEP_INVALID_KEY);
                }

                Task::none()
            }
            Message::ViewportChanged { bounds, offset } => {
                self.previous_viewport_offset = self.viewport_offset;
                self.viewport_offset = offset;
                self.viewport_bounds = Some(bounds);
                self.refresh_fit_zoom();
                Task::none()
            }
            Message::CtrlZoom { delta, control } => {
                self.handle_ctrl_zoom(delta, control)
            }
            Message::RawEvent(event) => self.handle_raw_event(event),
        }
    }

    fn update_zoom_display(&mut self, percent: f32) {
        self.zoom_percent = percent;
        self.zoom_input = format_number(percent);
    }

    fn apply_manual_zoom(&mut self, percent: f32) {
        let clamped = clamp_zoom(percent);
        self.manual_zoom_percent = clamped;
        self.update_zoom_display(clamped);
        self.zoom_input_dirty = false;
        self.zoom_input_error_key = None;
        self.fit_to_window = false;
    }

    fn enable_fit_to_window(&mut self) {
        self.fit_to_window = true;
        self.zoom_input_dirty = false;
        self.zoom_input_error_key = None;
        self.refresh_fit_zoom();
    }

    fn disable_fit_to_window(&mut self) {
        self.fit_to_window = false;
        let manual = clamp_zoom(self.manual_zoom_percent);
        self.manual_zoom_percent = manual;
        self.update_zoom_display(manual);
        self.zoom_input_dirty = false;
        self.zoom_input_error_key = None;
    }

    fn refresh_fit_zoom(&mut self) {
        self.fit_zoom_percent = self.compute_fit_zoom_percent();

        if self.fit_to_window {
            if let Some(fit_zoom) = self.fit_zoom_percent {
                self.update_zoom_display(fit_zoom);
                self.zoom_input_dirty = false;
                self.zoom_input_error_key = None;
            }
        }
    }

    fn compute_fit_zoom_percent(&self) -> Option<f32> {
        let image = self.image.as_ref()?;
        let bounds = self.viewport_bounds?;

        if image.width == 0 || image.height == 0 {
            return Some(DEFAULT_ZOOM_PERCENT);
        }

        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            return None;
        }

        let image_width = image.width as f32;
        let image_height = image.height as f32;

        let scale_x = bounds.width / image_width;
        let scale_y = bounds.height / image_height;

        let scale = scale_x.min(scale_y);

        if !scale.is_finite() || scale <= 0.0 {
            return Some(DEFAULT_ZOOM_PERCENT);
        }

        Some(clamp_zoom(scale * 100.0))
    }

    fn persist_zoom_preferences(&self) -> Task<Message> {
        if cfg!(test) {
            return Task::none();
        }

        let mut config = config::load().unwrap_or_default();
        config.fit_to_window = Some(self.fit_to_window);
        config.zoom_step = Some(self.zoom_step_percent);

        if let Err(error) = config::save(&config) {
            eprintln!("Failed to save config: {:?}", error);
        }

        Task::none()
    }

    pub(crate) fn zoom_step_input_value(&self) -> &str {
        &self.zoom_step_input
    }

    pub(crate) fn zoom_step_error_key(&self) -> Option<&'static str> {
        self.zoom_step_error_key
    }

    fn handle_ctrl_zoom(&mut self, delta: mouse::ScrollDelta, control: bool) -> Task<Message> {
        if !control {
            return Task::none();
        }

        let steps = scroll_steps(&delta);
        if steps.abs() < f32::EPSILON {
            return Task::none();
        }

        let new_zoom = self.zoom_percent + steps * self.zoom_step_percent;
        self.apply_manual_zoom(new_zoom);
        self.persist_zoom_preferences()
    }

    fn handle_raw_event(&mut self, event: event::Event) -> Task<Message> {
        match event {
            event::Event::Window(window_event) => {
                match window_event {
                    window::Event::Resized(size) => {
                        self.previous_viewport_offset = self.viewport_offset;
                        self.viewport_bounds = Some(size);
                        self.refresh_fit_zoom();
                    }
                    _ => {}
                }
                Task::none()
            }
            event::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::WheelScrolled { delta } => {
                    let control = self.modifiers.control();
                    self.handle_ctrl_zoom(delta, control)
                }
                _ => Task::none(),
            },
            event::Event::Keyboard(keyboard_event) => {
                match keyboard_event {
                    keyboard::Event::ModifiersChanged(modifiers) => {
                        self.modifiers = modifiers;
                    }
                    _ => {}
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let current_view: Element<'_, Message> = match self.mode {
            AppMode::Viewer => {
                if let Some(error_state) = &self.error {
                    let heading = Container::new(
                        Text::new(self.i18n.tr("error-load-image-heading")).size(24),
                    )
                    .width(Length::Fill)
                        .align_x(Horizontal::Center);

                    let summary = Container::new(
                        Text::new(error_state.friendly_text.clone()).width(Length::Fill),
                    )
                    .width(Length::Fill)
                        .align_x(Horizontal::Center);

                    let toggle_label = if error_state.show_details {
                        self.i18n.tr("error-details-hide")
                    } else {
                        self.i18n.tr("error-details-show")
                    };

                    let toggle_button = Container::new(
                        button(Text::new(toggle_label)).on_press(Message::ToggleErrorDetails),
                    )
                        .align_x(Horizontal::Center);

                    let mut error_content = Column::new()
                        .spacing(12)
                        .width(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center)
                        .push(heading)
                        .push(summary)
                        .push(toggle_button);

                    if error_state.show_details {
                        let details_heading = Container::new(
                            Text::new(self.i18n.tr("error-details-technical-heading")).size(16),
                        )
                        .width(Length::Fill)
                            .align_x(Horizontal::Center);

                        let details_body = Container::new(
                            Text::new(error_state.details.clone()).width(Length::Fill),
                        )
                        .width(Length::Fill)
                            .align_x(Horizontal::Left);

                        let details_column = Column::new()
                            .spacing(8)
                            .width(Length::Fill)
                            .push(details_heading)
                            .push(details_body);

                        error_content = error_content.push(
                            Container::new(details_column)
                                .width(Length::Fill)
                                .padding(16),
                        );
                    }

                    Container::new(error_content)
                        .width(Length::Fill)
                            .align_x(Horizontal::Center)
                            .align_y(Vertical::Center)
                        .into()
                } else if let Some(image_data) = &self.image {
                    let zoom_placeholder = self.i18n.tr("viewer-zoom-input-placeholder");
                    let zoom_label = Text::new(self.i18n.tr("viewer-zoom-label"));

                    let zoom_input = text_input(&zoom_placeholder, &self.zoom_input)
                        .on_input(Message::ZoomInputChanged)
                        .on_submit(Message::ZoomInputSubmitted)
                        .padding(6)
                        .size(16)
                        .width(Length::Fixed(90.0));

                    let zoom_out_button = button(Text::new(self.i18n.tr("viewer-zoom-out-button")))
                        .on_press(Message::ZoomOut)
                        .padding([6, 12]);

                    let reset_button = button(Text::new(self.i18n.tr("viewer-zoom-reset-button")))
                        .on_press(Message::ResetZoom)
                        .padding([6, 12]);

                    let zoom_in_button = button(Text::new(self.i18n.tr("viewer-zoom-in-button")))
                        .on_press(Message::ZoomIn)
                        .padding([6, 12]);

                    let fit_toggle = checkbox(self.i18n.tr("viewer-fit-to-window-toggle"), self.fit_to_window)
                        .on_toggle(Message::SetFitToWindow);

                    let mut zoom_controls_row = Row::new()
                        .spacing(10)
                        .align_y(Vertical::Center)
                        .push(zoom_label)
                        .push(zoom_input)
                        .push(Text::new("%"))
                        .push(zoom_out_button)
                        .push(reset_button)
                        .push(zoom_in_button)
                        .push(Space::new(Length::Fixed(16.0), Length::Shrink))
                        .push(fit_toggle);

                    if self.fit_to_window {
                        if let Some(fit_zoom) = self.fit_zoom_percent {
                            let label = self.i18n.tr("viewer-fit-percentage-label");
                            let value = format!("{}: {}%", label, format_number(fit_zoom));
                            zoom_controls_row = zoom_controls_row
                                .push(Space::new(Length::Fixed(12.0), Length::Shrink))
                                .push(Text::new(value));
                        }
                    }

                    let mut zoom_controls = Column::new().spacing(4).push(zoom_controls_row);

                    if let Some(error_key) = self.zoom_input_error_key {
                        let error_text = Text::new(self.i18n.tr(error_key)).size(14);
                        zoom_controls = zoom_controls.push(error_text);
                    }

                    let image_viewer = viewer::view_image(image_data, self.zoom_percent);

                    let scrollable = Scrollable::new(image_viewer)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .direction(Direction::Both {
                            vertical: Scrollbar::new(),
                            horizontal: Scrollbar::new(),
                        })
                        .on_scroll(|viewport: Viewport| {
                            let bounds = viewport.bounds();
                            Message::ViewportChanged {
                                bounds: Size::new(bounds.width, bounds.height),
                                offset: viewport.absolute_offset(),
                            }
                        });

                    let mut viewer_column = Column::new()
                        .spacing(16)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .push(zoom_controls);

                    viewer_column = viewer_column.push(
                        Container::new(scrollable)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_x(iced::alignment::Horizontal::Center)
                            .align_y(iced::alignment::Vertical::Center),
                    );

                    viewer_column.into()
                } else {
                    Text::new(self.i18n.tr("hello-message")).into()
                }
            }
            AppMode::Settings => settings::view_settings(self),
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
mod tests {
    use super::*;
    use crate::image_handler::ImageData;
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

        assert!(app.fit_to_window);
        assert_eq!(app.zoom_percent, DEFAULT_ZOOM_PERCENT);
        assert_eq!(app.zoom_input, format_number(DEFAULT_ZOOM_PERCENT));
        assert!(!app.zoom_input_dirty);
        assert!(app.zoom_input_error_key.is_none());

        assert_eq!(app.zoom_step_percent, DEFAULT_ZOOM_STEP_PERCENT);
        assert_eq!(app.zoom_step_input, format_number(DEFAULT_ZOOM_STEP_PERCENT));
        assert!(!app.zoom_step_input_dirty);
        assert!(app.zoom_step_error_key.is_none());
        assert!(MIN_ZOOM_STEP_PERCENT <= app.zoom_step_percent);
        assert!(MAX_ZOOM_STEP_PERCENT >= app.zoom_step_percent);
        assert_eq!(app.manual_zoom_percent, DEFAULT_ZOOM_PERCENT);
        assert_eq!(app.modifiers, keyboard::Modifiers::default());

        assert_eq!(app.viewport_offset.x, 0.0);
        assert_eq!(app.viewport_offset.y, 0.0);
        assert_eq!(app.previous_viewport_offset.x, 0.0);
        assert_eq!(app.previous_viewport_offset.y, 0.0);
        assert!(app.viewport_bounds.is_none());
        assert!(app.fit_zoom_percent.is_none());

        assert!(MIN_ZOOM_PERCENT < DEFAULT_ZOOM_PERCENT);
        assert!(MAX_ZOOM_PERCENT > DEFAULT_ZOOM_PERCENT);
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
        app.zoom_input = "150".into();
        app.fit_to_window = true;

        let _ = app.update(Message::ZoomInputSubmitted);

        assert_eq!(app.zoom_percent, 150.0);
        assert_eq!(app.manual_zoom_percent, 150.0);
        assert_eq!(app.zoom_input, format_number(150.0));
        assert!(!app.fit_to_window);
        assert!(app.zoom_input_error_key.is_none());
        assert!(!app.zoom_input_dirty);
    }

    #[test]
    fn submitting_out_of_range_zoom_clamps_value() {
        let mut app = App::default();
        app.zoom_input = "9999".into();

        let _ = app.update(Message::ZoomInputSubmitted);

        assert_eq!(app.zoom_percent, MAX_ZOOM_PERCENT);
        assert_eq!(app.zoom_input, format_number(MAX_ZOOM_PERCENT));
        assert_eq!(app.manual_zoom_percent, MAX_ZOOM_PERCENT);
        assert!(!app.fit_to_window);
        assert!(app.zoom_input_error_key.is_none());
    }

    #[test]
    fn submitting_invalid_zoom_sets_error() {
        let mut app = App::default();
        app.fit_to_window = true;
        app.zoom_input = "oops".into();

        let _ = app.update(Message::ZoomInputSubmitted);

        assert_eq!(app.zoom_percent, DEFAULT_ZOOM_PERCENT);
        assert!(app.fit_to_window);
        assert_eq!(app.zoom_input_error_key, Some(ZOOM_INPUT_INVALID_KEY));
        assert!(!app.zoom_input_dirty);
    }

    #[test]
    fn reset_zoom_restores_defaults() {
        let mut app = App::default();
        app.zoom_percent = 250.0;
        app.manual_zoom_percent = 250.0;
        app.fit_to_window = false;
        app.zoom_input = "250".into();

        let _ = app.update(Message::ResetZoom);

        assert_eq!(app.zoom_percent, DEFAULT_ZOOM_PERCENT);
        assert_eq!(app.manual_zoom_percent, DEFAULT_ZOOM_PERCENT);
        assert_eq!(app.zoom_input, format_number(DEFAULT_ZOOM_PERCENT));
        assert!(!app.fit_to_window);
        assert!(app.zoom_input_error_key.is_none());
    }

    #[test]
    fn toggling_fit_to_window_updates_zoom() {
        let mut app = App::default();
        app.image = Some(build_image(2000, 1000));
        let bounds = Size::new(1000.0, 500.0);
        let _ = app.update(Message::ViewportChanged {
            bounds,
            offset: AbsoluteOffset { x: 0.0, y: 0.0 },
        });

        app.fit_to_window = false;
        app.manual_zoom_percent = 160.0;

        let _ = app.update(Message::SetFitToWindow(true));

        assert!(app.fit_to_window);
        assert!(app.fit_zoom_percent.is_some());
        let fit_zoom = app.fit_zoom_percent.unwrap();
        assert_eq!(app.zoom_percent, fit_zoom);
        assert!(fit_zoom <= DEFAULT_ZOOM_PERCENT);
        assert_eq!(app.zoom_input, format_number(fit_zoom));

        let _ = app.update(Message::SetFitToWindow(false));

        assert!(!app.fit_to_window);
        assert_eq!(app.zoom_percent, app.manual_zoom_percent);
        assert_eq!(app.zoom_percent, 160.0);
    }

    #[test]
    fn viewport_change_updates_offset_tracking() {
        let mut app = App::default();
        let first = AbsoluteOffset { x: 10.0, y: 5.0 };
        let second = AbsoluteOffset { x: 4.0, y: 2.0 };
        let bounds = Size::new(800.0, 600.0);

        let _ = app.update(Message::ViewportChanged {
            bounds,
            offset: first,
        });
        let _ = app.update(Message::ViewportChanged {
            bounds,
            offset: second,
        });

        assert_eq!(app.previous_viewport_offset, first);
        assert_eq!(app.viewport_offset, second);
        assert_eq!(app.viewport_bounds, Some(bounds));
    }

    #[test]
    fn ctrl_scroll_applies_zoom_step_when_control_pressed() {
        let mut app = App::default();
        app.zoom_step_percent = 15.0;
        app.zoom_percent = 100.0;

        let _ = app.update(Message::CtrlZoom {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
            control: true,
        });

        assert_eq!(app.zoom_percent, 115.0);
        assert_eq!(app.manual_zoom_percent, 115.0);
        assert!(!app.fit_to_window);
    }

    #[test]
    fn ctrl_scroll_ignored_without_control() {
        let mut app = App::default();
        app.zoom_step_percent = 25.0;
        app.zoom_percent = 125.0;

        let _ = app.update(Message::CtrlZoom {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
            control: false,
        });

        assert_eq!(app.zoom_percent, 125.0);
        assert_eq!(app.manual_zoom_percent, DEFAULT_ZOOM_PERCENT);
        assert!(app.fit_to_window);
    }

    #[test]
    fn zoom_step_submission_updates_config() {
        let mut app = App::default();
        app.zoom_step_input = "5".into();

        let _ = app.update(Message::ZoomStepSubmitted);

        assert_eq!(app.zoom_step_percent, 5.0);
        assert_eq!(app.zoom_step_input, "5");
        assert!(app.zoom_step_error_key.is_none());
    }

    #[test]
    fn zoom_step_submission_rejects_invalid() {
        let mut app = App::default();
        app.zoom_step_input = "0".into();

        let _ = app.update(Message::ZoomStepSubmitted);

        assert_eq!(app.zoom_step_percent, DEFAULT_ZOOM_STEP_PERCENT);
        assert_eq!(app.zoom_step_error_key, Some(ZOOM_STEP_RANGE_KEY));

        app.zoom_step_input = "abc".into();
        let _ = app.update(Message::ZoomStepSubmitted);
        assert_eq!(app.zoom_step_error_key, Some(ZOOM_STEP_INVALID_KEY));
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

            let _ = app.update(Message::LanguageSelected(target_locale.clone()));

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
}
