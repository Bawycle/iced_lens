// SPDX-License-Identifier: MPL-2.0
//! View rendering for the application.
//!
//! This module handles the `view()` function that renders the current screen
//! based on application state.

use super::{Message, Screen};
use crate::config;
use crate::i18n::fluent::I18n;
use crate::ui::about::{self, ViewContext as AboutViewContext};
use crate::ui::help::{self, ViewContext as HelpViewContext};
use crate::ui::image_editor::{self, State as ImageEditorState};
use crate::ui::navbar::{self, ViewContext as NavbarViewContext};
use crate::ui::settings::{State as SettingsState, ViewContext as SettingsViewContext};
use crate::ui::viewer::component;
use iced::{
    widget::{Container, Text},
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
}

/// Renders the current application view based on the active screen.
pub fn view(ctx: ViewContext<'_>) -> Element<'_, Message> {
    let current_view: Element<'_, Message> = match ctx.screen {
        Screen::Viewer => view_viewer(ctx.viewer, ctx.i18n, ctx.settings, ctx.fullscreen, ctx.menu_open),
        Screen::Settings => view_settings(ctx.settings, ctx.i18n),
        Screen::ImageEditor => view_image_editor(ctx.image_editor, ctx.i18n, ctx.settings),
        Screen::Help => view_help(ctx.help_state, ctx.i18n),
        Screen::About => view_about(ctx.i18n),
    };

    let column = iced::widget::Column::new().push(
        Container::new(current_view)
            .width(Length::Fill)
            .height(Length::Fill),
    );

    Container::new(column.width(Length::Fill).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn view_viewer<'a>(
    viewer: &'a component::State,
    i18n: &'a I18n,
    settings: &'a SettingsState,
    fullscreen: bool,
    menu_open: bool,
) -> Element<'a, Message> {
    let config = config::load().unwrap_or_default();
    let overlay_timeout_secs = config
        .overlay_timeout_secs
        .unwrap_or(config::DEFAULT_OVERLAY_TIMEOUT_SECS);

    let viewer_content = viewer
        .view(component::ViewEnv {
            i18n,
            background_theme: settings.background_theme(),
            is_fullscreen: fullscreen,
            overlay_hide_delay: std::time::Duration::from_secs(overlay_timeout_secs as u64),
        })
        .map(Message::Viewer);

    // In fullscreen mode, don't show the navbar
    if fullscreen {
        viewer_content
    } else {
        // Add navbar above the viewer content
        let navbar_view = navbar::view(NavbarViewContext {
            i18n,
            menu_open,
            can_edit: !viewer.is_video(),
        })
        .map(Message::Navbar);

        iced::widget::Column::new()
            .push(navbar_view)
            .push(viewer_content)
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
) -> Element<'a, Message> {
    if let Some(editor_state) = image_editor {
        editor_state
            .view(image_editor::ViewContext {
                i18n,
                background_theme: settings.background_theme(),
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
