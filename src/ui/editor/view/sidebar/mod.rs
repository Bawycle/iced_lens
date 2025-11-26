// SPDX-License-Identifier: MPL-2.0
//! Sidebar layout composition.

pub mod crop_panel;
pub mod resize_panel;

use crate::ui::editor::state::{CropState, ResizeState};
use crate::ui::theme;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{
    button, container, horizontal_rule, svg, text, tooltip, Column, Row, Scrollable,
};
use iced::{alignment::Vertical, Background, Border, Element, Length};

use super::super::{EditorTool, Message, SidebarMessage, State, ViewContext};

const ROTATE_LEFT_SVG: &str = r#"<svg viewBox='0 0 24 24' xmlns='http://www.w3.org/2000/svg'>
<path d='M11 5v-3l-4 4 4 4V7c3.309 0 6 2.691 6 6 0 1.262-.389 2.432-1.053 3.403l1.553 1.234C18.42 16.299 19 14.729 19 13c0-4.411-3.589-8-8-8z' fill='currentColor'/>
</svg>"#;

const ROTATE_RIGHT_SVG: &str = r#"<svg viewBox='0 0 24 24' xmlns='http://www.w3.org/2000/svg'>
<path d='M13 5V3l4 4-4 4V7c-3.309 0-6 2.691-6 6 0 1.262.389 2.432 1.053 3.403l-1.553 1.234C5.58 16.299 5 14.729 5 13c0-4.411-3.589-8 8-8z' fill='currentColor'/>
</svg>"#;

const SIDEBAR_WIDTH: f32 = 290.0;

pub struct SidebarModel<'a> {
    pub active_tool: Option<EditorTool>,
    pub crop_state: &'a CropState,
    pub resize_state: &'a ResizeState,
    pub can_undo: bool,
    pub can_redo: bool,
    pub has_unsaved_changes: bool,
}

impl<'a> SidebarModel<'a> {
    pub fn from_state(state: &'a State) -> Self {
        Self {
            active_tool: state.active_tool,
            crop_state: &state.crop_state,
            resize_state: &state.resize_state,
            can_undo: state.can_undo(),
            can_redo: state.can_redo(),
            has_unsaved_changes: state.has_unsaved_changes(),
        }
    }
}

pub fn expanded<'a>(model: SidebarModel<'a>, ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let mut scrollable_section = Column::new().spacing(12);

    scrollable_section =
        scrollable_section.push(undo_redo_section(model.can_undo, model.can_redo, ctx));
    scrollable_section = scrollable_section.push(horizontal_rule(1));
    scrollable_section = scrollable_section.push(rotate_section(ctx));
    scrollable_section = scrollable_section.push(horizontal_rule(1));

    let crop_button = tool_button(
        ctx.i18n.tr("editor-tool-crop"),
        SidebarMessage::SelectTool(EditorTool::Crop),
        model.active_tool == Some(EditorTool::Crop),
    );
    scrollable_section = scrollable_section.push(crop_button);
    if model.active_tool == Some(EditorTool::Crop) {
        scrollable_section = scrollable_section.push(crop_panel::panel(model.crop_state, ctx));
    }

    let resize_button = tool_button(
        ctx.i18n.tr("editor-tool-resize"),
        SidebarMessage::SelectTool(EditorTool::Resize),
        model.active_tool == Some(EditorTool::Resize),
    );
    scrollable_section = scrollable_section.push(resize_button);
    if model.active_tool == Some(EditorTool::Resize) {
        scrollable_section = scrollable_section.push(resize_panel::panel(model.resize_state, ctx));
    }

    let scrollable = Scrollable::new(scrollable_section)
        .direction(Direction::Vertical(Scrollbar::new()))
        .height(Length::Fill)
        .width(Length::Fill);

    let layout = Column::new()
        .spacing(8)
        .padding(12)
        .width(Length::Fixed(SIDEBAR_WIDTH))
        .push(header_section(ctx))
        .push(scrollable)
        .push(footer_section(model.has_unsaved_changes, ctx));

    container(layout)
        .width(Length::Fixed(SIDEBAR_WIDTH))
        .height(Length::Fill)
        .style(|_theme: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(theme::viewer_toolbar_background())),
            border: Border {
                width: 0.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

pub fn collapsed<'a>() -> Element<'a, Message> {
    let toggle_button = button(text("☰").size(24))
        .on_press(SidebarMessage::ToggleSidebar.into())
        .padding(12);

    let collapsed_bg = theme::viewer_toolbar_background();
    container(toggle_button)
        .width(Length::Fixed(60.0))
        .height(Length::Fill)
        .padding(10)
        .style(move |_theme: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(collapsed_bg)),
            border: Border {
                width: 0.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn header_section<'a>(ctx: &ViewContext<'a>) -> Column<'a, Message> {
    let toggle_button = button(text("☰").size(20))
        .on_press(SidebarMessage::ToggleSidebar.into())
        .padding(8)
        .style(iced::widget::button::secondary);

    Column::new()
        .spacing(8)
        .push(
            Row::new()
                .spacing(8)
                .align_y(Vertical::Center)
                .push(toggle_button)
                .push(text(ctx.i18n.tr("editor-title")).size(18)),
        )
        .push(horizontal_rule(1))
}

fn tool_button<'a>(label: String, message: SidebarMessage, active: bool) -> Element<'a, Message> {
    button(text(label).size(16))
        .on_press(message.into())
        .padding(12)
        .width(Length::Fill)
        .style(if active {
            iced::widget::button::primary
        } else {
            iced::widget::button::secondary
        })
        .into()
}

fn undo_redo_section<'a>(
    can_undo: bool,
    can_redo: bool,
    ctx: &ViewContext<'a>,
) -> Element<'a, Message> {
    let undo_btn = button(text(ctx.i18n.tr("editor-undo")).size(16))
        .padding(8)
        .width(Length::Fill)
        .style(iced::widget::button::secondary);
    let undo_btn = if can_undo {
        undo_btn.on_press(SidebarMessage::Undo.into())
    } else {
        undo_btn
    };

    let redo_btn = button(text(ctx.i18n.tr("editor-redo")).size(16))
        .padding(8)
        .width(Length::Fill)
        .style(iced::widget::button::secondary);
    let redo_btn = if can_redo {
        redo_btn.on_press(SidebarMessage::Redo.into())
    } else {
        redo_btn
    };

    let controls = Row::new().spacing(8).push(undo_btn).push(redo_btn);
    let title = text(ctx.i18n.tr("editor-undo-redo-section-title")).size(14);

    container(Column::new().spacing(6).push(title).push(controls))
        .padding(12)
        .width(Length::Fill)
        .style(theme::settings_panel_style)
        .into()
}

