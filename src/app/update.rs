// SPDX-License-Identifier: MPL-2.0
//! Update logic and message handlers for the application.
//!
//! This module contains the main `update` function and all specialized
//! message handlers for different parts of the application.

use super::{persistence, Message, Screen};
use crate::config;
use crate::i18n::fluent::I18n;
use crate::media::{self, frame_export::ExportableFrame, MediaData, MediaNavigator};
use crate::ui::about::{self, Event as AboutEvent};
use crate::ui::help::{self, Event as HelpEvent};
use crate::ui::image_editor::{self, Event as ImageEditorEvent, State as ImageEditorState};
use crate::ui::navbar::{self, Event as NavbarEvent};
use crate::ui::settings::{self, Event as SettingsEvent, State as SettingsState};
use crate::ui::theming::ThemeMode;
use crate::ui::viewer::component;
use iced::{window, Task};
use std::path::PathBuf;

/// Context for update operations containing mutable references to app state.
pub struct UpdateContext<'a> {
    pub i18n: &'a mut I18n,
    pub screen: &'a mut Screen,
    pub settings: &'a mut SettingsState,
    pub viewer: &'a mut component::State,
    pub image_editor: &'a mut Option<ImageEditorState>,
    pub media_navigator: &'a mut MediaNavigator,
    pub fullscreen: &'a mut bool,
    pub window_id: &'a mut Option<window::Id>,
    pub theme_mode: &'a mut ThemeMode,
    pub video_autoplay: &'a mut bool,
    pub audio_normalization: &'a mut bool,
    pub frame_cache_mb: u32,
    pub frame_history_mb: u32,
    pub menu_open: &'a mut bool,
    pub help_state: &'a mut help::State,
    pub app_state: &'a mut super::persisted_state::AppState,
}

/// Handles viewer component messages.
pub fn handle_viewer_message(
    ctx: &mut UpdateContext<'_>,
    message: component::Message,
) -> Task<Message> {
    if let component::Message::RawEvent { window, .. } = &message {
        *ctx.window_id = Some(*window);
    }

    let (effect, task) = ctx.viewer.handle_message(message, ctx.i18n);
    let viewer_task = task.map(Message::Viewer);
    let side_effect = match effect {
        component::Effect::PersistPreferences => persistence::persist_preferences(
            ctx.viewer,
            ctx.settings,
            *ctx.theme_mode,
            *ctx.video_autoplay,
            *ctx.audio_normalization,
            ctx.frame_cache_mb,
            ctx.frame_history_mb,
            ctx.settings.keyboard_seek_step_secs(),
        ),
        component::Effect::ToggleFullscreen => toggle_fullscreen(ctx.fullscreen, ctx.window_id),
        component::Effect::ExitFullscreen => {
            update_fullscreen_mode(ctx.fullscreen, ctx.window_id, false)
        }
        component::Effect::OpenSettings => {
            *ctx.screen = Screen::Settings;
            Task::none()
        }
        component::Effect::EnterEditor => handle_screen_switch(ctx, Screen::ImageEditor),
        component::Effect::NavigateNext => handle_navigate_next(ctx),
        component::Effect::NavigatePrevious => handle_navigate_previous(ctx),
        component::Effect::CaptureFrame {
            frame,
            video_path,
            position_secs,
        } => handle_capture_frame(frame, video_path, position_secs),
        component::Effect::MediaDeleted { next_path } => {
            // Sync media_navigator with the new current path after deletion
            if let Some(path) = next_path {
                let config = config::load().unwrap_or_default();
                let sort_order = config.sort_order.unwrap_or_default();
                let _ = ctx.media_navigator.scan_directory(&path, sort_order);
            }
            Task::none()
        }
        component::Effect::None => Task::none(),
    };
    Task::batch([viewer_task, side_effect])
}

