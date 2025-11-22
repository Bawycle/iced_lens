use iced::{Application, Settings};
use iced_lens::app::{App, Flags};

fn parse_flags(mut args: pico_args::Arguments) -> Result<Flags, pico_args::Error> {
    let lang = args.opt_value_from_str("--lang")?;
    let file_path = args
        .finish()
        .into_iter()
        .next()
        .and_then(|s| s.into_string().ok());

    Ok(Flags { lang, file_path })
}

fn main() -> iced::Result {
    let args = pico_args::Arguments::from_env();
    let flags = parse_flags(args).expect("failed to parse CLI arguments");

    let mut settings = Settings::with_flags(flags);
    settings.window.size = [800.0, 600.0].into();
    App::run(settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn parse_flags_accepts_language_and_file_path() {
        let args = vec![
            OsString::from("--lang"),
            OsString::from("fr"),
            OsString::from("image.png"),
        ];
        let flags = parse_flags(pico_args::Arguments::from_vec(args)).expect("parse should work");
        assert_eq!(flags.lang.as_deref(), Some("fr"));
        assert_eq!(flags.file_path.as_deref(), Some("image.png"));
    }

    #[test]
    fn parse_flags_handles_missing_optional_values() {
        let args: Vec<OsString> = Vec::new();
        let flags = parse_flags(pico_args::Arguments::from_vec(args)).expect("parse should work");
        assert!(flags.lang.is_none());
        assert!(flags.file_path.is_none());
    }
}
