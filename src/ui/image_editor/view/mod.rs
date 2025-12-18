// SPDX-License-Identifier: MPL-2.0
//! Editor view composition helpers.

pub mod canvas;
pub mod centered_scrollable;
pub mod scrollable_canvas;
pub mod sidebar;
pub mod toolbar;

use iced::widget::{container, Column, Row};
use iced::{Element, Length};

use super::{Message, State, ViewContext};
use canvas::CanvasModel;
use sidebar::SidebarModel;
use toolbar::ToolbarModel;

pub fn render<'a>(state: &'a State, ctx: ViewContext<'a>) -> Element<'a, Message> {
    let toolbar_model = ToolbarModel::from_state(state);
    let toolbar = toolbar::view(&toolbar_model, &ctx);

    let mut main_row = Row::new().spacing(0.0);

    if state.sidebar_expanded {
        let sidebar_model = SidebarModel::from_state(state, &ctx);
        let sidebar = sidebar::expanded(sidebar_model, &ctx);
        main_row = main_row.push(sidebar);
    } else {
        main_row = main_row.push(sidebar::collapsed(ctx.is_dark_theme));
    }

    let canvas_model = CanvasModel::from_state(state);
    let canvas = canvas::view(canvas_model, &ctx);
    main_row = main_row.push(canvas);

    let content = Column::new().push(toolbar).push(main_row);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
