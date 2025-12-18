// SPDX-License-Identifier: MPL-2.0
//! Sidebar layout composition.

pub mod adjustments_panel;
pub mod crop_panel;
pub mod deblur_panel;
pub mod resize_panel;

use crate::media::deblur::ModelStatus;
use crate::media::frame_export::ExportFormat;
use crate::ui::action_icons;
use crate::ui::design_tokens::{sizing, spacing, typography};
use crate::ui::icons;
use crate::ui::image_editor::state::{AdjustmentState, CropState, DeblurState, ResizeState};
use crate::ui::styles;
use crate::ui::styles::button as button_styles;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{button, container, rule, text, tooltip, Column, Row, Scrollable};
use iced::{alignment::Vertical, Element, Length, Padding};

use super::super::{EditorTool, Message, SidebarMessage, State, ViewContext};

const SIDEBAR_WIDTH: f32 = 310.0;

pub struct SidebarModel<'a> {
    pub active_tool: Option<EditorTool>,
    pub crop_state: &'a CropState,
    pub resize_state: &'a ResizeState,
    pub adjustment_state: &'a AdjustmentState,
    pub deblur_state: &'a DeblurState,
    pub can_undo: bool,
    pub can_redo: bool,
    pub has_unsaved_changes: bool,
    /// True if editing a captured video frame (no source file).
    pub is_captured_frame: bool,
    /// Selected export format for Save As.
    pub export_format: ExportFormat,
    /// Current status of the deblur model.
    pub deblur_model_status: &'a ModelStatus,
    /// True if deblur has already been applied to this image.
    pub has_deblur_applied: bool,
}

impl<'a> SidebarModel<'a> {
    pub fn from_state(state: &'a State, ctx: &ViewContext<'a>) -> Self {
        Self {
            active_tool: state.active_tool,
            crop_state: &state.crop_state,
            resize_state: &state.resize_state,
            adjustment_state: &state.adjustment_state,
            deblur_state: &state.deblur_state,
            can_undo: state.can_undo(),
            can_redo: state.can_redo(),
            has_unsaved_changes: state.has_unsaved_changes(),
            is_captured_frame: state.is_captured_frame(),
            export_format: state.export_format(),
            deblur_model_status: ctx.deblur_model_status,
            has_deblur_applied: state.has_deblur_applied(),
        }
    }
}

