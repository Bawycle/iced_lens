use iced::{Application, Settings};
use iced_lens::app::{App, Flags};
use pico_args;

fn main() -> iced::Result {
    let mut args = pico_args::Arguments::from_env();

    let flags = Flags {
        lang: args.opt_value_from_str("--lang").unwrap(),
        file_path: args
            .finish()
            .into_iter()
            .next()
            .and_then(|s| s.into_string().ok()),
    };

    let mut settings = Settings::with_flags(flags);
    settings.window.size = [800.0, 600.0].into();
    App::run(settings)
}
