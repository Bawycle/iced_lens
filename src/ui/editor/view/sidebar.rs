// SPDX-License-Identifier: MPL-2.0
//! Sidebar layout for the editor.

use crate::ui::editor::state::CropRatio;
use crate::ui::theme;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{
    button, checkbox, container, horizontal_rule, slider, svg, text, text_input, tooltip, Column,
    Row, Scrollable,
};
use iced::{alignment::Vertical, Background, Border, Element, Length};

use super::super::{
    EditorTool, Message, State, ViewContext, ROTATE_LEFT_SVG, ROTATE_RIGHT_SVG, SIDEBAR_WIDTH,
};

pub fn expanded<'a>(state: &'a State, ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let mut scrollable_section = Column::new().spacing(12);

    scrollable_section = scrollable_section.push(undo_redo_section(state, ctx));
    scrollable_section = scrollable_section.push(horizontal_rule(1));
    scrollable_section = scrollable_section.push(rotate_section(ctx));
    scrollable_section = scrollable_section.push(horizontal_rule(1));

    let crop_button = tool_button(
        ctx.i18n.tr("editor-tool-crop"),
        Message::SelectTool(EditorTool::Crop),
        state.active_tool == Some(EditorTool::Crop),
    );
    scrollable_section = scrollable_section.push(crop_button);
    if state.active_tool == Some(EditorTool::Crop) {
        scrollable_section = scrollable_section.push(crop_panel(state, ctx));
    }

    let resize_button = tool_button(
        ctx.i18n.tr("editor-tool-resize"),
        Message::SelectTool(EditorTool::Resize),
        state.active_tool == Some(EditorTool::Resize),
    );
    scrollable_section = scrollable_section.push(resize_button);
    if state.active_tool == Some(EditorTool::Resize) {
        scrollable_section = scrollable_section.push(resize_panel(state, ctx));
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
        .push(footer_section(state, ctx));

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

fn header_section<'a>(ctx: &ViewContext<'a>) -> Column<'a, Message> {
    let toggle_button = button(text("☰").size(20))
        .on_press(Message::ToggleSidebar)
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

fn tool_button<'a>(label: String, message: Message, active: bool) -> Element<'a, Message> {
    button(text(label).size(16))
        .on_press(message)
        .padding(12)
        .width(Length::Fill)
        .style(if active {
            iced::widget::button::primary
        } else {
            iced::widget::button::secondary
        })
        .into()
}

fn undo_redo_section<'a>(state: &'a State, ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let undo_btn = button(text(ctx.i18n.tr("editor-undo")).size(16))
        .padding(8)
        .width(Length::Fill)
        .style(iced::widget::button::secondary);
    let undo_btn = if state.can_undo() {
        undo_btn.on_press(Message::Undo)
    } else {
        undo_btn
    };

    let redo_btn = button(text(ctx.i18n.tr("editor-redo")).size(16))
        .padding(8)
        .width(Length::Fill)
        .style(iced::widget::button::secondary);
    let redo_btn = if state.can_redo() {
        redo_btn.on_press(Message::Redo)
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
            .on_press(Message::RotateLeft)
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
            .on_press(Message::RotateRight)
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

fn crop_panel<'a>(state: &'a State, ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let title = text(ctx.i18n.tr("editor-crop-section-title")).size(14);
    let ratio_label = text(ctx.i18n.tr("editor-crop-ratio-label")).size(13);

    let ratios_row1 = Row::new()
        .spacing(4)
        .push(crop_ratio_button(
            state,
            ctx.i18n.tr("editor-crop-ratio-free"),
            CropRatio::Free,
        ))
        .push(crop_ratio_button(
            state,
            ctx.i18n.tr("editor-crop-ratio-square"),
            CropRatio::Square,
        ));

    let ratios_row2 = Row::new()
        .spacing(4)
        .push(crop_ratio_button(
            state,
            ctx.i18n.tr("editor-crop-ratio-landscape"),
            CropRatio::Landscape,
        ))
        .push(crop_ratio_button(
            state,
            ctx.i18n.tr("editor-crop-ratio-portrait"),
            CropRatio::Portrait,
        ));

    let ratios_row3 = Row::new()
        .spacing(4)
        .push(crop_ratio_button(
            state,
            ctx.i18n.tr("editor-crop-ratio-photo"),
            CropRatio::Photo,
        ))
        .push(crop_ratio_button(
            state,
            ctx.i18n.tr("editor-crop-ratio-photo-portrait"),
            CropRatio::PhotoPortrait,
        ));

    let crop_info = text(format!(
        "{}×{} px",
        state.crop_state.width, state.crop_state.height
    ))
    .size(12);

    let apply_btn = {
        let btn = button(text(ctx.i18n.tr("editor-crop-apply")).size(14))
            .padding(8)
            .width(Length::Fill)
            .style(iced::widget::button::primary);
        if state.crop_state.overlay.visible {
            btn.on_press(Message::ApplyCrop)
        } else {
            btn
        }
    };

    container(
        Column::new()
            .spacing(8)
            .push(title)
            .push(ratio_label)
            .push(ratios_row1)
            .push(ratios_row2)
            .push(ratios_row3)
            .push(crop_info)
            .push(apply_btn),
    )
    .padding(12)
    .width(Length::Fill)
    .style(theme::settings_panel_style)
    .into()
}