pub fn expanded<'a>(model: SidebarModel<'a>, ctx: &ViewContext<'a>) -> Element<'a, Message> {
    // Right padding provides space for the scrollbar
    let mut scrollable_section = Column::new()
        .spacing(spacing::SM)
        .padding(Padding::ZERO.right(spacing::MD));

    scrollable_section =
        scrollable_section.push(undo_redo_section(model.can_undo, model.can_redo, ctx));
    scrollable_section = scrollable_section.push(rule::horizontal(1));
    scrollable_section = scrollable_section.push(rotate_section(ctx));
    scrollable_section = scrollable_section.push(rule::horizontal(1));
    scrollable_section = scrollable_section.push(flip_section(ctx));
    scrollable_section = scrollable_section.push(rule::horizontal(1));

    let crop_button = tool_button(
        ctx.i18n.tr("image-editor-tool-crop"),
        SidebarMessage::SelectTool(EditorTool::Crop),
        model.active_tool == Some(EditorTool::Crop),
    );
    scrollable_section = scrollable_section.push(crop_button);
    if model.active_tool == Some(EditorTool::Crop) {
        scrollable_section = scrollable_section.push(crop_panel::panel(model.crop_state, ctx));
    }

    let resize_button = tool_button(
        ctx.i18n.tr("image-editor-tool-resize"),
        SidebarMessage::SelectTool(EditorTool::Resize),
        model.active_tool == Some(EditorTool::Resize),
    );
    scrollable_section = scrollable_section.push(resize_button);
    if model.active_tool == Some(EditorTool::Resize) {
        scrollable_section = scrollable_section.push(resize_panel::panel(model.resize_state, ctx));
    }

    let light_button = tool_button(
        ctx.i18n.tr("image-editor-tool-light"),
        SidebarMessage::SelectTool(EditorTool::Adjust),
        model.active_tool == Some(EditorTool::Adjust),
    );
    scrollable_section = scrollable_section.push(light_button);
    if model.active_tool == Some(EditorTool::Adjust) {
        scrollable_section =
            scrollable_section.push(adjustments_panel::panel(model.adjustment_state, ctx));
    }

    let deblur_button = tool_button(
        ctx.i18n.tr("image-editor-tool-deblur"),
        SidebarMessage::SelectTool(EditorTool::Deblur),
        model.active_tool == Some(EditorTool::Deblur),
    );
    scrollable_section = scrollable_section.push(deblur_button);
    if model.active_tool == Some(EditorTool::Deblur) {
        scrollable_section = scrollable_section.push(deblur_panel::panel(
            model.deblur_state,
            model.deblur_model_status,
            model.has_deblur_applied,
            ctx,
        ));
    }

    let scrollable = Scrollable::new(scrollable_section)
        .direction(Direction::Vertical(Scrollbar::new().margin(spacing::XXS)))
        .height(Length::Fill)
        .width(Length::Fill);

    let layout = Column::new()
        .spacing(spacing::XS)
        .padding(spacing::SM)
        .width(Length::Fixed(SIDEBAR_WIDTH))
        .push(header_section(ctx))
        .push(scrollable)
        .push(footer_section(
            model.has_unsaved_changes,
            model.is_captured_frame,
            model.export_format,
            ctx,
        ));

    container(layout)
        .width(Length::Fixed(SIDEBAR_WIDTH))
        .height(Length::Fill)
        .style(styles::editor::toolbar)
        .into()
}

pub fn collapsed<'a>(is_dark_theme: bool) -> Element<'a, Message> {
    let toggle_button = button(action_icons::sized(
        action_icons::navigation::expand_left_panel(is_dark_theme),
        sizing::ICON_SM,
    ))
    .on_press(SidebarMessage::ToggleSidebar.into())
    .padding(spacing::XXS);

    container(toggle_button)
        .width(Length::Fixed(60.0))
        .height(Length::Fill)
        .padding(spacing::SM)
        .style(styles::editor::toolbar)
        .into()
}

fn header_section<'a>(ctx: &ViewContext<'a>) -> Column<'a, Message> {
    let toggle_button = button(action_icons::sized(
        action_icons::navigation::collapse_left_panel(ctx.is_dark_theme),
        sizing::ICON_SM,
    ))
    .on_press(SidebarMessage::ToggleSidebar.into())
    .padding(spacing::XXS);

    Column::new()
        .spacing(spacing::XS)
        .push(
            Row::new()
                .spacing(spacing::XS)
                .align_y(Vertical::Center)
                .push(toggle_button)
                .push(text(ctx.i18n.tr("image-editor-title")).size(typography::TITLE_SM)),
        )
        .push(rule::horizontal(1))
}

fn tool_button<'a>(label: String, message: SidebarMessage, active: bool) -> Element<'a, Message> {
    button(text(label).size(typography::BODY_LG))
        .on_press(message.into())
        .padding(spacing::SM)
        .width(Length::Fill)
        .style(if active {
            button_styles::selected
        } else {
            button_styles::unselected
        })
        .into()
}