/// Handles screen transitions.
pub fn handle_screen_switch(ctx: &mut UpdateContext<'_>, target: Screen) -> Task<Message> {
    // Handle Settings → Viewer transition
    if matches!(target, Screen::Viewer) && matches!(ctx.screen, Screen::Settings) {
        match ctx.settings.ensure_zoom_step_committed() {
            Ok(Some(value)) => {
                ctx.viewer.set_zoom_step_percent(value);
                *ctx.screen = target;
                return persistence::persist_preferences(
                    ctx.viewer,
                    ctx.settings,
                    *ctx.theme_mode,
                    *ctx.video_autoplay,
                    *ctx.audio_normalization,
                    ctx.frame_cache_mb,
                    ctx.frame_history_mb,
                    ctx.settings.keyboard_seek_step_secs(),
                );
            }
            Ok(None) => {
                *ctx.screen = target;
                return Task::none();
            }
            Err(_) => {
                *ctx.screen = Screen::Settings;
                return Task::none();
            }
        }
    }

    // Handle Viewer → Editor transition
    if matches!(target, Screen::ImageEditor) && matches!(ctx.screen, Screen::Viewer) {
        if let (Some(image_path), Some(media_data)) = (
            ctx.viewer.current_image_path.clone(),
            ctx.viewer.media().cloned(),
        ) {
            // Editor only supports images in v0.2, not videos
            let image_data = match media_data {
                MediaData::Image(img) => img,
                MediaData::Video(_) => {
                    eprintln!("Video editing is not supported in this version");
                    return Task::none();
                }
            };

            // Synchronize media_navigator with viewer state before entering editor
            let config = config::load().unwrap_or_default();
            let sort_order = config.sort_order.unwrap_or_default();
            if let Err(err) = ctx.media_navigator.scan_directory(&image_path, sort_order) {
                eprintln!("Failed to scan directory: {:?}", err);
            }

            match ImageEditorState::new(image_path, image_data) {
                Ok(state) => {
                    *ctx.image_editor = Some(state);
                    *ctx.screen = target;
                }
                Err(err) => {
                    eprintln!("Failed to enter editor screen: {err:?}");
                }
            }
            return Task::none();
        } else {
            // Can't enter editor screen without an image
            return Task::none();
        }
    }

    // Handle Editor → Viewer transition
    if matches!(target, Screen::Viewer) && matches!(ctx.screen, Screen::ImageEditor) {
        *ctx.image_editor = None;
        *ctx.screen = target;
        return Task::none();
    }

    *ctx.screen = target;
    Task::none()
}

