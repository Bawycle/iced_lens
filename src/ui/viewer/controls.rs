// SPDX-License-Identifier: MPL-2.0
//! Viewer controls: zoom inputs, buttons, and fit-to-window toggle.

use crate::i18n::fluent::I18n;
use crate::ui::state::zoom::ZoomState;
use iced::{
    alignment::Vertical,
    widget::{button, checkbox, text_input, Column, Row, Space, Text},
    Element, Length,
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
}

pub fn view<'a>(ctx: ViewContext<'a>, zoom: &'a ZoomState) -> Element<'a, Message> {
    let zoom_placeholder = ctx.i18n.tr("viewer-zoom-input-placeholder");
    let zoom_label = Text::new(ctx.i18n.tr("viewer-zoom-label"));

    let zoom_input = text_input(&zoom_placeholder, &zoom.zoom_input)
        .on_input(Message::ZoomInputChanged)
        .on_submit(Message::ZoomInputSubmitted)
        .padding(6)
        .size(16)
        .width(Length::Fixed(90.0));

    let zoom_out_button = button(Text::new(ctx.i18n.tr("viewer-zoom-out-button")))
        .on_press(Message::ZoomOut)
        .padding([6, 12]);

    let reset_button = button(Text::new(ctx.i18n.tr("viewer-zoom-reset-button")))
        .on_press(Message::ResetZoom)
        .padding([6, 12]);

    let zoom_in_button = button(Text::new(ctx.i18n.tr("viewer-zoom-in-button")))
        .on_press(Message::ZoomIn)
        .padding([6, 12]);

    let fit_toggle = checkbox(
        ctx.i18n.tr("viewer-fit-to-window-toggle"),
        zoom.fit_to_window,
    )
    .on_toggle(Message::SetFitToWindow)
    .text_wrapping(iced::widget::text::Wrapping::Word);

    let zoom_controls_row = Row::new()
        .spacing(10)
        .align_y(Vertical::Center)
        .push(zoom_label)
        .push(zoom_input)
        .push(zoom_out_button)
        .push(reset_button)
        .push(zoom_in_button)
        .push(Space::new(Length::Fixed(16.0), Length::Shrink))
        .push(fit_toggle);

    let mut zoom_controls = Column::new().spacing(4).push(zoom_controls_row);

    if let Some(error_key) = zoom.zoom_input_error_key {
        let error_text = Text::new(ctx.i18n.tr(error_key)).size(14);
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
        let _element = view(ViewContext { i18n: &i18n }, &zoom);
    }
}
