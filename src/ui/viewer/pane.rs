// SPDX-License-Identifier: MPL-2.0
//! Viewer pane that renders the image inside the scrollable area with proper
//! background, cursor interaction, and position indicator.

use crate::config::BackgroundTheme;
use crate::image_handler::ImageData;
use crate::ui::viewer::component::Message;
use crate::ui::widgets::wheel_blocking_scrollable::wheel_blocking_scrollable;
use iced::widget::{mouse_area, Container, Scrollable, Stack, Text};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::canvas,
    widget::scrollable::{Direction, Id, Scrollbar, Viewport},
    Background, Border, Color, Element, Length, Padding, Theme,
};
use iced::{mouse, Rectangle};

pub struct ViewContext {
    pub background_theme: BackgroundTheme,
    pub scroll_position: Option<(f32, f32)>,
    pub scrollable_id: &'static str,
}

pub struct ViewModel<'a> {
    pub image: &'a ImageData,
    pub zoom_percent: f32,
    pub padding: Padding,
    pub is_dragging: bool,
    pub cursor_over_image: bool,
}

pub fn view<'a>(ctx: ViewContext, model: ViewModel<'a>) -> Element<'a, Message> {
    let image_viewer = super::view_image(model.image, model.zoom_percent);
    let image_container = Container::new(image_viewer).padding(model.padding);

    let scrollable = Scrollable::new(image_container)
        .id(Id::new(ctx.scrollable_id))
        .width(Length::Fill)
        .height(Length::Fill)
        .direction(Direction::Both {
            vertical: Scrollbar::new().width(0).scroller_width(0),
            horizontal: Scrollbar::new().width(0).scroller_width(0),
        })
        .on_scroll(|viewport: Viewport| {
            let bounds = viewport.bounds();
            Message::ViewportChanged {
                bounds,
                offset: viewport.absolute_offset(),
            }
        });

    let wheel_blocked_scrollable = wheel_blocking_scrollable(scrollable);

    let cursor_interaction = if model.is_dragging {
        mouse::Interaction::Grabbing
    } else if model.cursor_over_image {
        mouse::Interaction::Grab
    } else {
        mouse::Interaction::default()
    };

    let scrollable_with_cursor =
        mouse_area(wheel_blocked_scrollable).interaction(cursor_interaction);

    let scrollable_container = Container::new(scrollable_with_cursor)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center);

    let base_surface: Element<'_, Message> = match ctx.background_theme {
        BackgroundTheme::Light => {
            let color = Color::from_rgb8(245, 245, 245);
            scrollable_container
                .style(move |_theme: &Theme| iced::widget::container::Style {
                    background: Some(Background::Color(color)),
                    ..Default::default()
                })
                .into()
        }
        BackgroundTheme::Dark => {
            let color = Color::from_rgb8(32, 33, 36);
            scrollable_container
                .style(move |_theme: &Theme| iced::widget::container::Style {
                    background: Some(Background::Color(color)),
                    ..Default::default()
                })
                .into()
        }
        BackgroundTheme::Checkerboard => Stack::new()
            .push(
                canvas::Canvas::new(CheckerboardBackground)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .push(scrollable_container)
            .into(),
    };

    if let Some((px, py)) = ctx.scroll_position {
        let indicator_text = format!("Position: {:.0}% x {:.0}%", px, py);
        let indicator = Container::new(Text::new(indicator_text).size(12))
            .padding(6)
            .style(|_theme: &Theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.7))),
                text_color: Some(Color::WHITE),
                border: Border {
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.2),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            });

        Stack::new()
            .push(base_surface)
            .push(
                Container::new(indicator)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(10)
                    .align_x(Horizontal::Right)
                    .align_y(Vertical::Bottom),
            )
            .into()
    } else {
        base_surface
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct CheckerboardBackground;

impl CheckerboardBackground {
    const TILE: f32 = 20.0;
}

impl canvas::Program<Message> for CheckerboardBackground {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let tile = Self::TILE;
        let light = Color::from_rgb(0.85, 0.85, 0.85);
        let dark = Color::from_rgb(0.75, 0.75, 0.75);

        let cols = ((bounds.width / tile).ceil() as i32).max(1);
        let rows = ((bounds.height / tile).ceil() as i32).max(1);

        for row in 0..rows {
            for col in 0..cols {
                let color = if (row + col) % 2 == 0 { light } else { dark };
                let x = col as f32 * tile;
                let y = row as f32 * tile;
                let path = canvas::Path::rectangle(
                    iced::Point::new(x, y),
                    iced::Size::new(tile + 0.5, tile + 0.5),
                );
                frame.fill(&path, color);
            }
        }

        vec![frame.into_geometry()]
    }
}