fn rotate_section<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let rotate_left_icon = svg::Svg::new(svg::Handle::from_memory(ROTATE_LEFT_SVG.as_bytes()))
        .width(Length::Fixed(28.0))
        .height(Length::Fixed(28.0));
    let rotate_right_icon = svg::Svg::new(svg::Handle::from_memory(ROTATE_RIGHT_SVG.as_bytes()))
        .width(Length::Fixed(28.0))
        .height(Length::Fixed(28.0));

    let rotate_left_btn = tooltip::Tooltip::new(
        button(rotate_left_icon)
            .on_press(SidebarMessage::RotateLeft.into())
            .padding(8)
            .width(Length::Fill)
            .style(iced::widget::button::secondary),
        text(ctx.i18n.tr("editor-rotate-left-tooltip")),
        tooltip::Position::FollowCursor,
    )
    .gap(4)
    .padding(6);

    let rotate_right_btn = tooltip::Tooltip::new(
        button(rotate_right_icon)
            .on_press(SidebarMessage::RotateRight.into())
            .padding(8)
            .width(Length::Fill)
            .style(iced::widget::button::secondary),
        text(ctx.i18n.tr("editor-rotate-right-tooltip")),
        tooltip::Position::FollowCursor,
    )
    .gap(4)
    .padding(6);

    let controls = Row::new()
        .spacing(8)
        .push(rotate_left_btn)
        .push(rotate_right_btn);
    let title = text(ctx.i18n.tr("editor-rotate-section-title")).size(14);

    container(Column::new().spacing(6).push(title).push(controls))
        .padding(12)
        .width(Length::Fill)
        .style(theme::settings_panel_style)
        .into()
}

fn footer_section<'a>(has_changes: bool, ctx: &ViewContext<'a>) -> Column<'a, Message> {
    let prev_btn = button(
        container(text("◀").size(20))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .padding([8, 16])
    .width(Length::Fill)
    .height(Length::Shrink);
    let prev_btn = if has_changes {
        prev_btn
    } else {
        prev_btn.on_press(SidebarMessage::NavigatePrevious.into())
    };

    let next_btn = button(
        container(text("▶").size(20))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .padding([8, 16])
    .width(Length::Fill)
    .height(Length::Shrink);
    let next_btn = if has_changes {
        next_btn
    } else {
        next_btn.on_press(SidebarMessage::NavigateNext.into())
    };

    let nav_row = Row::new().spacing(8).push(prev_btn).push(next_btn);

    let cancel_btn = button(text(ctx.i18n.tr("editor-cancel")).size(16))
        .padding(12)
        .width(Length::Fill)
        .style(iced::widget::button::secondary);
    let cancel_btn = if has_changes {
        cancel_btn.on_press(SidebarMessage::Cancel.into())
    } else {
        cancel_btn
    };

    let save_btn = button(text(ctx.i18n.tr("editor-save")).size(16))
        .padding(12)
        .width(Length::Fill)
        .style(iced::widget::button::primary);
    let save_btn = if has_changes {
        save_btn.on_press(SidebarMessage::Save.into())
    } else {
        save_btn
    };

    let save_as_btn = button(text(ctx.i18n.tr("editor-save-as")).size(16))
        .padding(12)
        .width(Length::Fill)
        .style(iced::widget::button::secondary);
    let save_as_btn = if has_changes {
        save_as_btn.on_press(SidebarMessage::SaveAs.into())
    } else {
        save_as_btn
    };

    Column::new()
        .spacing(8)
        .push(horizontal_rule(1))
        .push(nav_row)
        .push(horizontal_rule(1))
        .push(cancel_btn)
        .push(save_btn)
        .push(save_as_btn)
}
