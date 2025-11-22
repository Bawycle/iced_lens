use crate::config;
use crate::error::Error;
use crate::i18n::fluent::I18n;
use crate::image_handler::{self, ImageData};
use crate::ui::settings;
use crate::ui::viewer;
use iced::{
    widget::{button, Container, Scrollable, Text},
    window,
    Element, Length, Task, Theme,
};
use iced_widget::scrollable::Direction;
use std::fmt;

pub struct App {
    image: Option<ImageData>,
    error: Option<String>,
    pub i18n: I18n, // Made public
    mode: AppMode,
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
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ImageLoaded(Result<ImageData, Error>),
    SwitchMode(AppMode),
    LanguageSelected(unic_langid::LanguageIdentifier),
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

        let app = App {
            i18n,
            mode: AppMode::Viewer,
            ..Self::default()
        };

        (app, task)
    }

    fn title(&self) -> String {
        self.i18n.tr("window-title")
    }

    fn theme(&self) -> Theme {
        Theme::default()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ImageLoaded(Ok(image_data)) => {
                self.image = Some(image_data);
                self.error = None;
                Task::none()
            }
            Message::ImageLoaded(Err(e)) => {
                self.image = None;
                self.error = Some(e.to_string());
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

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let current_view: Element<'_, Message> = match self.mode {
            AppMode::Viewer => {
                if let Some(error_message) = &self.error {
                    Text::new(format!("Error: {}", error_message)).into()
                } else if let Some(image_data) = &self.image {
                    let image_viewer = viewer::view_image(image_data);

                    let scrollable = Scrollable::new(image_viewer)
                        .width(Length::Shrink)
                        .height(Length::Shrink)
                        .direction(Direction::Both {
                            vertical: Default::default(),
                            horizontal: Default::default(),
                        });

                    let centered = Container::new(scrollable)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center);
                    centered.into()
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
    fn update_image_loaded_err_clears_image_and_sets_error() {
        let mut app = App::default();
        app.image = Some(sample_image_data());

        let _ = app.update(Message::ImageLoaded(Err(Error::Io("boom".into()))));

        assert!(app.image.is_none());
        assert!(app.error.as_ref().unwrap().contains("boom"));
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
        app.error = Some("failure".into());
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
