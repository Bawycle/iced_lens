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
    cfg.fit_to_window = Some(ctx.viewer.image_fit_to_window());
    cfg.zoom_step = Some(ctx.viewer.zoom_step_percent());
    cfg.background_theme = Some(ctx.settings.background_theme());
    cfg.sort_order = Some(ctx.settings.sort_order());
    cfg.overlay_timeout_secs = Some(ctx.settings.overlay_timeout_secs());
    cfg.theme_mode = ctx.theme_mode;
    cfg.video_autoplay = Some(ctx.video_autoplay);
    cfg.audio_normalization = Some(ctx.audio_normalization);
    cfg.frame_cache_mb = Some(ctx.frame_cache_mb);
    cfg.frame_history_mb = Some(ctx.frame_history_mb);
    cfg.keyboard_seek_step_secs = Some(ctx.keyboard_seek_step_secs);

    // Video playback preferences (persisted but not in Settings UI)
    cfg.video_volume = Some(ctx.viewer.video_volume());
    cfg.video_muted = Some(ctx.viewer.video_muted());
    cfg.video_loop = Some(ctx.viewer.video_loop());

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

    cfg.language = Some(locale.to_string());

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
pub fn rescan_directory_if_same(
    viewer: &mut component::State,
    media_navigator: &mut MediaNavigator,
    saved_path: &Path,
) {
    let saved_dir = saved_path.parent();

    // Get the viewer's current directory
    let viewer_dir = viewer.current_image_path.as_ref().and_then(|p| p.parent());

    // Only rescan if both directories exist and match
    if let (Some(saved), Some(viewer_path)) = (saved_dir, viewer_dir) {
        if saved == viewer_path {
            // Rescan the media navigator (single source of truth)
            let (config, _) = config::load();
            let sort_order = config.sort_order.unwrap_or_default();
            if let Some(current_path) = viewer.current_image_path.clone() {
                let _ = media_navigator.scan_directory(&current_path, sort_order);
            }
        }
    }
}
