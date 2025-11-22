use crate::config;
use crate::error::Error;
use crate::i18n::fluent::I18n;
use crate::image_handler::{self, ImageData};
use crate::ui::settings;
use crate::ui::viewer;
use iced::{
    executor,
    widget::{Button, Container, Scrollable, Space, Text}, // Added Button, Container, Space
    Application, Command, Element, Length, Theme, window::{self, Id}, // Removed Alignment, Vertical
};
// ...existing code...
use iced_widget::scrollable::Direction;
use std::fmt;
use unic_langid::LanguageIdentifier;

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
    WindowMaximized, // New message
}

#[derive(Debug, Default)]
pub struct Flags {
    pub lang: Option<String>,
    pub file_path: Option<String>,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        let config = config::load().unwrap_or_default();
        let i18n = I18n::new(flags.lang, &config);

        let command = if let Some(path) = flags.file_path {
            Command::perform(
                async move { image_handler::load_image(&path) },
                Message::ImageLoaded,
            )
        } else {
            Command::none()
        };

        let app = App {
            i18n,
            mode: AppMode::Viewer,
            ..Self::default()
        };

        (app, command)
    }

    fn title(&self) -> String {
        self.i18n.tr("window-title")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ImageLoaded(Ok(image_data)) => {
                self.image = Some(image_data);
                self.error = None;
                Command::batch(vec![
                    window::maximize(Id::MAIN, true).map(|_: ()| Message::WindowMaximized)
                ])
            }
            Message::ImageLoaded(Err(e)) => {
                self.image = None;
                self.error = Some(e.to_string());
                Command::none()
            }
            Message::SwitchMode(mode) => {
                self.mode = mode;
                Command::none()
            }
            Message::LanguageSelected(locale) => {
                self.i18n.set_locale(locale.clone());

                let mut config = config::load().unwrap_or_default();
                config.language = Some(locale.to_string());
                if let Err(e) = config::save(&config) {
                    eprintln!("Failed to save config: {:?}", e);
                }
                Command::none()
            }
            Message::WindowMaximized => Command::none(), // New match arm
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let current_view: Element<'_, Message> = match self.mode {
            AppMode::Viewer => {
                if let Some(error_message) = &self.error {
                    Text::new(format!("Error: {}", error_message)).into()
                } else if let Some(image_data) = &self.image {
                    let image_viewer = viewer::view_image(image_data);
                    Scrollable::new(image_viewer)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .direction(Direction::Both {
                            vertical: Default::default(),
                            horizontal: Default::default(),
                        })
                        .into()
                } else {
                    Text::new(self.i18n.tr("hello-message")).into()
                }
            }
            AppMode::Settings => settings::view_settings(self),
        };

        let switch_button = if self.mode == AppMode::Viewer {
            Button::new(Text::new(self.i18n.tr("open-settings-button")))
                .on_press(Message::SwitchMode(AppMode::Settings))
        } else {
            Button::new(Text::new(self.i18n.tr("back-to-viewer-button")))
                .on_press(Message::SwitchMode(AppMode::Viewer))
        };

        Container::new(
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
            .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}