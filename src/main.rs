// SPDX-License-Identifier: MPL-2.0
use iced_lens::app::{self, Flags};

/// Application run mode derived from CLI arguments.
pub enum RunMode {
    /// Normal execution with parsed flags.
    Normal(Flags),
    /// Help requested; print usage and exit.
    Help,
}

fn parse_run_mode(mut args: pico_args::Arguments) -> Result<RunMode, pico_args::Error> {
    if args.contains("--help") || args.contains("-h") {
        return Ok(RunMode::Help);
    }
    let lang = args.opt_value_from_str("--lang")?;
    let file_path = args
        .finish()
        .into_iter()
        .next()
        .and_then(|s| s.into_string().ok());
    Ok(RunMode::Normal(Flags { lang, file_path }))
}

fn main() -> iced::Result {
    let args = pico_args::Arguments::from_env();
    match parse_run_mode(args).expect("failed to parse CLI arguments") {
        RunMode::Help => {
            println!("{}", help_text());
            Ok(())
        }
        RunMode::Normal(flags) => app::run(flags),
    }
}

fn help_text() -> &'static str {
    // Keep simple, human-readable. Extend with localized help if needed.
    "IcedLens – Image Viewer\n\nUSAGE:\n  iced_lens [OPTIONS] [IMAGE_PATH]\n\nOPTIONS:\n  -h, --help        Show this help text\n      --lang <id>   Set locale (e.g. en-US, fr)\n\nARGS:\n  <IMAGE_PATH>     Path to an image file to open\n\nEXAMPLES:\n  iced_lens ./photo.png\n  iced_lens --lang fr ./image.jpg\n  iced_lens --help\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn parse_run_mode_accepts_language_and_file_path() {
        let args = vec![
            OsString::from("--lang"),
            OsString::from("fr"),
            OsString::from("image.png"),
        ];
        let mode = parse_run_mode(pico_args::Arguments::from_vec(args)).expect("parse should work");
        match mode {
            RunMode::Normal(flags) => {
                assert_eq!(flags.lang.as_deref(), Some("fr"));
                assert_eq!(flags.file_path.as_deref(), Some("image.png"));
            }
            _ => panic!("expected Normal mode"),
        }
    }

    #[test]
    fn parse_run_mode_handles_missing_optional_values() {
        let args: Vec<OsString> = Vec::new();
        let mode = parse_run_mode(pico_args::Arguments::from_vec(args)).expect("parse should work");
        match mode {
            RunMode::Normal(flags) => {
                assert!(flags.lang.is_none());
                assert!(flags.file_path.is_none());
            }
            _ => panic!("expected Normal mode"),
        }
    }

    #[test]
    fn parse_run_mode_help_flag_triggers_help() {
        let args = vec![OsString::from("--help")];
        let mode = parse_run_mode(pico_args::Arguments::from_vec(args)).expect("parse should work");
        matches!(mode, RunMode::Help);
    }

    #[test]
    fn help_text_contains_usage_header() {
        assert!(help_text().contains("USAGE:"));
        assert!(help_text().contains("OPTIONS:"));
    }
}
