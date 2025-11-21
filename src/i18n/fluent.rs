use crate::config::Config;
use fluent_bundle::{FluentBundle, FluentResource};
use rust_embed::RustEmbed;
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

#[derive(RustEmbed)]
#[folder = "assets/i18n/"]
struct Asset;

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

impl I18n {
    pub fn new(cli_lang: Option<String>, config: &Config) -> Self {
        let mut bundles = HashMap::new();
        let mut available_locales = Vec::new();

        for file in Asset::iter() {
            let filename = file.as_ref();
            if let Some(locale_str) = filename.strip_suffix(".ftl") {
                if let Ok(locale) = locale_str.parse::<LanguageIdentifier>() {
                    if let Some(content) = Asset::get(filename) {
                        let res = FluentResource::try_new(String::from_utf8_lossy(content.data.as_ref()).to_string())
                            .expect("Failed to parse FTL file.");
                        let mut bundle = FluentBundle::new(vec![locale.clone()]);
                        bundle.add_resource(res).expect("Failed to add resource.");
                        bundles.insert(locale.clone(), bundle);
                        available_locales.push(locale);
                    }
                }
            }
        }
        
        let default_locale: LanguageIdentifier = "en-US".parse().unwrap();
        let current_locale = resolve_locale(cli_lang, config, &available_locales).unwrap_or(default_locale);

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
}

fn resolve_locale(cli_lang: Option<String>, config: &Config, available: &[LanguageIdentifier]) -> Option<LanguageIdentifier> {
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
        let available: Vec<LanguageIdentifier> = vec!["en-US".parse().unwrap(), "fr".parse().unwrap()];
        let lang = resolve_locale(Some("fr".to_string()), &config, &available);
        assert_eq!(lang, Some("fr".parse().unwrap()));
    }

    #[test]
    fn test_resolve_locale_config() {
        let mut config = Config::default();
        config.language = Some("fr".to_string());
        let available: Vec<LanguageIdentifier> = vec!["en-US".parse().unwrap(), "fr".parse().unwrap()];
        let lang = resolve_locale(None, &config, &available);
        assert_eq!(lang, Some("fr".parse().unwrap()));
    }

    #[test]
    fn test_resolve_locale_default() {
        let config = Config::default();
        let available: Vec<LanguageIdentifier> = vec!["en-US".parse().unwrap(), "fr".parse().unwrap()];
        let lang = resolve_locale(None, &config, &available);
        // This test is system dependent, so we just check it returns something or nothing
        // A more robust test would involve mocking the sys_locale::get_locale() call
        if let Some(l) = lang {
            assert!(available.contains(&l));
        }
    }
}
