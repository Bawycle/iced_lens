// SPDX-License-Identifier: MPL-2.0
//! Editor view composition helpers.

pub mod canvas;
pub mod sidebar;
pub mod toolbar;

use crate::ui::theme;
use iced::widget::{button, container, Column, Row, Text};
use iced::{Background, Border, Element, Length};

use super::{Message, State, ViewContext};
use sidebar::SidebarModel;

pub fn render<'a>(state: &'a State, ctx: ViewContext<'a>) -> Element<'a, Message> {
    let toolbar = toolbar::view(state, &ctx);

    let mut main_row = Row::new().spacing(0);

    if state.sidebar_expanded {
        let sidebar_model = SidebarModel::from_state(state);
        let sidebar = sidebar::expanded(sidebar_model, &ctx);
        main_row = main_row.push(sidebar);
    } else {
        main_row = main_row.push(collapsed_sidebar());
    }

    let canvas = canvas::view(state, &ctx);
    main_row = main_row.push(canvas);

    let content = Column::new().push(toolbar).push(main_row);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn collapsed_sidebar<'a>() -> Element<'a, Message> {
    let toggle_button = button(Text::new("☰").size(24))
        .on_press(Message::ToggleSidebar)
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
