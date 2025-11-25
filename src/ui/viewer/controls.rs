// SPDX-License-Identifier: MPL-2.0
//! Viewer controls: zoom inputs, buttons, and fit-to-window toggle.

use crate::i18n::fluent::I18n;
use crate::ui::state::zoom::ZoomState;
use iced::{
    alignment::Vertical,
    widget::{button, checkbox, svg, text_input, tooltip, Column, Row, Space, Text},
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
    ToggleFullscreen,
    DeleteCurrentImage,
}

const DELETE_ICON_SVG: &str = r#"<svg viewBox='0 0 24 24' xmlns='http://www.w3.org/2000/svg' fill='none'><path d='M9 3v1H4v2h1v13a2 2 0 002 2h10a2 2 0 002-2V6h1V4h-5V3H9zm2 4h2v11h-2V7zm-4 0h2v11H7V7zm8 0h2v11h-2V7z' fill='currentColor'/></svg>"#;
const ZOOM_IN_ICON_SVG: &str = r#"<svg viewBox='0 0 24 24' xmlns='http://www.w3.org/2000/svg' fill='none'><path d='M11 4a7 7 0 015.657 11.113l4.115 4.115-1.414 1.414-4.115-4.115A7 7 0 1111 4zm0 2a5 5 0 100 10 5 5 0 000-10zm1 2v2h2v2h-2v2h-2v-2H8v-2h2V8h2z' fill='currentColor'/></svg>"#;
const ZOOM_OUT_ICON_SVG: &str = r#"<svg viewBox='0 0 24 24' xmlns='http://www.w3.org/2000/svg' fill='none'><path d='M11 4a7 7 0 015.657 11.113l4.115 4.115-1.414 1.414-4.115-4.115A7 7 0 1111 4zm0 2a5 5 0 100 10 5 5 0 000-10zm-3 4h6v2H8v-2z' fill='currentColor'/></svg>"#;
const FULLSCREEN_ICON_SVG: &str = r#"<svg viewBox='0 0 24 24' xmlns='http://www.w3.org/2000/svg' fill='none'><path d='M4 9V4h5V2H2v7h2zm15-5h-5V2h7v7h-2zm-5 15h5v-5h2v7h-7v-2zM4 15v5h5v2H2v-7h2z' fill='currentColor'/></svg>"#;
const ICON_BUTTON_SIZE: f32 = 32.0;

pub fn view<'a>(ctx: ViewContext<'a>, zoom: &'a ZoomState) -> Element<'a, Message> {
    let zoom_placeholder = ctx.i18n.tr("viewer-zoom-input-placeholder");
    let zoom_label = Text::new(ctx.i18n.tr("viewer-zoom-label"));

    let zoom_input = text_input(&zoom_placeholder, &zoom.zoom_input)
        .on_input(Message::ZoomInputChanged)
        .on_submit(Message::ZoomInputSubmitted)
        .padding(6)
        .size(16)
        .width(Length::Fixed(90.0));

    let reset_button = button(Text::new(ctx.i18n.tr("viewer-zoom-reset-button")))
        .on_press(Message::ResetZoom)
        .padding([6, 12]);

    let zoom_out_tooltip = ctx.i18n.tr("viewer-zoom-out-tooltip");
    let zoom_out_icon = svg::Svg::new(svg::Handle::from_memory(
        ZOOM_OUT_ICON_SVG.as_bytes().to_vec(),
    ))
    .width(Length::Fill)
    .height(Length::Fill);
    let zoom_out_button_content: Element<'_, Message> = button(zoom_out_icon)
        .on_press(Message::ZoomOut)
        .padding(4)
        .width(Length::Fixed(ICON_BUTTON_SIZE))
        .height(Length::Fixed(ICON_BUTTON_SIZE))
        .into();
    let zoom_out_button = tooltip(
        zoom_out_button_content,
        Text::new(zoom_out_tooltip),
        tooltip::Position::Bottom,
    )
    .gap(4);

    let zoom_in_tooltip = ctx.i18n.tr("viewer-zoom-in-tooltip");
    let zoom_in_icon = svg::Svg::new(svg::Handle::from_memory(
        ZOOM_IN_ICON_SVG.as_bytes().to_vec(),
    ))
    .width(Length::Fill)
    .height(Length::Fill);
    let zoom_in_button_content: Element<'_, Message> = button(zoom_in_icon)
        .on_press(Message::ZoomIn)
        .padding(4)
        .width(Length::Fixed(ICON_BUTTON_SIZE))
        .height(Length::Fixed(ICON_BUTTON_SIZE))
        .into();
    let zoom_in_button = tooltip(
        zoom_in_button_content,
        Text::new(zoom_in_tooltip),
        tooltip::Position::Bottom,
    )
    .gap(4);

    let fit_toggle = checkbox(
        ctx.i18n.tr("viewer-fit-to-window-toggle"),
        zoom.fit_to_window,
    )
    .on_toggle(Message::SetFitToWindow)
    .text_wrapping(iced::widget::text::Wrapping::Word);

    let fullscreen_tooltip = ctx.i18n.tr("viewer-fullscreen-tooltip");
    let fullscreen_icon = svg::Svg::new(svg::Handle::from_memory(
        FULLSCREEN_ICON_SVG.as_bytes().to_vec(),
    ))
    .width(Length::Fill)
    .height(Length::Fill);
    let fullscreen_button_content: Element<'_, Message> = button(fullscreen_icon)
        .on_press(Message::ToggleFullscreen)
        .padding(4)
        .width(Length::Fixed(ICON_BUTTON_SIZE))
        .height(Length::Fixed(ICON_BUTTON_SIZE))
        .into();
    let fullscreen_button = tooltip(
        fullscreen_button_content,
        Text::new(fullscreen_tooltip),
        tooltip::Position::Bottom,
    )
    .gap(4);

    let delete_tooltip = ctx.i18n.tr("viewer-delete-tooltip");
    let delete_icon = svg::Svg::new(svg::Handle::from_memory(
        DELETE_ICON_SVG.as_bytes().to_vec(),
    ))
    .width(Length::Fill)
    .height(Length::Fill);
    let delete_button_content: Element<'_, Message> = button(delete_icon)
        .on_press(Message::DeleteCurrentImage)
        .padding(4)
        .width(Length::Fixed(ICON_BUTTON_SIZE))
        .height(Length::Fixed(ICON_BUTTON_SIZE))
        .into();
    let delete_button = tooltip(
        delete_button_content,
        Text::new(delete_tooltip),
        tooltip::Position::Bottom,
    )
    .gap(4);

    let zoom_controls_row = Row::new()
        .spacing(10)
        .align_y(Vertical::Center)
        .push(zoom_label)
        .push(zoom_input)
        .push(zoom_out_button)
        .push(reset_button)
        .push(zoom_in_button)
        .push(Space::new(Length::Fixed(16.0), Length::Shrink))
        .push(fit_toggle)
        .push(delete_button)
        .push(fullscreen_button);

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
