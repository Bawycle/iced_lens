// SPDX-License-Identifier: MPL-2.0
//! Configuration persistence logic.
//!
//! This module handles saving and loading user preferences to disk,
//! including zoom settings, theme preferences, and language selection.

use super::Message;
use crate::config;
use crate::i18n::fluent::I18n;
use crate::media::MediaNavigator;
use crate::ui::notifications;
use crate::ui::settings::State as SettingsState;
use crate::ui::theming::ThemeMode;
use crate::ui::viewer::component;
use iced::Task;
use std::path::Path;
use unic_langid::LanguageIdentifier;

/// Context for persisting preferences, bundling all required state references.
pub struct PreferencesContext<'a> {
    pub viewer: &'a component::State,
    pub settings: &'a SettingsState,
    pub theme_mode: ThemeMode,
    pub video_autoplay: bool,
    pub audio_normalization: bool,
    pub frame_cache_mb: u32,
    pub frame_history_mb: u32,
    pub keyboard_seek_step_secs: f64,
    pub notifications: &'a mut notifications::Manager,
}

/// Persists the current viewer + settings preferences to disk.
///
/// Guarded during tests to keep isolation: unit tests exercise the logic by
/// calling the function directly rather than through `Effect`s.
pub fn persist_preferences(ctx: PreferencesContext<'_>) -> Task<Message> {
    if cfg!(test) {
        return Task::none();
    }

    let (mut cfg, load_warning) = config::load();
    if let Some(key) = load_warning {
        ctx.notifications
            .push(notifications::Notification::warning(&key));
    }

    // Use image_fit_to_window() to only persist the image setting, not video
    cfg.display.fit_to_window = Some(ctx.viewer.image_fit_to_window());
    cfg.display.zoom_step = Some(ctx.viewer.zoom_step_percent());
    cfg.display.background_theme = Some(ctx.settings.background_theme());
    cfg.display.sort_order = Some(ctx.settings.sort_order());
    cfg.display.max_skip_attempts = Some(ctx.settings.max_skip_attempts());
    cfg.fullscreen.overlay_timeout_secs = Some(ctx.settings.overlay_timeout_secs());
    cfg.general.theme_mode = ctx.theme_mode;
    cfg.video.autoplay = Some(ctx.video_autoplay);
    cfg.video.audio_normalization = Some(ctx.audio_normalization);
    cfg.video.frame_cache_mb = Some(ctx.frame_cache_mb);
    cfg.video.frame_history_mb = Some(ctx.frame_history_mb);
    cfg.video.keyboard_seek_step_secs = Some(ctx.keyboard_seek_step_secs);

    // Video playback preferences (persisted but not in Settings UI)
    cfg.video.volume = Some(ctx.viewer.video_volume());
    cfg.video.muted = Some(ctx.viewer.video_muted());
    cfg.video.loop_enabled = Some(ctx.viewer.video_loop());

    // AI preferences (note: enable flags are stored in AppState, not config)
    cfg.ai.deblur_model_url = Some(ctx.settings.deblur_model_url().to_string());
    cfg.ai.upscale_model_url = Some(ctx.settings.upscale_model_url().to_string());

    if config::save(&cfg).is_err() {
        ctx.notifications.push(notifications::Notification::warning(
            "notification-config-save-error",
        ));
    }

    Task::none()
}

/// Applies the newly selected locale, persists it to config, and refreshes
/// any visible error strings that depend on localization.
pub fn apply_language_change(
    i18n: &mut I18n,
    viewer: &mut component::State,
    locale: LanguageIdentifier,
    notifications: &mut notifications::Manager,
) -> Task<Message> {
    i18n.set_locale(locale.clone());

    let (mut cfg, load_warning) = config::load();
    if let Some(key) = load_warning {
        notifications.push(notifications::Notification::warning(&key));
    }

    cfg.general.language = Some(locale.to_string());

    if config::save(&cfg).is_err() {
        notifications.push(notifications::Notification::warning(
            "notification-config-save-error",
        ));
    }

    viewer.refresh_error_translation(i18n);
    Task::none()
}

/// Rescans the viewer's directory if the given path is in the same folder.
///
/// This is called after Save As to update the file list when a new image
/// is saved in the currently viewed directory. The current media remains
/// selected (no auto-switch to the new file).
pub fn rescan_directory_if_same(media_navigator: &mut MediaNavigator, saved_path: &Path) {
    let saved_dir = saved_path.parent();

    // Get the current directory from media_navigator (single source of truth)
    let current_dir = media_navigator
        .current_media_path()
        .and_then(|p| p.parent());

    // Only rescan if both directories exist and match
    if let (Some(saved), Some(current_path)) = (saved_dir, current_dir) {
        if saved == current_path {
            // Clone the path to avoid borrow conflict
            if let Some(path) = media_navigator
                .current_media_path()
                .map(|p| p.to_path_buf())
            {
                // Rescan the media navigator
                let (config, _) = config::load();
                let sort_order = config.display.sort_order.unwrap_or_default();
                let _ = media_navigator.scan_directory(&path, sort_order);
            }
        }
    }
}