fn undo_redo_section<'a>(
    can_undo: bool,
    can_redo: bool,
    ctx: &ViewContext<'a>,
) -> Element<'a, Message> {
    let undo_btn = button(text(ctx.i18n.tr("image-editor-undo")).size(typography::BODY_LG))
        .padding(spacing::XS)
        .width(Length::Fill);
    let undo_btn = if can_undo {
        undo_btn.on_press(SidebarMessage::Undo.into())
    } else {
        undo_btn.style(button_styles::disabled())
    };

    let redo_btn = button(text(ctx.i18n.tr("image-editor-redo")).size(typography::BODY_LG))
        .padding(spacing::XS)
        .width(Length::Fill);
    let redo_btn = if can_redo {
        redo_btn.on_press(SidebarMessage::Redo.into())
    } else {
        redo_btn.style(button_styles::disabled())
    };

    let controls = Row::new()
        .spacing(spacing::XS)
        .push(undo_btn)
        .push(redo_btn);
    let title = text(ctx.i18n.tr("image-editor-undo-redo-section-title")).size(typography::BODY);

    container(
        Column::new()
            .spacing(spacing::XXS)
            .push(title)
            .push(controls),
    )
    .padding(spacing::SM)
    .width(Length::Fill)
    .style(styles::container::panel)
    .into()
}

fn rotate_section<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let icon_size = 28.0;

    let rotate_left_btn = tooltip::Tooltip::new(
        button(icons::sized(icons::rotate_left(), icon_size))
            .on_press(SidebarMessage::RotateLeft.into())
            .padding(spacing::XS)
            .width(Length::Fill),
        text(ctx.i18n.tr("image-editor-rotate-left-tooltip")),
        tooltip::Position::FollowCursor,
    )
    .gap(4)
    .padding(spacing::XXS);

    let rotate_right_btn = tooltip::Tooltip::new(
        button(icons::sized(icons::rotate_right(), icon_size))
            .on_press(SidebarMessage::RotateRight.into())
            .padding(spacing::XS)
            .width(Length::Fill),
        text(ctx.i18n.tr("image-editor-rotate-right-tooltip")),
        tooltip::Position::FollowCursor,
    )
    .gap(4)
    .padding(spacing::XXS);

    let controls = Row::new()
        .spacing(spacing::XS)
        .push(rotate_left_btn)
        .push(rotate_right_btn);
    let title = text(ctx.i18n.tr("image-editor-rotate-section-title")).size(typography::BODY);

    container(
        Column::new()
            .spacing(spacing::XXS)
            .push(title)
            .push(controls),
    )
    .padding(spacing::SM)
    .width(Length::Fill)
    .style(styles::container::panel)
    .into()
}

fn flip_section<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let icon_size = 28.0;

    let flip_horizontal_btn = tooltip::Tooltip::new(
        button(icons::sized(icons::flip_horizontal(), icon_size))
            .on_press(SidebarMessage::FlipHorizontal.into())
            .padding(spacing::XS)
            .width(Length::Fill),
        text(ctx.i18n.tr("image-editor-flip-horizontal-tooltip")),
        tooltip::Position::FollowCursor,
    )
    .gap(4)
    .padding(spacing::XXS);

    let flip_vertical_btn = tooltip::Tooltip::new(
        button(icons::sized(icons::flip_vertical(), icon_size))
            .on_press(SidebarMessage::FlipVertical.into())
            .padding(spacing::XS)
            .width(Length::Fill),
        text(ctx.i18n.tr("image-editor-flip-vertical-tooltip")),
        tooltip::Position::FollowCursor,
    )
    .gap(4)
    .padding(spacing::XXS);

    let controls = Row::new()
        .spacing(spacing::XS)
        .push(flip_horizontal_btn)
        .push(flip_vertical_btn);
    let title = text(ctx.i18n.tr("image-editor-flip-section-title")).size(typography::BODY);

    container(
        Column::new()
            .spacing(spacing::XXS)
            .push(title)
            .push(controls),
    )
    .padding(spacing::SM)
    .width(Length::Fill)
    .style(styles::container::panel)
    .into()
}

