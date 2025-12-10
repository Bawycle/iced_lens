// SPDX-License-Identifier: MPL-2.0
//! Viewer controls: zoom inputs, buttons, and fit-to-window toggle.

use crate::i18n::fluent::I18n;
use crate::ui::icons;
use crate::ui::state::zoom::ZoomState;
use crate::ui::styles;
use crate::ui::theme;
use crate::ui::viewer::shared_styles;
use iced::{
    alignment::Vertical,
    widget::{button, text, text_input, tooltip, Column, Row, Space, Text},
    Element, Length, Theme,
};

#[derive(Clone)]
pub struct ViewContext<'a> {
    pub i18n: &'a I18n,
}

#[derive(Debug, Clone)]
pub enum Message {
    ZoomInputChanged(String),
    ZoomInputSubmitted,
    ResetZoom,
    ZoomIn,
    ZoomOut,
    SetFitToWindow(bool),
    ToggleFullscreen,
    DeleteCurrentImage,
}

pub fn view<'a>(
    ctx: ViewContext<'a>,
    zoom: &'a ZoomState,
    effective_fit_to_window: bool,
) -> Element<'a, Message> {
    let zoom_placeholder = ctx.i18n.tr("viewer-zoom-input-placeholder");
    let zoom_label = Text::new(ctx.i18n.tr("viewer-zoom-label"));

    let zoom_input = text_input(&zoom_placeholder, &zoom.zoom_input)
        .on_input(Message::ZoomInputChanged)
        .on_submit(Message::ZoomInputSubmitted)
        .padding(6)
        .size(16)
        .width(Length::Fixed(60.0));

    let zoom_percent_label = Text::new("%").size(16);

    let reset_tooltip = ctx.i18n.tr("viewer-zoom-reset-button");
    let reset_button_content: Element<'_, Message> = button(icons::fill(icons::refresh()))
        .on_press(Message::ResetZoom)
        .padding(4)
        .width(Length::Fixed(shared_styles::ICON_SIZE))
        .height(Length::Fixed(shared_styles::ICON_SIZE))
        .into();
    let reset_button = tooltip(
        reset_button_content,
        Text::new(reset_tooltip),
        tooltip::Position::Bottom,
    )
    .gap(4);

    let zoom_out_tooltip = ctx.i18n.tr("viewer-zoom-out-tooltip");
    let zoom_out_button_content: Element<'_, Message> = button(icons::fill(icons::zoom_out()))
        .on_press(Message::ZoomOut)
        .padding(4)
        .width(Length::Fixed(shared_styles::ICON_SIZE))
        .height(Length::Fixed(shared_styles::ICON_SIZE))
        .into();
    let zoom_out_button = tooltip(
        zoom_out_button_content,
        Text::new(zoom_out_tooltip),
        tooltip::Position::Bottom,
    )
    .gap(4);

    let zoom_in_tooltip = ctx.i18n.tr("viewer-zoom-in-tooltip");
    let zoom_in_button_content: Element<'_, Message> = button(icons::fill(icons::zoom_in()))
        .on_press(Message::ZoomIn)
        .padding(4)
        .width(Length::Fixed(shared_styles::ICON_SIZE))
        .height(Length::Fixed(shared_styles::ICON_SIZE))
        .into();
    let zoom_in_button = tooltip(
        zoom_in_button_content,
        Text::new(zoom_in_tooltip),
        tooltip::Position::Bottom,
    )
    .gap(4);

    let fit_tooltip = ctx.i18n.tr("viewer-fit-to-window-toggle");
    let fit_icon = if effective_fit_to_window {
        icons::fill(icons::compress())
    } else {
        icons::fill(icons::expand())
    };
    let fit_button = button(fit_icon)
        .on_press(Message::SetFitToWindow(!effective_fit_to_window))
        .padding(4)
        .width(Length::Fixed(shared_styles::ICON_SIZE))
        .height(Length::Fixed(shared_styles::ICON_SIZE));

    // Apply different style when fit is active (highlighted)
    let fit_button_content: Element<'_, Message> = if effective_fit_to_window {
        fit_button.style(styles::button_primary).into()
    } else {
        fit_button.into()
    };
    let fit_toggle = tooltip(
        fit_button_content,
        Text::new(fit_tooltip),
        tooltip::Position::Bottom,
    )
    .gap(4);

    let fullscreen_tooltip = ctx.i18n.tr("viewer-fullscreen-tooltip");
    let fullscreen_button_content: Element<'_, Message> = button(icons::fill(icons::fullscreen()))
        .on_press(Message::ToggleFullscreen)
        .padding(4)
        .width(Length::Fixed(shared_styles::ICON_SIZE))
        .height(Length::Fixed(shared_styles::ICON_SIZE))
        .into();
    let fullscreen_button = tooltip(
        fullscreen_button_content,
        Text::new(fullscreen_tooltip),
        tooltip::Position::Bottom,
    )
    .gap(4);

    let delete_tooltip = ctx.i18n.tr("viewer-delete-tooltip");
    let delete_button_content: Element<'_, Message> = button(icons::fill(icons::trash()))
        .on_press(Message::DeleteCurrentImage)
        .padding(4)
        .width(Length::Fixed(shared_styles::ICON_SIZE))
        .height(Length::Fixed(shared_styles::ICON_SIZE))
        .into();
    let delete_button = tooltip(
        delete_button_content,
        Text::new(delete_tooltip),
        tooltip::Position::Bottom,
    )
    .gap(4);

    let zoom_controls_row = Row::new()
        .spacing(shared_styles::CONTROL_SPACING)
        .align_y(Vertical::Center)
        .push(zoom_label)
        .push(zoom_input)
        .push(zoom_percent_label)
        .push(zoom_out_button)
        .push(zoom_in_button)
        .push(reset_button)
        .push(Space::new(Length::Fixed(16.0), Length::Shrink))
        .push(fit_toggle)
        .push(Space::new(Length::Fixed(16.0), Length::Shrink))
        .push(delete_button)
        .push(fullscreen_button);

    let mut zoom_controls = Column::new().spacing(4).push(zoom_controls_row);

    if let Some(error_key) = zoom.zoom_input_error_key {
        let error_text = Text::new(ctx.i18n.tr(error_key))
            .size(14)
            .style(|_theme: &Theme| text::Style {
                color: Some(theme::error_text_color()),
            });
        zoom_controls = zoom_controls.push(error_text);
    }

    zoom_controls.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::fluent::I18n;
    use crate::ui::state::zoom::ZoomState;

    #[test]
    fn controls_view_renders() {
        let i18n = I18n::default();
        let zoom = ZoomState::default();
        let _element = view(ViewContext { i18n: &i18n }, &zoom, true);
    }
}