fn crop_ratio_button<'a>(
    state: &'a State,
    label: String,
    ratio: CropRatio,
) -> Element<'a, Message> {
    let selected = state.crop_state.ratio == ratio;
    button(text(label).size(11))
        .on_press(Message::SetCropRatio(ratio))
        .padding([4, 6])
        .width(Length::Fill)
        .style(if selected {
            iced::widget::button::primary
        } else {
            iced::widget::button::secondary
        })
        .into()
}

fn resize_panel<'a>(state: &'a State, ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let scale_section = Column::new()
        .spacing(6)
        .push(text(ctx.i18n.tr("editor-resize-section-title")).size(14))
        .push(text(ctx.i18n.tr("editor-resize-scale-label")).size(13))
        .push(
            slider(
                10.0..=200.0,
                state.resize_state.scale_percent,
                Message::ScaleChanged,
            )
            .step(1.0),
        )
        .push(text(format!("{:.0}%", state.resize_state.scale_percent)).size(13));

    let mut presets = Row::new().spacing(8);
    for preset in [50.0, 75.0, 150.0, 200.0] {
        let label = format!("{preset:.0}%");
        presets = presets.push(
            button(text(label))
                .on_press(Message::ApplyResizePreset(preset))
                .padding([6, 8])
                .style(iced::widget::button::secondary),
        );
    }

    let presets_section = Column::new()
        .spacing(6)
        .push(text(ctx.i18n.tr("editor-resize-presets-label")).size(13))
        .push(presets);

    let width_placeholder = ctx.i18n.tr("editor-resize-width-label");
    let width_label = text(width_placeholder.clone()).size(13);
    let width_input = text_input(width_placeholder.as_str(), &state.resize_state.width_input)
        .on_input(Message::WidthInputChanged)
        .padding(6)
        .size(14)
        .width(Length::Fill);

    let height_placeholder = ctx.i18n.tr("editor-resize-height-label");
    let height_label = text(height_placeholder.clone()).size(13);
    let height_input = text_input(
        height_placeholder.as_str(),
        &state.resize_state.height_input,
    )
    .on_input(Message::HeightInputChanged)
    .padding(6)
    .size(14)
    .width(Length::Fill);

    let dimensions_row = Row::new()
        .spacing(8)
        .push(
            Column::new()
                .spacing(4)
                .width(Length::Fill)
                .push(width_label)
                .push(width_input),
        )
        .push(
            Column::new()
                .spacing(4)
                .width(Length::Fill)
                .push(height_label)
                .push(height_input),
        );

    let lock_checkbox = checkbox(
        ctx.i18n.tr("editor-resize-lock-aspect"),
        state.resize_state.lock_aspect,
    )
    .on_toggle(|_| Message::ToggleLockAspect);

    let apply_btn = button(text(ctx.i18n.tr("editor-resize-apply")).size(16))
        .padding(10)
        .width(Length::Fill)
        .style(iced::widget::button::primary)
        .on_press(Message::ApplyResize);

    container(
        Column::new()
            .spacing(12)
            .push(scale_section)
            .push(presets_section)
            .push(text(ctx.i18n.tr("editor-resize-dimensions-label")).size(13))
            .push(dimensions_row)
            .push(lock_checkbox)
            .push(apply_btn),
    )
    .padding(12)
    .width(Length::Fill)
    .style(theme::settings_panel_style)
    .into()
}

fn footer_section<'a>(state: &'a State, ctx: &ViewContext<'a>) -> Column<'a, Message> {
    let has_changes = state.has_unsaved_changes();

    let prev_btn = button(text("◀").size(20))
        .padding([8, 16])
        .width(Length::Fill);
    let prev_btn = if has_changes {
        prev_btn
    } else {
        prev_btn.on_press(Message::NavigatePrevious)
    };

    let next_btn = button(text("▶").size(20))
        .padding([8, 16])
        .width(Length::Fill);
    let next_btn = if has_changes {
        next_btn
    } else {
        next_btn.on_press(Message::NavigateNext)
    };

    let nav_row = Row::new().spacing(8).push(prev_btn).push(next_btn);

    let cancel_btn = button(text(ctx.i18n.tr("editor-cancel")).size(16))
        .padding(12)
        .width(Length::Fill)
        .style(iced::widget::button::secondary);
    let cancel_btn = if has_changes {
        cancel_btn.on_press(Message::Cancel)
    } else {
        cancel_btn
    };

    let save_btn = button(text(ctx.i18n.tr("editor-save")).size(16))
        .padding(12)
        .width(Length::Fill)
        .style(iced::widget::button::primary);
    let save_btn = if has_changes {
        save_btn.on_press(Message::Save)
    } else {
        save_btn
    };

    let save_as_btn = button(text(ctx.i18n.tr("editor-save-as")).size(16))
        .padding(12)
        .width(Length::Fill)
        .style(iced::widget::button::secondary);
    let save_as_btn = if has_changes {
        save_as_btn.on_press(Message::SaveAs)
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
