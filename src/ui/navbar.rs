// SPDX-License-Identifier: MPL-2.0
//! Navigation bar module for app-level navigation.
//!
//! This module provides the hamburger menu and edit button that appear
//! at the top of the viewer screen. The menu provides access to Settings,
//! Help, and About screens.

use crate::i18n::fluent::I18n;
use crate::ui::action_icons;
use crate::ui::design_tokens::{radius, sizing, spacing};
use crate::ui::icons;
use crate::ui::styles;
use iced::widget::image::{Handle, Image};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{button, container, Column, Container, Row, Text},
    Border, Element, Length, Theme,
};

/// Contextual data needed to render the navbar.
pub struct ViewContext<'a> {
    pub i18n: &'a I18n,
    pub menu_open: bool,
    pub can_edit: bool,
    pub info_panel_open: bool,
    /// Whether media is loaded (used to enable/disable info button).
    pub has_media: bool,
    /// Whether metadata editor has unsaved changes (disables edit button).
    pub metadata_editor_has_changes: bool,
}

/// Messages emitted by the navbar.
#[derive(Debug, Clone)]
pub enum Message {
    ToggleMenu,
    CloseMenu,
    OpenSettings,
    OpenHelp,
    OpenAbout,
    EnterEditor,
    ToggleInfoPanel,
}

/// Events propagated to the parent application.
#[derive(Debug, Clone)]
pub enum Event {
    None,
    OpenSettings,
    OpenHelp,
    OpenAbout,
    EnterEditor,
    ToggleInfoPanel,
}

/// Process a navbar message and return the corresponding event.
pub fn update(message: Message, menu_open: &mut bool) -> Event {
    match message {
        Message::ToggleMenu => {
            *menu_open = !*menu_open;
            Event::None
        }
        Message::CloseMenu => {
            *menu_open = false;
            Event::None
        }
        Message::OpenSettings => {
            *menu_open = false;
            Event::OpenSettings
        }
        Message::OpenHelp => {
            *menu_open = false;
            Event::OpenHelp
        }
        Message::OpenAbout => {
            *menu_open = false;
            Event::OpenAbout
        }
        Message::EnterEditor => {
            *menu_open = false;
            Event::EnterEditor
        }
        Message::ToggleInfoPanel => {
            *menu_open = false;
            Event::ToggleInfoPanel
        }
    }
}

/// Render the navigation bar.
pub fn view<'a>(ctx: ViewContext<'a>) -> Element<'a, Message> {
    let mut content = Column::new().width(Length::Fill);

    // Top bar with hamburger menu and edit button
    let top_bar = build_top_bar(&ctx);
    content = content.push(top_bar);

    // Dropdown menu (if open)
    if ctx.menu_open {
        let dropdown = build_dropdown(&ctx);
        content = content.push(dropdown);
    }

    content.into()
}

/// Build the top bar with hamburger menu button, edit button, and info button.
fn build_top_bar<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let menu_button = button(icons::sized(
        action_icons::navigation::menu(),
        sizing::ICON_MD,
    ))
    .on_press(Message::ToggleMenu)
    .padding(spacing::XS);

    let edit_label = ctx.i18n.tr("navbar-edit-button");
    let edit_button = if ctx.metadata_editor_has_changes {
        // Disabled: metadata editor has unsaved changes
        button(Text::new(edit_label)).style(styles::button::disabled())
    } else if ctx.can_edit {
        button(Text::new(edit_label)).on_press(Message::EnterEditor)
    } else {
        button(Text::new(edit_label)).style(styles::button::disabled())
    };

    // Info button with toggle styling (highlighted when panel is open)
    // Disabled when:
    // - No media is loaded (no metadata to show)
    // - Panel is open with unsaved changes (can't close it)
    let info_label = ctx.i18n.tr("navbar-info-button");
    let info_button = if !ctx.has_media {
        // No media: disabled
        button(Text::new(info_label)).style(styles::button::disabled())
    } else if ctx.info_panel_open && ctx.metadata_editor_has_changes {
        // Panel open with unsaved changes: disabled (can't close)
        button(Text::new(info_label)).style(styles::button::selected)
    } else if ctx.info_panel_open {
        // Panel open, no unsaved changes: can close
        button(Text::new(info_label))
            .on_press(Message::ToggleInfoPanel)
            .style(styles::button::selected)
    } else {
        // Panel closed: can open
        button(Text::new(info_label)).on_press(Message::ToggleInfoPanel)
    };

    let row = Row::new()
        .spacing(spacing::SM)
        .padding(spacing::SM)
        .align_y(Vertical::Center)
        .push(menu_button)
        .push(edit_button)
        .push(info_button);

    Container::new(row)
        .width(Length::Fill)
        .align_x(Horizontal::Left)
        .style(styles::editor::toolbar)
        .into()
}

