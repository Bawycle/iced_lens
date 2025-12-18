// SPDX-License-Identifier: MPL-2.0
//! Viewer controls: zoom inputs, buttons, and fit-to-window toggle.

use crate::i18n::fluent::I18n;
use crate::ui::design_tokens::{spacing, typography};
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
    /// Whether metadata editor has unsaved changes (disables fullscreen).
    pub metadata_editor_has_changes: bool,
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
    is_fullscreen: bool,
) -> Element<'a, Message> {
    let zoom_placeholder = ctx.i18n.tr("viewer-zoom-input-placeholder");
    let zoom_label = Text::new(ctx.i18n.tr("viewer-zoom-label"));

    let zoom_input = text_input(&zoom_placeholder, &zoom.zoom_input)
        .on_input(Message::ZoomInputChanged)
        .on_submit(Message::ZoomInputSubmitted)
        .padding(spacing::XXS)
        .size(typography::BODY_LG)
        .width(Length::Fixed(60.0));

    let zoom_percent_label = Text::new("%").size(typography::BODY_LG);

    let reset_tooltip = ctx.i18n.tr("viewer-zoom-reset-button");
    let reset_button_content: Element<'_, Message> = button(icons::fill(icons::refresh()))
        .on_press(Message::ResetZoom)
        .padding(spacing::XXS)
        .width(Length::Fixed(shared_styles::ICON_SIZE))
        .height(Length::Fixed(shared_styles::ICON_SIZE))
        .into();
    let reset_button = tooltip(
        reset_button_content,
        Text::new(reset_tooltip),
        tooltip::Position::Bottom,
    )
    .gap(spacing::XXS);

    let zoom_out_tooltip = ctx.i18n.tr("viewer-zoom-out-tooltip");
    let zoom_out_button_content: Element<'_, Message> = button(icons::fill(icons::zoom_out()))
        .on_press(Message::ZoomOut)
        .padding(spacing::XXS)
        .width(Length::Fixed(shared_styles::ICON_SIZE))
        .height(Length::Fixed(shared_styles::ICON_SIZE))
        .into();
    let zoom_out_button = tooltip(
        zoom_out_button_content,
        Text::new(zoom_out_tooltip),
        tooltip::Position::Bottom,
    )
    .gap(spacing::XXS);

    let zoom_in_tooltip = ctx.i18n.tr("viewer-zoom-in-tooltip");
    let zoom_in_button_content: Element<'_, Message> = button(icons::fill(icons::zoom_in()))
        .on_press(Message::ZoomIn)
        .padding(spacing::XXS)
        .width(Length::Fixed(shared_styles::ICON_SIZE))
        .height(Length::Fixed(shared_styles::ICON_SIZE))
        .into();
    let zoom_in_button = tooltip(
        zoom_in_button_content,
        Text::new(zoom_in_tooltip),
        tooltip::Position::Bottom,
    )
    .gap(spacing::XXS);

    let fit_tooltip = ctx.i18n.tr("viewer-fit-to-window-toggle");
    let fit_icon = if effective_fit_to_window {
        icons::fill(icons::compress())
    } else {
        icons::fill(icons::expand())
    };
    let fit_button = button(fit_icon)
        .on_press(Message::SetFitToWindow(!effective_fit_to_window))
        .padding(spacing::XXS)
        .width(Length::Fixed(shared_styles::ICON_SIZE))
        .height(Length::Fixed(shared_styles::ICON_SIZE));

    // Apply different style when fit is active (highlighted)
    let fit_button_content: Element<'_, Message> = if effective_fit_to_window {
        fit_button.style(styles::button::selected).into()
    } else {
        fit_button.into()
    };
    let fit_toggle = tooltip(
        fit_button_content,
        Text::new(fit_tooltip),
        tooltip::Position::Bottom,
    )
    .gap(spacing::XXS);

    // Fullscreen button - disabled when metadata editor has unsaved changes
    let fullscreen_button = button(icons::fill(icons::fullscreen()))
        .padding(spacing::XXS)
        .width(Length::Fixed(shared_styles::ICON_SIZE))
        .height(Length::Fixed(shared_styles::ICON_SIZE));

    let (fullscreen_button_content, fullscreen_tooltip): (Element<'_, Message>, String) =
        if ctx.metadata_editor_has_changes {
            // Disabled: cannot toggle fullscreen with unsaved metadata changes
            (
                fullscreen_button.style(styles::button::disabled()).into(),
                ctx.i18n.tr("viewer-fullscreen-disabled-unsaved"),
            )
        } else if is_fullscreen {
            // Highlighted: fullscreen is active
            (
                fullscreen_button
                    .on_press(Message::ToggleFullscreen)
                    .style(styles::button::selected)
                    .into(),
                ctx.i18n.tr("viewer-fullscreen-tooltip"),
            )
        } else {
            // Normal: can toggle fullscreen
            (
                fullscreen_button
                    .on_press(Message::ToggleFullscreen)
                    .into(),
                ctx.i18n.tr("viewer-fullscreen-tooltip"),
            )
        };
    let fullscreen_toggle = tooltip(
        fullscreen_button_content,
        Text::new(fullscreen_tooltip),
        tooltip::Position::Bottom,
    )
    .gap(spacing::XXS);

    let delete_tooltip = ctx.i18n.tr("viewer-delete-tooltip");
    let delete_button_content: Element<'_, Message> = button(icons::fill(icons::trash()))
        .on_press(Message::DeleteCurrentImage)
        .padding(spacing::XXS)
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
        .padding([0.0, shared_styles::CONTROL_PADDING])
        .align_y(Vertical::Center)
        .push(zoom_label)
        .push(zoom_input)
        .push(zoom_percent_label)
        .push(zoom_out_button)
        .push(zoom_in_button)
        .push(reset_button)
        .push(Space::new().width(Length::Fixed(shared_styles::CONTROL_PADDING)))
        .push(fit_toggle)
        .push(Space::new().width(Length::Fixed(shared_styles::CONTROL_PADDING)))
        .push(delete_button)
        .push(fullscreen_toggle);

    let mut zoom_controls = Column::new().spacing(spacing::XXS).push(zoom_controls_row);

    if let Some(error_key) = zoom.zoom_input_error_key {
        let error_text = Text::new(ctx.i18n.tr(error_key))
            .size(typography::BODY)
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
        let _element = view(
            ViewContext {
                i18n: &i18n,
                metadata_editor_has_changes: false,
            },
            &zoom,
            true,
            false,
        );
    }
}