/// Handles settings component messages.
pub fn handle_settings_message(
    ctx: &mut UpdateContext<'_>,
    message: settings::Message,
) -> Task<Message> {
    match ctx.settings.update(message) {
        SettingsEvent::None => Task::none(),
        SettingsEvent::BackToViewer => {
            *ctx.screen = Screen::Viewer;
            Task::none()
        }
        SettingsEvent::BackToViewerWithZoomChange(value) => {
            ctx.viewer.set_zoom_step_percent(value);
            *ctx.screen = Screen::Viewer;
            persistence::persist_preferences(
                ctx.viewer,
                ctx.settings,
                *ctx.theme_mode,
                *ctx.video_autoplay,
                *ctx.audio_normalization,
                ctx.frame_cache_mb,
                ctx.frame_history_mb,
                ctx.settings.keyboard_seek_step_secs(),
            )
        }
        SettingsEvent::LanguageSelected(locale) => {
            persistence::apply_language_change(ctx.i18n, ctx.viewer, locale)
        }
        SettingsEvent::ZoomStepChanged(value) => {
            ctx.viewer.set_zoom_step_percent(value);
            persistence::persist_preferences(
                ctx.viewer,
                ctx.settings,
                *ctx.theme_mode,
                *ctx.video_autoplay,
                *ctx.audio_normalization,
                ctx.frame_cache_mb,
                ctx.frame_history_mb,
                ctx.settings.keyboard_seek_step_secs(),
            )
        }
        SettingsEvent::BackgroundThemeSelected(_) => persistence::persist_preferences(
            ctx.viewer,
            ctx.settings,
            *ctx.theme_mode,
            *ctx.video_autoplay,
            *ctx.audio_normalization,
            ctx.frame_cache_mb,
            ctx.frame_history_mb,
            ctx.settings.keyboard_seek_step_secs(),
        ),
        SettingsEvent::ThemeModeSelected(mode) => {
            *ctx.theme_mode = mode;
            persistence::persist_preferences(
                ctx.viewer,
                ctx.settings,
                *ctx.theme_mode,
                *ctx.video_autoplay,
                *ctx.audio_normalization,
                ctx.frame_cache_mb,
                ctx.frame_history_mb,
                ctx.settings.keyboard_seek_step_secs(),
            )
        }
        SettingsEvent::SortOrderSelected(_) => persistence::persist_preferences(
            ctx.viewer,
            ctx.settings,
            *ctx.theme_mode,
            *ctx.video_autoplay,
            *ctx.audio_normalization,
            ctx.frame_cache_mb,
            ctx.frame_history_mb,
            ctx.settings.keyboard_seek_step_secs(),
        ),
        SettingsEvent::OverlayTimeoutChanged(_) => persistence::persist_preferences(
            ctx.viewer,
            ctx.settings,
            *ctx.theme_mode,
            *ctx.video_autoplay,
            *ctx.audio_normalization,
            ctx.frame_cache_mb,
            ctx.frame_history_mb,
            ctx.settings.keyboard_seek_step_secs(),
        ),
        SettingsEvent::VideoAutoplayChanged(enabled) => {
            *ctx.video_autoplay = enabled;
            ctx.viewer.set_video_autoplay(enabled);
            persistence::persist_preferences(
                ctx.viewer,
                ctx.settings,
                *ctx.theme_mode,
                *ctx.video_autoplay,
                *ctx.audio_normalization,
                ctx.frame_cache_mb,
                ctx.frame_history_mb,
                ctx.settings.keyboard_seek_step_secs(),
            )
        }
        SettingsEvent::AudioNormalizationChanged(enabled) => {
            *ctx.audio_normalization = enabled;
            persistence::persist_preferences(
                ctx.viewer,
                ctx.settings,
                *ctx.theme_mode,
                *ctx.video_autoplay,
                *ctx.audio_normalization,
                ctx.frame_cache_mb,
                ctx.frame_history_mb,
                ctx.settings.keyboard_seek_step_secs(),
            )
        }
        SettingsEvent::FrameCacheMbChanged(_) => persistence::persist_preferences(
            ctx.viewer,
            ctx.settings,
            *ctx.theme_mode,
            *ctx.video_autoplay,
            *ctx.audio_normalization,
            ctx.frame_cache_mb,
            ctx.frame_history_mb,
            ctx.settings.keyboard_seek_step_secs(),
        ),
        SettingsEvent::FrameHistoryMbChanged(_) => persistence::persist_preferences(
            ctx.viewer,
            ctx.settings,
            *ctx.theme_mode,
            *ctx.video_autoplay,
            *ctx.audio_normalization,
            ctx.frame_cache_mb,
            ctx.frame_history_mb,
            ctx.settings.keyboard_seek_step_secs(),
        ),
        SettingsEvent::KeyboardSeekStepChanged(step) => {
            ctx.viewer.set_keyboard_seek_step_secs(step);
            persistence::persist_preferences(
                ctx.viewer,
                ctx.settings,
                *ctx.theme_mode,
                *ctx.video_autoplay,
                *ctx.audio_normalization,
                ctx.frame_cache_mb,
                ctx.frame_history_mb,
                ctx.settings.keyboard_seek_step_secs(),
            )
        }
    }
}

/// Handles image editor component messages.
pub fn handle_editor_message(
    ctx: &mut UpdateContext<'_>,
    message: image_editor::Message,
) -> Task<Message> {
    let Some(editor_state) = ctx.image_editor.as_mut() else {
        return Task::none();
    };

    match editor_state.update(message) {
        ImageEditorEvent::None => Task::none(),
        ImageEditorEvent::ExitEditor => {
            // Get the image source before dropping the editor
            let image_source = editor_state.image_source().clone();

            *ctx.image_editor = None;
            *ctx.screen = Screen::Viewer;

            // For file mode: reload the image in the viewer to show any saved changes
            // For captured frame mode: just return to viewer without reloading
            match image_source {
                image_editor::ImageSource::File(current_image_path) => {
                    // Set loading state directly (before render)
                    ctx.viewer.is_loading_media = true;
                    ctx.viewer.loading_started_at = Some(std::time::Instant::now());

                    // Reload the image in the viewer to show any saved changes
                    Task::perform(
                        async move { media::load_media(&current_image_path) },
                        |result| Message::Viewer(component::Message::ImageLoaded(result)),
                    )
                }
                image_editor::ImageSource::CapturedFrame { .. } => {
                    // Just return to viewer, no need to reload anything
                    Task::none()
                }
            }
        }
        ImageEditorEvent::NavigateNext => handle_editor_navigate_next(ctx),
        ImageEditorEvent::NavigatePrevious => handle_editor_navigate_previous(ctx),
        ImageEditorEvent::SaveRequested { path, overwrite: _ } => {
            // Save the edited image
            if let Some(editor) = ctx.image_editor.as_mut() {
                match editor.save_image(&path) {
                    Ok(()) => {
                        eprintln!("Image saved successfully to: {:?}", path);
                    }
                    Err(err) => {
                        eprintln!("Failed to save image: {:?}", err);
                    }
                }
            }
            Task::none()
        }
        ImageEditorEvent::SaveAsRequested => {
            let editor_state = ctx.image_editor.as_ref().expect("editor state exists");
            let last_dir = ctx.app_state.last_save_directory.clone();
            handle_save_as_dialog(editor_state, last_dir)
        }
    }
}