fn footer_section<'a>(
    has_changes: bool,
    is_captured_frame: bool,
    export_format: ExportFormat,
    ctx: &ViewContext<'a>,
) -> Column<'a, Message> {
    let mut footer = Column::new().spacing(spacing::XS).push(rule::horizontal(1));

    // Navigation buttons - only for file mode, not captured frames
    if !is_captured_frame {
        let prev_btn = button(
            container(text("◀").size(typography::TITLE_MD))
                .center_x(Length::Fill)
                .center_y(Length::Shrink),
        )
        .padding([spacing::XS, spacing::MD])
        .width(Length::Fill)
        .height(Length::Shrink);
        let prev_btn = if has_changes {
            prev_btn.style(button_styles::disabled())
        } else {
            prev_btn.on_press(SidebarMessage::NavigatePrevious.into())
        };

        let next_btn = button(
            container(text("▶").size(typography::TITLE_MD))
                .center_x(Length::Fill)
                .center_y(Length::Shrink),
        )
        .padding([spacing::XS, spacing::MD])
        .width(Length::Fill)
        .height(Length::Shrink);
        let next_btn = if has_changes {
            next_btn.style(button_styles::disabled())
        } else {
            next_btn.on_press(SidebarMessage::NavigateNext.into())
        };

        let nav_row = Row::new()
            .spacing(spacing::XS)
            .push(prev_btn)
            .push(next_btn);
        footer = footer.push(nav_row).push(rule::horizontal(1));
    }

    // Cancel button - available when there are changes
    let cancel_btn = button(text(ctx.i18n.tr("image-editor-cancel")).size(typography::BODY_LG))
        .padding(spacing::SM)
        .width(Length::Fill);
    let cancel_btn = if has_changes {
        cancel_btn.on_press(SidebarMessage::Cancel.into())
    } else {
        cancel_btn.style(button_styles::disabled())
    };
    footer = footer.push(cancel_btn);

    // Save button - only for file mode, not captured frames
    if !is_captured_frame {
        let save_btn = button(text(ctx.i18n.tr("image-editor-save")).size(typography::BODY_LG))
            .padding(spacing::SM)
            .width(Length::Fill);
        let save_btn = if has_changes {
            save_btn.on_press(SidebarMessage::Save.into())
        } else {
            save_btn.style(button_styles::disabled())
        };
        footer = footer.push(save_btn);
    }

    // Export format selector - shown before Save As button
    footer = footer.push(export_format_section(export_format, ctx));

    // Save As button
    let save_as_btn = button(text(ctx.i18n.tr("image-editor-save-as")).size(typography::BODY_LG))
        .padding(spacing::SM)
        .width(Length::Fill);
    // For captured frames, Save As is always enabled (it's the only way to save)
    let save_as_btn = if is_captured_frame || has_changes {
        save_as_btn.on_press(SidebarMessage::SaveAs.into())
    } else {
        save_as_btn.style(button_styles::disabled())
    };
    footer = footer.push(save_as_btn);

    footer
}

/// Export format selector for Save As operations.
fn export_format_section<'a>(
    current_format: ExportFormat,
    ctx: &ViewContext<'a>,
) -> Element<'a, Message> {
    let format_label = text(ctx.i18n.tr("image-editor-export-format-label")).size(typography::BODY);

    let format_buttons: Vec<Element<'a, Message>> = ExportFormat::all()
        .iter()
        .map(|&format| {
            let is_selected = format == current_format;
            let label = match format {
                ExportFormat::Png => "PNG",
                ExportFormat::Jpeg => "JPEG",
                ExportFormat::WebP => "WebP",
            };

            button(text(label).size(typography::BODY))
                .padding([spacing::XS, spacing::SM])
                .width(Length::FillPortion(1))
                .style(if is_selected {
                    button_styles::selected
                } else {
                    button_styles::unselected
                })
                .on_press(SidebarMessage::SetExportFormat(format).into())
                .into()
        })
        .collect();

    let format_row = Row::with_children(format_buttons)
        .spacing(spacing::XXS)
        .width(Length::Fill);

    container(
        Column::new()
            .spacing(spacing::XXS)
            .push(format_label)
            .push(format_row),
    )
    .padding(spacing::SM)
    .width(Length::Fill)
    .style(styles::container::panel)
    .into()
}
