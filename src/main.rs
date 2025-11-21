use app::{App, Flags};
use iced::{Application, Settings};
use pico_args;

pub mod app;
pub mod config;
pub mod error;
pub mod i18n;
pub mod image_handler;
pub mod ui;

fn main() -> iced::Result {
    let mut args = pico_args::Arguments::from_env();

    let flags = Flags {
        lang: args.opt_value_from_str("--lang").unwrap(),
        file_path: args.finish().into_iter().next().and_then(|s| s.into_string().ok()),
    };

    let mut settings = Settings::with_flags(flags);
    settings.window.size = [800.0, 600.0].into();
    App::run(settings)
}