/// Build the dropdown menu with Settings, Help, and About options.
fn build_dropdown<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let settings_item = build_menu_item(
        icons::cog(),
        ctx.i18n.tr("menu-settings"),
        Message::OpenSettings,
    );

    let help_item = build_menu_item(icons::help(), ctx.i18n.tr("menu-help"), Message::OpenHelp);

    let about_item = build_menu_item(icons::info(), ctx.i18n.tr("menu-about"), Message::OpenAbout);

    let menu_column = Column::new()
        .spacing(spacing::XXS)
        .push(settings_item)
        .push(help_item)
        .push(about_item);

    Container::new(menu_column)
        .padding(spacing::XS)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.extended_palette().background.weak.color.into()),
            border: Border {
                radius: radius::SM.into(),
                width: 1.0,
                color: theme.extended_palette().background.strong.color,
            },
            ..Default::default()
        })
        .into()
}

/// Build a single menu item with icon and label.
fn build_menu_item<'a>(
    icon: Image<Handle>,
    label: String,
    message: Message,
) -> Element<'a, Message> {
    let icon_sized = icons::sized(icon, sizing::ICON_SM);

    let row = Row::new()
        .spacing(spacing::SM)
        .align_y(Vertical::Center)
        .push(icon_sized)
        .push(Text::new(label));

    button(row)
        .on_press(message)
        .padding([spacing::XS, spacing::SM])
        .width(Length::Fill)
        .style(menu_item_style)
        .into()
}

/// Style function for menu items.
fn menu_item_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    match status {
        button::Status::Active => button::Style {
            background: None,
            text_color: palette.background.base.text,
            border: Border::default(),
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(palette.background.strong.color.into()),
            text_color: palette.background.base.text,
            border: Border {
                radius: radius::SM.into(),
                ..Default::default()
            },
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(palette.primary.strong.color.into()),
            text_color: palette.primary.strong.text,
            border: Border {
                radius: radius::SM.into(),
                ..Default::default()
            },
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: None,
            text_color: palette.background.weak.text,
            border: Border::default(),
            ..Default::default()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::fluent::I18n;

    #[test]
    fn navbar_view_renders() {
        let i18n = I18n::default();
        let ctx = ViewContext {
            i18n: &i18n,
            menu_open: false,
            can_edit: true,
            info_panel_open: false,
            has_media: true,
            metadata_editor_has_changes: false,
        };
        let _element = view(ctx);
    }

    #[test]
    fn navbar_view_renders_with_menu_open() {
        let i18n = I18n::default();
        let ctx = ViewContext {
            i18n: &i18n,
            menu_open: true,
            can_edit: true,
            info_panel_open: false,
            has_media: true,
            metadata_editor_has_changes: false,
        };
        let _element = view(ctx);
    }

    #[test]
    fn navbar_view_renders_with_info_panel_open() {
        let i18n = I18n::default();
        let ctx = ViewContext {
            i18n: &i18n,
            menu_open: false,
            can_edit: true,
            info_panel_open: true,
            has_media: true,
            metadata_editor_has_changes: false,
        };
        let _element = view(ctx);
    }

    #[test]
    fn navbar_view_renders_without_media() {
        let i18n = I18n::default();
        let ctx = ViewContext {
            i18n: &i18n,
            menu_open: false,
            can_edit: false,
            info_panel_open: false,
            has_media: false,
            metadata_editor_has_changes: false,
        };
        let _element = view(ctx);
    }

    #[test]
    fn toggle_info_panel_emits_event() {
        let mut menu_open = true;
        let event = update(Message::ToggleInfoPanel, &mut menu_open);
        assert!(!menu_open); // Menu closes
        assert!(matches!(event, Event::ToggleInfoPanel));
    }

    #[test]
    fn toggle_menu_changes_state() {
        let mut menu_open = false;
        let event = update(Message::ToggleMenu, &mut menu_open);
        assert!(menu_open);
        assert!(matches!(event, Event::None));

        let event = update(Message::ToggleMenu, &mut menu_open);
        assert!(!menu_open);
        assert!(matches!(event, Event::None));
    }

    #[test]
    fn menu_items_close_menu_and_emit_event() {
        let mut menu_open = true;

        let event = update(Message::OpenSettings, &mut menu_open);
        assert!(!menu_open);
        assert!(matches!(event, Event::OpenSettings));

        menu_open = true;
        let event = update(Message::OpenHelp, &mut menu_open);
        assert!(!menu_open);
        assert!(matches!(event, Event::OpenHelp));

        menu_open = true;
        let event = update(Message::OpenAbout, &mut menu_open);
        assert!(!menu_open);
        assert!(matches!(event, Event::OpenAbout));
    }
}
