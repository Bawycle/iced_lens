//! This module provides internationalization (i18n) capabilities for the application
//! using the Fluent localization system. It dynamically discovers and loads `.ftl`
//! translation files from the `assets/i18n/` directory, resolves the active locale
//! based on CLI arguments, user configuration, and OS settings, and provides
//! methods to retrieve translated strings.
//!
//! # Examples
//!
//! ```no_run
//! use iced_lens::i18n::fluent::I18n;
//! use iced_lens::config::Config;
//! use unic_langid::LanguageIdentifier;
//!
//! // Initialize I18n
//! let config = Config { language: Some("fr".to_string()) };
//! let mut i18n = I18n::new(None, &config);
//!
//! // Get a translated string
//! let welcome_message = i18n.tr("window-title");
//!
//! // Change locale at runtime
//! i18n.set_locale("en-US".parse().unwrap());
//! let new_welcome_message = i18n.tr("window-title");
//!
//! assert_eq!(i18n.current_locale().to_string(), "en-US");
//! ```

use crate::config::Config;
use fluent_bundle::{FluentBundle, FluentResource};
use std::collections::HashMap;
use std::fs;
use unic_langid::LanguageIdentifier;

pub struct I18n {
    bundles: HashMap<LanguageIdentifier, FluentBundle<FluentResource>>,
    pub available_locales: Vec<LanguageIdentifier>,
    current_locale: LanguageIdentifier,
}

impl Default for I18n {
    fn default() -> Self {
        Self::new(None, &Config::default())
    }
}

const TRANSLATIONS_DIR: &str = "assets/i18n/";

impl I18n {
    pub fn new(cli_lang: Option<String>, config: &Config) -> Self {
        let mut bundles = HashMap::new();
        let mut available_locales = Vec::new();

        if let Ok(entries) = fs::read_dir(TRANSLATIONS_DIR) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                        if let Some(locale_str) = filename.strip_suffix(".ftl") {
                            if let Ok(locale) = locale_str.parse::<LanguageIdentifier>() {
                                if let Ok(content) = fs::read_to_string(&path) {
                                    let res =
                                        FluentResource::try_new(content).unwrap_or_else(|_| {
                                            panic!("Failed to parse FTL file: {}", filename)
                                        });
                                    let mut bundle = FluentBundle::new(vec![locale.clone()]);
                                    bundle.add_resource(res).expect("Failed to add resource.");
                                    bundles.insert(locale.clone(), bundle);
                                    available_locales.push(locale);
                                } else {
                                    eprintln!("Failed to read FTL file: {}", filename);
                                }
                            } else {
                                eprintln!("Invalid locale in FTL filename: {}", filename);
                            }
                        }
                    }
                }
            }
        } else {
            eprintln!(
                "Failed to read translations directory: {}",
                TRANSLATIONS_DIR
            );
        }

        available_locales.sort_by_key(|a| a.to_string());

        let default_locale: LanguageIdentifier = "en-US".parse().unwrap();
        let current_locale =
            resolve_locale(cli_lang, config, &available_locales).unwrap_or(default_locale);

        Self {
            bundles,
            available_locales,
            current_locale,
        }
    }

    pub fn set_locale(&mut self, locale: LanguageIdentifier) {
        if self.bundles.contains_key(&locale) {
            self.current_locale = locale;
        }
    }

    pub fn tr(&self, key: &str) -> String {
        if let Some(bundle) = self.bundles.get(&self.current_locale) {
            if let Some(msg) = bundle.get_message(key) {
                if let Some(pattern) = msg.value() {
                    let mut errors = vec![];
                    let value = bundle.format_pattern(pattern, None, &mut errors);
                    if errors.is_empty() {
                        return value.to_string();
                    }
                }
            }
        }
        format!("MISSING: {}", key)
    }

    pub fn current_locale(&self) -> &LanguageIdentifier {
        &self.current_locale
    }
}

fn resolve_locale(
    cli_lang: Option<String>,
    config: &Config,
    available: &[LanguageIdentifier],
) -> Option<LanguageIdentifier> {
    // 1. Check CLI args
    if let Some(lang_str) = cli_lang {
        if let Ok(lang) = lang_str.parse::<LanguageIdentifier>() {
            if available.contains(&lang) {
                return Some(lang);
            }
        }
    }

    // 2. Check config file
    if let Some(lang_str) = &config.language {
        if let Ok(lang) = lang_str.parse::<LanguageIdentifier>() {
            if available.contains(&lang) {
                return Some(lang);
            }
        }
    }

    // 3. Check OS locale
    if let Some(os_locale_str) = sys_locale::get_locale() {
        if let Ok(os_lang) = os_locale_str.parse::<LanguageIdentifier>() {
            if available.contains(&os_lang) {
                return Some(os_lang);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use unic_langid::LanguageIdentifier;

    #[test]
    fn test_resolve_locale_cli() {
        let config = Config::default();
        let available: Vec<LanguageIdentifier> =
            vec!["en-US".parse().unwrap(), "fr".parse().unwrap()];
        let lang = resolve_locale(Some("fr".to_string()), &config, &available);
        assert_eq!(lang, Some("fr".parse().unwrap()));
    }

    #[test]
    fn test_resolve_locale_config() {
        let mut config = Config::default();
        config.language = Some("fr".to_string());
        let available: Vec<LanguageIdentifier> =
            vec!["en-US".parse().unwrap(), "fr".parse().unwrap()];
        let lang = resolve_locale(None, &config, &available);
        assert_eq!(lang, Some("fr".parse().unwrap()));
    }

    #[test]
    fn test_resolve_locale_default() {
        let config = Config::default();
        let available: Vec<LanguageIdentifier> =
            vec!["en-US".parse().unwrap(), "fr".parse().unwrap()];
        let lang = resolve_locale(None, &config, &available);
        // This test is system dependent, so we just check it returns something or nothing
        // A more robust test would involve mocking the sys_locale::get_locale() call
        if let Some(l) = lang {
            assert!(available.contains(&l));
        }
    }

    #[test]
    fn test_tr_returns_missing_for_unknown_key() {
        let config = Config::default();
        let i18n = I18n::new(None, &config);
        let missing = i18n.tr("non-existent-key");
        assert!(missing.starts_with("MISSING:"));
    }

    #[test]
    fn test_set_locale_ignores_unknown_language() {
        let mut i18n = I18n::new(None, &Config::default());
        let original = i18n.current_locale().clone();
        let unknown_locale: LanguageIdentifier = "es-ES".parse().unwrap();
        i18n.set_locale(unknown_locale);
        assert_eq!(i18n.current_locale(), &original);
    }

    #[test]
    fn test_resolve_locale_returns_none_when_no_available_locales() {
        let config = Config::default();
        let available: Vec<LanguageIdentifier> = Vec::new();
        let lang = resolve_locale(Some("fr".to_string()), &config, &available);
        assert!(lang.is_none());
    }
}