/// Handles Save As dialog request.
fn handle_save_as_dialog(
    editor_state: &ImageEditorState,
    last_save_directory: Option<PathBuf>,
) -> Task<Message> {
    use crate::media::frame_export::{generate_default_filename, ExportFormat};

    let image_source = editor_state.image_source().clone();
    let export_format = editor_state.export_format();

    // Get filter based on selected export format
    let (filter_name, filter_ext): (&str, Vec<&str>) = match export_format {
        ExportFormat::Png => ("PNG Image", vec!["png"]),
        ExportFormat::Jpeg => ("JPEG Image", vec!["jpg", "jpeg"]),
        ExportFormat::WebP => ("WebP Image", vec!["webp"]),
    };

    // Generate filename based on image source, with selected format extension
    let filename = match &image_source {
        image_editor::ImageSource::File(path) => {
            // Replace extension with selected format
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
            format!("{}.{}", stem, export_format.extension())
        }
        image_editor::ImageSource::CapturedFrame {
            video_path,
            position_secs,
        } => generate_default_filename(video_path, *position_secs, export_format),
    };

    Task::perform(
        async move {
            let mut dialog = rfd::AsyncFileDialog::new()
                .set_file_name(&filename)
                .add_filter(filter_name, &filter_ext);

            // Use last save directory if available
            if let Some(dir) = last_save_directory {
                if dir.exists() {
                    dialog = dialog.set_directory(&dir);
                }
            }

            dialog.save_file().await.map(|h| h.path().to_path_buf())
        },
        Message::SaveAsDialogResult,
    )
}

/// Handles editor navigation to next image.
fn handle_editor_navigate_next(ctx: &mut UpdateContext<'_>) -> Task<Message> {
    // Rescan directory to handle added/removed images
    if let Some(current_path) = ctx
        .media_navigator
        .current_media_path()
        .map(|p| p.to_path_buf())
    {
        let config = config::load().unwrap_or_default();
        let sort_order = config.sort_order.unwrap_or_default();
        let _ = ctx
            .media_navigator
            .scan_directory(&current_path, sort_order);
    }

    // Navigate to next image in the list
    if let Some(next_path) = ctx.media_navigator.navigate_next() {
        // Synchronize viewer state immediately
        ctx.viewer.current_image_path = Some(next_path.clone());
        ctx.viewer.image_list.set_current(&next_path);

        // Load the next image and create a new ImageEditorState
        Task::perform(
            async move { media::load_media(&next_path) },
            Message::ImageEditorLoaded,
        )
    } else {
        Task::none()
    }
}

/// Handles editor navigation to previous image.
fn handle_editor_navigate_previous(ctx: &mut UpdateContext<'_>) -> Task<Message> {
    // Rescan directory to handle added/removed images
    if let Some(current_path) = ctx
        .media_navigator
        .current_media_path()
        .map(|p| p.to_path_buf())
    {
        let config = config::load().unwrap_or_default();
        let sort_order = config.sort_order.unwrap_or_default();
        let _ = ctx
            .media_navigator
            .scan_directory(&current_path, sort_order);
    }

    // Navigate to previous image in the list
    if let Some(prev_path) = ctx.media_navigator.navigate_previous() {
        // Synchronize viewer state immediately
        ctx.viewer.current_image_path = Some(prev_path.clone());
        ctx.viewer.image_list.set_current(&prev_path);

        // Load the previous image and create a new ImageEditorState
        Task::perform(
            async move { media::load_media(&prev_path) },
            Message::ImageEditorLoaded,
        )
    } else {
        Task::none()
    }
}

/// Handles navbar component messages.
pub fn handle_navbar_message(
    ctx: &mut UpdateContext<'_>,
    message: navbar::Message,
) -> Task<Message> {
    match navbar::update(message, ctx.menu_open) {
        NavbarEvent::None => Task::none(),
        NavbarEvent::OpenSettings => {
            *ctx.screen = Screen::Settings;
            Task::none()
        }
        NavbarEvent::OpenHelp => {
            *ctx.screen = Screen::Help;
            Task::none()
        }
        NavbarEvent::OpenAbout => {
            *ctx.screen = Screen::About;
            Task::none()
        }
        NavbarEvent::EnterEditor => handle_screen_switch(ctx, Screen::ImageEditor),
    }
}

