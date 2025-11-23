// SPDX-License-Identifier: MPL-2.0
use iced_lens::app::{self, Flags};

/// Application run mode derived from CLI arguments.
pub enum RunMode {
    Normal(Flags),
    Help(Option<String>, Option<String>), // (lang, i18n_dir)
}

fn parse_run_mode(mut args: pico_args::Arguments) -> Result<RunMode, pico_args::Error> {
    let lang = args.opt_value_from_str("--lang")?;
    let i18n_dir = args.opt_value_from_str("--i18n-dir")?;
    if args.contains("--help") || args.contains("-h") {
        return Ok(RunMode::Help(lang, i18n_dir));
    }
    let file_path = args
        .finish()
        .into_iter()
        .next()
        .and_then(|s| s.into_string().ok());
    Ok(RunMode::Normal(Flags {
        lang,
        file_path,
        i18n_dir,
    }))
}

fn main() -> iced::Result {
    let args = pico_args::Arguments::from_env();
    match parse_run_mode(args).expect("failed to parse CLI arguments") {
        RunMode::Help(lang, i18n_dir) => {
            let config = iced_lens::config::load().unwrap_or_default();
            let i18n = iced_lens::i18n::fluent::I18n::new(lang, i18n_dir, &config);
            println!("{}", help_text(&i18n));
            Ok(())
        }
        RunMode::Normal(flags) => app::run(flags),
    }
}
fn help_text(i18n: &iced_lens::i18n::fluent::I18n) -> String {
    format!(
        "{desc}\n\n{usage}\n  iced_lens [OPTIONS] [IMAGE_PATH]\n\n{opts}\n  {line_help}\n  {line_lang}\n  {line_i18n_dir}\n\n{args}\n  {arg_path}\n\n{examples}\n  {ex1}\n  {ex2}\n  {ex3}\n",
        desc = i18n.tr("help-description"),
        usage = i18n.tr("help-usage-heading"),
        opts = i18n.tr("help-options-heading"),
        line_help = i18n.tr("help-line-option-help"),
        line_lang = i18n.tr("help-line-option-lang"),
        line_i18n_dir = i18n.tr("help-line-option-i18n-dir"),
        args = i18n.tr("help-args-heading"),
        arg_path = i18n.tr("help-arg-image-path"),
        examples = i18n.tr("help-examples-heading"),
        ex1 = i18n.tr("help-example-1"),
        ex2 = i18n.tr("help-example-2"),
        ex3 = i18n.tr("help-example-3"),
    )
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
            OsString::from("--i18n-dir"),
            OsString::from("custom/langs"),
            OsString::from("image.png"),
        ];
        let mode = parse_run_mode(pico_args::Arguments::from_vec(args)).expect("parse should work");
        match mode {
            RunMode::Normal(flags) => {
                assert_eq!(flags.lang.as_deref(), Some("fr"));
                assert_eq!(flags.file_path.as_deref(), Some("image.png"));
                assert_eq!(flags.i18n_dir.as_deref(), Some("custom/langs"));
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
                assert!(flags.i18n_dir.is_none());
            }
            _ => panic!("expected Normal mode"),
        }
    }

    #[test]
    fn parse_run_mode_help_flag_triggers_help() {
        let args = vec![OsString::from("--help")];
        let mode = parse_run_mode(pico_args::Arguments::from_vec(args)).expect("parse should work");
        match mode {
            RunMode::Help(_, _) => {}
            _ => panic!("expected Help mode"),
        }
    }

    #[test]
    fn help_text_localized_french() {
        let args = vec![
            OsString::from("--help"),
            OsString::from("--lang"),
            OsString::from("fr"),
            OsString::from("--i18n-dir"),
            OsString::from("custom"),
        ];
        let mode = parse_run_mode(pico_args::Arguments::from_vec(args)).expect("parse should work");
        match mode {
            RunMode::Help(lang, dir) => {
                let config = iced_lens::config::load().unwrap_or_default();
                let i18n = iced_lens::i18n::fluent::I18n::new(lang, dir, &config);
                let text = help_text(&i18n);
                assert!(text.contains("UTILISATION"));
                assert!(text.contains("OPTIONS"));
            }
            _ => panic!("expected Help mode"),
        }
    }
}
