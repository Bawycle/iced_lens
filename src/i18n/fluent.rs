#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

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