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
use crate::ui::design_tokens::spacing;
use crate::ui::diagnostics_screen::{self, ViewContext as DiagnosticsViewContext};
use crate::ui::help::{self, ViewContext as HelpViewContext};
use crate::ui::image_editor::{self, State as ImageEditorState};
use crate::ui::metadata_panel::{self, MetadataEditorState, PanelContext as MetadataPanelContext};
use crate::ui::navbar::{self, ViewContext as NavbarViewContext};
use crate::ui::notifications::{Manager as NotificationManager, Toast};
use crate::ui::settings::{State as SettingsState, ViewContext as SettingsViewContext};
use crate::ui::viewer::{component, filter_dropdown};
use iced::{
    widget::{mouse_area, Container, Row, Stack, Text},
    Element, Length,
};

/// Context required to render the application view.
// Allow excessive bools: UI context structs legitimately need multiple boolean flags
// for distinct display states (fullscreen, menu, panel, theme).
#[allow(clippy::struct_excessive_bools)]
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
    /// Navigation info from the central `MediaNavigator` (single source of truth).
    pub navigation: NavigationInfo,
    /// Current media metadata for the info panel.
    pub current_metadata: Option<&'a MediaMetadata>,
    /// Metadata editor state when in edit mode.
    pub metadata_editor_state: Option<&'a MetadataEditorState>,
    /// Current media path for save operations.
    /// Uses `media_navigator` as single source of truth.
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
    /// Current media filter (from navigator).
    pub filter: &'a crate::media::filter::MediaFilter,
    /// Total count of media files in directory.
    pub total_count: usize,
    /// Filtered count of media files.
    pub filtered_count: usize,
}

/// Context required to render the viewer screen.
// Allow excessive bools: viewer context has multiple distinct boolean display flags.
#[allow(clippy::struct_excessive_bools)]
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
    filter: &'a crate::media::filter::MediaFilter,
    /// Total count of media files in directory.
    total_count: usize,
    /// Filtered count of media files.
    filtered_count: usize,
}

/// Renders the current application view based on the active screen.
// Allow pass-by-value: ViewContext contains references and is cheap to move.
// Passing by reference would require complex lifetime annotations that conflict
// with the returned Element's lifetime requirements.
#[allow(clippy::needless_pass_by_value)]
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
            filter: ctx.filter,
            total_count: ctx.total_count,
            filtered_count: ctx.filtered_count,
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
        Screen::Help => view_help(ctx.help_state, ctx.i18n, ctx.is_dark_theme),
        Screen::About => view_about(ctx.i18n),
        Screen::Diagnostics => view_diagnostics(ctx.i18n),
    };

    let main_content = Container::new(current_view)
        .width(Length::Fill)
        .height(Length::Fill);

    // Render toast notifications as an overlay
    let toast_overlay = Toast::view_overlay(ctx.notifications, ctx.i18n).map(Message::Notification);

    // Build filter dropdown overlay (only on Viewer screen, not in fullscreen)
    let filter_overlay: Option<Element<'_, Message>> = if matches!(ctx.screen, Screen::Viewer)
        && !ctx.fullscreen
    {
        let filter_dropdown_state = ctx.viewer.filter_dropdown_state();
        if filter_dropdown_state.is_open {
            filter_dropdown::view_panel(filter_dropdown::ViewContext {
                i18n: ctx.i18n,
                filter: ctx.filter,
                state: filter_dropdown_state,
                total_count: ctx.total_count,
                filtered_count: ctx.filtered_count,
            })
            .map(|panel| {
                let mapped_panel =
                    panel.map(|msg| Message::Navbar(navbar::Message::FilterDropdown(msg)));

                // Position panel below navbar, aligned to left
                let navbar_height = spacing::SM * 2.0 + 32.0;

                // Wrap panel in mouse_area to prevent clicks from closing dropdown
                let panel_with_click_guard = mouse_area(mapped_panel).on_press(Message::Navbar(
                    navbar::Message::FilterDropdown(filter_dropdown::Message::ConsumeClick),
                ));

                Container::new(panel_with_click_guard)
                    .width(Length::Shrink)
                    .padding(iced::Padding {
                        top: navbar_height,
                        right: 0.0,
                        bottom: 0.0,
                        left: spacing::SM,
                    })
                    .into()
            })
        } else {
            None
        }
    } else {
        None
    };

    // Stack the main content with overlays
    let mut stack = Stack::new()
        .width(Length::Fill)
        .height(Length::Fill)
        .push(main_content);

    // Add click-outside overlay and filter panel if open
    if let Some(panel) = filter_overlay {
        // Full-screen click catcher to close dropdown when clicking outside
        let click_outside = mouse_area(
            Container::new(Text::new(""))
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(Message::Navbar(navbar::Message::FilterDropdown(
            filter_dropdown::Message::CloseDropdown,
        )));
        stack = stack.push(click_outside);
        stack = stack.push(panel);
    }

    stack.push(toast_overlay).into()
}

// Allow pass-by-value: ViewerViewContext contains references and is cheap to move.
#[allow(clippy::needless_pass_by_value)]
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
        .is_some_and(MetadataEditorState::has_changes);

    let viewer_content = ctx
        .viewer
        .view(component::ViewEnv {
            i18n: ctx.i18n,
            background_theme: ctx.settings.background_theme(),
            is_fullscreen: ctx.fullscreen,
            overlay_hide_delay: overlay_timeout.as_duration(),
            navigation: ctx.navigation,
            metadata_editor_has_changes,
            filter: ctx.filter,
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
            filter: ctx.filter,
            filter_dropdown: ctx.viewer.filter_dropdown_state(),
            total_count: ctx.total_count,
            filtered_count: ctx.filtered_count,
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
            .view(&image_editor::ViewContext {
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

fn view_help<'a>(
    help_state: &'a crate::ui::help::State,
    i18n: &'a I18n,
    is_dark_theme: bool,
) -> Element<'a, Message> {
    help::view(&HelpViewContext {
        i18n,
        state: help_state,
        is_dark_theme,
    })
    .map(Message::Help)
}

fn view_about(i18n: &I18n) -> Element<'_, Message> {
    about::view(AboutViewContext { i18n }).map(Message::About)
}

fn view_diagnostics(i18n: &I18n) -> Element<'_, Message> {
    diagnostics_screen::view(DiagnosticsViewContext { i18n }).map(Message::Diagnostics)
}
