use iced::{
    Alignment, Element, Length, Theme,
    widget::{Column, button, column, container, row, scrollable},
};
use smart_default::SmartDefault;

use crate::app::Message;

fn column_style(theme: &Theme) -> iced::widget::container::Style {
    use iced::{Border, Color, Shadow, Vector, border::Radius, widget::container::Style};

    Style {
        border: Border {
            color: theme.palette().primary,
            radius: Radius::new(2.0),
            width: 2.0,
        },
        text_color: None,
        background: Some(iced::Background::Color(theme.palette().background)),
        shadow: Shadow {
            color: Color::BLACK,
            blur_radius: 10.0,
            offset: Vector::new(2.0, 2.0),
        },
    }
}

#[derive(SmartDefault)]
pub struct Window {
    #[default(_code = "iced::window::Id::unique()")]
    pub id: iced::window::Id,
    #[default(_code = "(0, 0)")]
    pub cur_ind: (usize, usize), // (the index of the extension, the index of the entry in the
                                 // extension)
}

impl Window {
    pub fn view(&self) -> Element<Message> {
        let left_col_content: Vec<Element<Message>> = vec![];

        let left = container(scrollable(
            Column::from_vec(left_col_content).width(Length::Fill),
        ))
        .width(Length::FillPortion(3))
        .height(Length::Fill)
        .padding(5)
        .style(|theme| column_style(theme));

        let right = container(scrollable(column![
            button("Yo")
                .width(Length::Fill)
                .on_press(Message::CreateWindow)
        ]))
        .width(Length::FillPortion(7))
        .height(Length::Fill)
        .padding(5)
        .style(|theme| column_style(theme));

        row![left, right]
            .width(Length::Fill)
            .spacing(10)
            .padding(10)
            .align_y(Alignment::Center)
            .into()
    }
}
