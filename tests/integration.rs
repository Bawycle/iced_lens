// SPDX-License-Identifier: MPL-2.0
use iced_lens::config::{self, Config, DEFAULT_ZOOM_STEP_PERCENT};
use iced_lens::i18n::fluent::I18n;
use tempfile::tempdir;

#[test]
fn test_open_image_updates_state() {
    // This will require the full application state and image loading to be implemented.
    // For now, this is a placeholder.
    assert!(true);
}

#[test]
fn test_language_change_via_config() {
    // Create a temporary directory for the config file
    let dir = tempdir().expect("Failed to create temporary directory");
    let temp_config_file_path = dir.path().join("settings.toml");

    // 1. Initial config: en-US
    let initial_config = Config {
        language: Some("en-US".to_string()),
        fit_to_window: Some(true),
        zoom_step: Some(DEFAULT_ZOOM_STEP_PERCENT),
    };
    config::save_to_path(&initial_config, &temp_config_file_path)
        .expect("Failed to write initial config file");

    // Load i18n with initial config
    let loaded_initial_config = config::load_from_path(&temp_config_file_path)
        .expect("Failed to load initial config from path");
    let i18n_en = I18n::new(None, None, &loaded_initial_config);
    assert_eq!(i18n_en.current_locale().to_string(), "en-US");

    // 2. Change config to fr
    let french_config = Config {
        language: Some("fr".to_string()),
        fit_to_window: Some(true),
        zoom_step: Some(DEFAULT_ZOOM_STEP_PERCENT),
    };
    config::save_to_path(&french_config, &temp_config_file_path)
        .expect("Failed to write french config file");

    // Load i18n with french config
    let loaded_french_config = config::load_from_path(&temp_config_file_path)
        .expect("Failed to load french config from path");
    let i18n_fr = I18n::new(None, None, &loaded_french_config);
    assert_eq!(i18n_fr.current_locale().to_string(), "fr");

    // Clean up temporary directory
    dir.close().expect("Failed to close temporary directory");
}
