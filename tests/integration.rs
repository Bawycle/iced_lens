// SPDX-License-Identifier: MPL-2.0
use iced_lens::app::paths;
use iced_lens::app::persisted_state::AppState;
use iced_lens::config::{
    self, AiConfig, Config, DisplayConfig, FullscreenConfig, GeneralConfig, VideoConfig,
    DEFAULT_FRAME_CACHE_MB, DEFAULT_OVERLAY_TIMEOUT_SECS, DEFAULT_ZOOM_STEP_PERCENT,
};
use iced_lens::i18n::fluent::I18n;
use iced_lens::ui::theming::ThemeMode;
use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::tempdir;

// Mutex to prevent parallel tests from interfering with each other's env vars
static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
#[allow(clippy::assertions_on_constants)]
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
        general: GeneralConfig {
            language: Some("en-US".to_string()),
            theme_mode: ThemeMode::System,
        },
        display: DisplayConfig {
            fit_to_window: Some(true),
            zoom_step: Some(DEFAULT_ZOOM_STEP_PERCENT),
            background_theme: Some(config::BackgroundTheme::Dark),
            sort_order: Some(config::SortOrder::Alphabetical),
        },
        video: VideoConfig {
            autoplay: Some(false),
            volume: Some(config::DEFAULT_VOLUME),
            muted: Some(false),
            loop_enabled: Some(false),
            audio_normalization: Some(true),
            frame_cache_mb: Some(DEFAULT_FRAME_CACHE_MB),
            frame_history_mb: Some(config::DEFAULT_FRAME_HISTORY_MB),
            keyboard_seek_step_secs: Some(config::DEFAULT_KEYBOARD_SEEK_STEP_SECS),
        },
        fullscreen: FullscreenConfig {
            overlay_timeout_secs: Some(DEFAULT_OVERLAY_TIMEOUT_SECS),
        },
        ai: AiConfig::default(),
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
        general: GeneralConfig {
            language: Some("fr".to_string()),
            theme_mode: ThemeMode::System,
        },
        display: DisplayConfig {
            fit_to_window: Some(true),
            zoom_step: Some(DEFAULT_ZOOM_STEP_PERCENT),
            background_theme: Some(config::BackgroundTheme::Dark),
            sort_order: Some(config::SortOrder::Alphabetical),
        },
        video: VideoConfig {
            autoplay: Some(false),
            volume: Some(config::DEFAULT_VOLUME),
            muted: Some(false),
            loop_enabled: Some(false),
            audio_normalization: Some(true),
            frame_cache_mb: Some(DEFAULT_FRAME_CACHE_MB),
            frame_history_mb: Some(config::DEFAULT_FRAME_HISTORY_MB),
            keyboard_seek_step_secs: Some(config::DEFAULT_KEYBOARD_SEEK_STEP_SECS),
        },
        fullscreen: FullscreenConfig {
            overlay_timeout_secs: Some(DEFAULT_OVERLAY_TIMEOUT_SECS),
        },
        ai: AiConfig::default(),
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

// =============================================================================
// Path Injection Integration Tests
// =============================================================================

/// Tests that AppState and Config can use separate isolated directories.
#[test]
fn test_isolated_directories_for_state_and_config() {
    let state_dir = tempdir().expect("create state temp dir");
    let config_dir = tempdir().expect("create config temp dir");

    // Save app state to state_dir
    let state = AppState {
        last_save_directory: Some(PathBuf::from("/test/isolated/state")),
        last_open_directory: None,
        enable_deblur: false,
        enable_upscale: false,
    };
    let state_result = state.save_to(Some(state_dir.path().to_path_buf()));
    assert!(state_result.is_none(), "state save should succeed");

    // Save config to config_dir
    let mut config = Config::default();
    config.general.language = Some("ja".to_string());
    config::save_with_override(&config, Some(config_dir.path().to_path_buf()))
        .expect("config save should succeed");

    // Verify files are in separate locations
    assert!(state_dir.path().join("state.cbor").exists());
    assert!(config_dir.path().join("settings.toml").exists());

    // Verify data is independent
    let (loaded_state, _) = AppState::load_from(Some(state_dir.path().to_path_buf()));
    let (loaded_config, _) = config::load_with_override(Some(config_dir.path().to_path_buf()));

    assert_eq!(
        loaded_state.last_save_directory,
        Some(PathBuf::from("/test/isolated/state"))
    );
    assert_eq!(loaded_config.general.language, Some("ja".to_string()));
}

/// Tests that environment variables override default paths.
#[test]
fn test_env_var_overrides_paths() {
    let _lock = ENV_MUTEX.lock().unwrap();

    let data_temp = tempdir().expect("create data temp dir");
    let config_temp = tempdir().expect("create config temp dir");

    // Set environment variables
    std::env::set_var(paths::ENV_DATA_DIR, data_temp.path());
    std::env::set_var(paths::ENV_CONFIG_DIR, config_temp.path());

    // Verify paths module respects env vars
    let data_dir = paths::get_app_data_dir().expect("should get data dir");
    let config_dir = paths::get_app_config_dir().expect("should get config dir");

    assert_eq!(data_dir, data_temp.path());
    assert_eq!(config_dir, config_temp.path());

    // Clean up
    std::env::remove_var(paths::ENV_DATA_DIR);
    std::env::remove_var(paths::ENV_CONFIG_DIR);
}

