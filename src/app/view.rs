// SPDX-License-Identifier: MPL-2.0
//! View rendering for the application.
//!
//! This module handles the `view()` function that renders the current screen
//! based on application state.

use super::{Message, Screen};
use crate::config;
use crate::i18n::fluent::I18n;
use crate::media::deblur::ModelStatus;
use crate::media::metadata::MediaMetadata;
use crate::media::navigator::NavigationInfo;
use crate::media::upscale::UpscaleModelStatus;
use crate::ui::about::{self, ViewContext as AboutViewContext};
use crate::ui::help::{self, ViewContext as HelpViewContext};
use crate::ui::image_editor::{self, State as ImageEditorState};
use crate::ui::metadata_panel::{self, MetadataEditorState, PanelContext as MetadataPanelContext};
use crate::ui::navbar::{self, ViewContext as NavbarViewContext};
use crate::ui::notifications::{Manager as NotificationManager, Toast};
use crate::ui::settings::{State as SettingsState, ViewContext as SettingsViewContext};
use crate::ui::viewer::component;
use iced::{
    widget::{Container, Row, Stack, Text},
    Element, Length,
};

/// Context required to render the application view.
pub struct ViewContext<'a> {
    pub i18n: &'a I18n,
    pub screen: Screen,
    pub settings: &'a SettingsState,
    pub viewer: &'a component::State,
    pub image_editor: Option<&'a ImageEditorState>,
    pub help_state: &'a crate::ui::help::State,
    pub fullscreen: bool,
    pub menu_open: bool,
    pub info_panel_open: bool,
    /// Navigation info from the central MediaNavigator (single source of truth).
    pub navigation: NavigationInfo,
    /// Current media metadata for the info panel.
    pub current_metadata: Option<&'a MediaMetadata>,
    /// Metadata editor state when in edit mode.
    pub metadata_editor_state: Option<&'a MetadataEditorState>,
    /// Current media path for save operations.
    /// Uses media_navigator as single source of truth.
    pub current_media_path: Option<&'a std::path::Path>,
    /// Whether the current media is an image (for edit button enablement).
    pub is_image: bool,
    /// Notification manager for rendering toast overlays.
    pub notifications: &'a NotificationManager,
    /// True if the application is using dark theme.
    pub is_dark_theme: bool,
    /// Current status of the AI deblur model.
    pub deblur_model_status: &'a ModelStatus,
    /// Current status of the AI upscale model.
    pub upscale_model_status: &'a UpscaleModelStatus,
    /// Whether AI upscaling is enabled for resize operations.
    pub enable_upscale: bool,
}

/// Context required to render the viewer screen.
struct ViewerViewContext<'a> {
    viewer: &'a component::State,
    i18n: &'a I18n,
    settings: &'a SettingsState,
    fullscreen: bool,
    menu_open: bool,
    info_panel_open: bool,
    navigation: NavigationInfo,
    current_metadata: Option<&'a MediaMetadata>,
    metadata_editor_state: Option<&'a MetadataEditorState>,
    current_media_path: Option<&'a std::path::Path>,
    is_image: bool,
    is_dark_theme: bool,
}