/// Handles help screen messages.
pub fn handle_help_message(ctx: &mut UpdateContext<'_>, message: help::Message) -> Task<Message> {
    match help::update(ctx.help_state, message) {
        HelpEvent::None => Task::none(),
        HelpEvent::BackToViewer => {
            *ctx.screen = Screen::Viewer;
            Task::none()
        }
    }
}

/// Handles about screen messages.
pub fn handle_about_message(ctx: &mut UpdateContext<'_>, message: about::Message) -> Task<Message> {
    match about::update(message) {
        AboutEvent::None => Task::none(),
        AboutEvent::BackToViewer => {
            *ctx.screen = Screen::Viewer;
            Task::none()
        }
    }
}

/// Handles navigation to next image.
pub fn handle_navigate_next(ctx: &mut UpdateContext<'_>) -> Task<Message> {
    // Rescan directory to handle added/removed images
    if let Some(current_path) = ctx
        .media_navigator
        .current_media_path()
        .map(|p| p.to_path_buf())
    {
        let config = config::load().unwrap_or_default();
        let sort_order = config.sort_order.unwrap_or_default();
        let _ = ctx
            .media_navigator
            .scan_directory(&current_path, sort_order);
    }

    // Navigate to next image
    if let Some(next_path) = ctx.media_navigator.navigate_next() {
        // Synchronize viewer state from navigator
        ctx.viewer.current_image_path = Some(next_path.clone());
        // Also sync viewer.image_list from viewer.current_image_path
        let _ = ctx.viewer.scan_directory();

        // Set loading state directly (before render)
        ctx.viewer.is_loading_media = true;
        ctx.viewer.loading_started_at = Some(std::time::Instant::now());

        // Load the next image
        Task::perform(async move { media::load_media(&next_path) }, |result| {
            Message::Viewer(component::Message::ImageLoaded(result))
        })
    } else {
        Task::none()
    }
}

/// Handles navigation to previous image.
pub fn handle_navigate_previous(ctx: &mut UpdateContext<'_>) -> Task<Message> {
    // Rescan directory to handle added/removed images
    if let Some(current_path) = ctx
        .media_navigator
        .current_media_path()
        .map(|p| p.to_path_buf())
    {
        let config = config::load().unwrap_or_default();
        let sort_order = config.sort_order.unwrap_or_default();
        let _ = ctx
            .media_navigator
            .scan_directory(&current_path, sort_order);
    }

    // Navigate to previous image
    if let Some(prev_path) = ctx.media_navigator.navigate_previous() {
        // Synchronize viewer state from navigator
        ctx.viewer.current_image_path = Some(prev_path.clone());
        // Also sync viewer.image_list from viewer.current_image_path
        let _ = ctx.viewer.scan_directory();

        // Set loading state directly (before render)
        ctx.viewer.is_loading_media = true;
        ctx.viewer.loading_started_at = Some(std::time::Instant::now());

        // Load the previous image
        Task::perform(async move { media::load_media(&prev_path) }, |result| {
            Message::Viewer(component::Message::ImageLoaded(result))
        })
    } else {
        Task::none()
    }
}

/// Handles frame capture: opens the editor with the captured frame.
pub fn handle_capture_frame(
    frame: ExportableFrame,
    video_path: PathBuf,
    position_secs: f64,
) -> Task<Message> {
    Task::done(Message::OpenImageEditorWithFrame {
        frame,
        video_path,
        position_secs,
    })
}

/// Toggles fullscreen mode.
fn toggle_fullscreen(fullscreen: &mut bool, window_id: &Option<window::Id>) -> Task<Message> {
    update_fullscreen_mode(fullscreen, window_id, !*fullscreen)
}

/// Updates fullscreen mode to the desired state.
fn update_fullscreen_mode(
    fullscreen: &mut bool,
    window_id: &Option<window::Id>,
    desired: bool,
) -> Task<Message> {
    if *fullscreen == desired {
        return Task::none();
    }

    let Some(window_id) = *window_id else {
        return Task::none();
    };

    *fullscreen = desired;
    let mode = if desired {
        window::Mode::Fullscreen
    } else {
        window::Mode::Windowed
    };
    window::set_mode(window_id, mode)
}
