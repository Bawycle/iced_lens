use crate::config;
use crate::error::Error;
use crate::i18n::fluent::I18n;
use crate::image_handler::{self, ImageData};
use crate::ui::viewer;
use iced::{
    executor,
    widget::{Container, Scrollable, Text},
    Application, Command, Element, Length, Theme,
};
use std::fmt;

pub struct App {
    image: Option<ImageData>,
    error: Option<String>,
    i18n: I18n,
}

impl fmt::Debug for App {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("App")
            .field("image", &self.image)
            .field("error", &self.error)
            .finish()
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            image: None,
            error: None,
            i18n: I18n::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ImageLoaded(Result<ImageData, Error>),
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
            }
            Message::ImageLoaded(Err(e)) => {
                self.image = None;
                self.error = Some(e.to_string());
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let content: Element<'_, Message> = if let Some(error_message) = &self.error {
            Text::new(format!("Error: {}", error_message)).into()
        } else if let Some(image_data) = &self.image {
            let image_viewer = viewer::view_image(image_data);
            Scrollable::new(image_viewer).into()
        } else {
            Text::new("Hello, world!").into()
        };

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