/// Renders the current application view based on the active screen.
pub fn view(ctx: ViewContext<'_>) -> Element<'_, Message> {
    let current_view: Element<'_, Message> = match ctx.screen {
        Screen::Viewer => view_viewer(ViewerViewContext {
            viewer: ctx.viewer,
            i18n: ctx.i18n,
            settings: ctx.settings,
            fullscreen: ctx.fullscreen,
            menu_open: ctx.menu_open,
            info_panel_open: ctx.info_panel_open,
            navigation: ctx.navigation,
            current_metadata: ctx.current_metadata,
            metadata_editor_state: ctx.metadata_editor_state,
            current_media_path: ctx.current_media_path,
            is_image: ctx.is_image,
            is_dark_theme: ctx.is_dark_theme,
        }),
        Screen::Settings => view_settings(ctx.settings, ctx.i18n),
        Screen::ImageEditor => view_image_editor(
            ctx.image_editor,
            ctx.i18n,
            ctx.settings,
            ctx.is_dark_theme,
            ctx.deblur_model_status,
            ctx.upscale_model_status,
            ctx.enable_upscale,
        ),
        Screen::Help => view_help(ctx.help_state, ctx.i18n),
        Screen::About => view_about(ctx.i18n),
    };

    let main_content = Container::new(current_view)
        .width(Length::Fill)
        .height(Length::Fill);

    // Render toast notifications as an overlay
    let toast_overlay = Toast::view_overlay(ctx.notifications, ctx.i18n).map(Message::Notification);

    // Stack the main content with the toast overlay
    Stack::new()
        .push(main_content)
        .push(toast_overlay)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn view_viewer(ctx: ViewerViewContext<'_>) -> Element<'_, Message> {
    let (config, _) = config::load();
    let overlay_timeout = crate::ui::state::OverlayTimeout::new(
        config
            .fullscreen
            .overlay_timeout_secs
            .unwrap_or(config::DEFAULT_OVERLAY_TIMEOUT_SECS),
    );

    let metadata_editor_has_changes = ctx
        .metadata_editor_state
        .is_some_and(|editor| editor.has_changes());

    let viewer_content = ctx
        .viewer
        .view(component::ViewEnv {
            i18n: ctx.i18n,
            background_theme: ctx.settings.background_theme(),
            is_fullscreen: ctx.fullscreen,
            overlay_hide_delay: overlay_timeout.as_duration(),
            navigation: ctx.navigation,
            metadata_editor_has_changes,
        })
        .map(Message::Viewer);

    // Build metadata panel if open
    let metadata_panel = if ctx.info_panel_open {
        Some(
            metadata_panel::panel(MetadataPanelContext {
                i18n: ctx.i18n,
                metadata: ctx.current_metadata,
                is_dark_theme: ctx.is_dark_theme,
                current_path: ctx.current_media_path,
                editor_state: ctx.metadata_editor_state,
                is_image: ctx.is_image,
            })
            .map(Message::MetadataPanel),
        )
    } else {
        None
    };

    // In fullscreen mode, don't show the navbar but show metadata panel as overlay
    if ctx.fullscreen {
        if let Some(panel) = metadata_panel {
            // Fullscreen with metadata panel: overlay on right side
            let panel_container = Container::new(panel)
                .width(Length::Shrink)
                .height(Length::Fill);

            let viewer_container = Container::new(viewer_content)
                .width(Length::Fill)
                .height(Length::Fill);

            // Use Row to push content (panel floats on right)
            Row::new()
                .push(viewer_container)
                .push(panel_container)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            viewer_content
        }
    } else {
        // Add navbar above the viewer content
        let has_media = ctx.viewer.has_media();
        let navbar_view = navbar::view(NavbarViewContext {
            i18n: ctx.i18n,
            menu_open: ctx.menu_open,
            can_edit: has_media && !ctx.viewer.is_video(),
            info_panel_open: ctx.info_panel_open,
            has_media,
            metadata_editor_has_changes,
        })
        .map(Message::Navbar);

        // Build main content with or without metadata panel
        let main_content = if let Some(panel) = metadata_panel {
            // Windowed mode with metadata panel: push layout (Row)
            let panel_container = Container::new(panel)
                .width(Length::Shrink)
                .height(Length::Fill);

            let viewer_container = Container::new(viewer_content)
                .width(Length::Fill)
                .height(Length::Fill);

            Row::new()
                .push(viewer_container)
                .push(panel_container)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            viewer_content
        };

        iced::widget::Column::new()
            .push(navbar_view)
            .push(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn view_settings<'a>(settings: &'a SettingsState, i18n: &'a I18n) -> Element<'a, Message> {
    settings
        .view(SettingsViewContext { i18n })
        .map(Message::Settings)
}

fn view_image_editor<'a>(
    image_editor: Option<&'a ImageEditorState>,
    i18n: &'a I18n,
    settings: &'a SettingsState,
    is_dark_theme: bool,
    deblur_model_status: &'a ModelStatus,
    upscale_model_status: &'a UpscaleModelStatus,
    enable_upscale: bool,
) -> Element<'a, Message> {
    if let Some(editor_state) = image_editor {
        editor_state
            .view(image_editor::ViewContext {
                i18n,
                background_theme: settings.background_theme(),
                is_dark_theme,
                deblur_model_status,
                upscale_model_status,
                enable_upscale,
            })
            .map(Message::ImageEditor)
    } else {
        // Fallback if editor state is missing
        Container::new(Text::new("Editor error"))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn view_help<'a>(help_state: &'a crate::ui::help::State, i18n: &'a I18n) -> Element<'a, Message> {
    help::view(HelpViewContext {
        i18n,
        state: help_state,
    })
    .map(Message::Help)
}

fn view_about(i18n: &I18n) -> Element<'_, Message> {
    about::view(AboutViewContext { i18n }).map(Message::About)
}