/// Tests that multiple parallel test scenarios don't interfere with each other.
#[test]
fn test_parallel_test_isolation() {
    // Scenario A: German user with specific state
    let dir_a = tempdir().expect("create temp dir A");
    let base_a = dir_a.path().to_path_buf();

    let mut config_a = Config::default();
    config_a.general.language = Some("de".to_string());
    config_a.display.zoom_step = Some(25.0);
    config::save_with_override(&config_a, Some(base_a.clone())).expect("save A");

    let state_a = AppState {
        last_save_directory: Some(PathBuf::from("/user/a/downloads")),
        last_open_directory: None,
        enable_deblur: false,
        enable_upscale: false,
    };
    state_a.save_to(Some(base_a.clone()));

    // Scenario B: Spanish user with different state
    let dir_b = tempdir().expect("create temp dir B");
    let base_b = dir_b.path().to_path_buf();

    let mut config_b = Config::default();
    config_b.general.language = Some("es".to_string());
    config_b.display.zoom_step = Some(50.0);
    config::save_with_override(&config_b, Some(base_b.clone())).expect("save B");

    let state_b = AppState {
        last_save_directory: Some(PathBuf::from("/user/b/pictures")),
        last_open_directory: None,
        enable_deblur: true,
        enable_upscale: false,
    };
    state_b.save_to(Some(base_b.clone()));

    // Verify complete isolation
    let (loaded_config_a, _) = config::load_with_override(Some(base_a.clone()));
    let (loaded_config_b, _) = config::load_with_override(Some(base_b.clone()));
    let (loaded_state_a, _) = AppState::load_from(Some(base_a));
    let (loaded_state_b, _) = AppState::load_from(Some(base_b));

    assert_eq!(loaded_config_a.general.language, Some("de".to_string()));
    assert_eq!(loaded_config_a.display.zoom_step, Some(25.0));
    assert_eq!(loaded_config_b.general.language, Some("es".to_string()));
    assert_eq!(loaded_config_b.display.zoom_step, Some(50.0));

    assert_eq!(
        loaded_state_a.last_save_directory,
        Some(PathBuf::from("/user/a/downloads"))
    );
    assert_eq!(
        loaded_state_b.last_save_directory,
        Some(PathBuf::from("/user/b/pictures"))
    );
}

/// Tests that explicit override takes precedence over environment variable.
#[test]
fn test_explicit_override_takes_precedence_over_env_var() {
    let _lock = ENV_MUTEX.lock().unwrap();

    let env_dir = tempdir().expect("create env temp dir");
    let explicit_dir = tempdir().expect("create explicit temp dir");

    // Set environment variable
    std::env::set_var(paths::ENV_DATA_DIR, env_dir.path());

    // Save using explicit override (should ignore env var)
    let state = AppState {
        last_save_directory: Some(PathBuf::from("/explicit/path")),
        last_open_directory: None,
        enable_deblur: false,
        enable_upscale: false,
    };
    state.save_to(Some(explicit_dir.path().to_path_buf()));

    // Verify file is in explicit directory, not env directory
    assert!(explicit_dir.path().join("state.cbor").exists());
    assert!(!env_dir.path().join("state.cbor").exists());

    // Load using explicit override
    let (loaded, _) = AppState::load_from(Some(explicit_dir.path().to_path_buf()));
    assert_eq!(
        loaded.last_save_directory,
        Some(PathBuf::from("/explicit/path"))
    );

    // Clean up
    std::env::remove_var(paths::ENV_DATA_DIR);
}

/// Tests CI/CD-friendly behavior: tests using different temp dirs run independently.
#[test]
fn test_ci_friendly_isolated_tests() {
    // This test demonstrates that CI can run multiple test processes
    // without them interfering, as long as each uses its own temp directory

    let test_runs: Vec<_> = (0..3)
        .map(|i| {
            let dir = tempdir().expect("create temp dir");
            let base = dir.path().to_path_buf();

            let mut config = Config::default();
            config.general.language = Some(format!("lang-{}", i));
            config::save_with_override(&config, Some(base.clone())).expect("save");

            let state = AppState {
                last_save_directory: Some(PathBuf::from(format!("/run/{}/save", i))),
                last_open_directory: None,
                enable_deblur: false,
                enable_upscale: false,
            };
            state.save_to(Some(base.clone()));

            (dir, base, i)
        })
        .collect();

    // Verify each run is isolated
    for (dir, base, i) in test_runs {
        let (loaded_config, _) = config::load_with_override(Some(base.clone()));
        let (loaded_state, _) = AppState::load_from(Some(base));

        assert_eq!(loaded_config.general.language, Some(format!("lang-{}", i)));
        assert_eq!(
            loaded_state.last_save_directory,
            Some(PathBuf::from(format!("/run/{}/save", i)))
        );

        // Explicitly drop to clean up temp dir
        drop(dir);
    }
}
