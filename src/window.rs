use iced::{
    Alignment, Element, Length, Theme,
    widget::{button, column, container, row, text, text_input},
};
use smart_default::SmartDefault;

use crate::app::{Context, Message};

#[derive(SmartDefault)]
pub struct Window {
    #[default(_code = "iced::window::Id::unique()")]
    pub id: iced::window::Id,
}

impl Window {
    pub fn view(&self, context: &Context) -> Element<Message> {
        let title = text("Welcome to the fishing util!")
            .size(28)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.palette().text),
            });

        let select_range_button = button("Select range").on_press(Message::GetScale);

        let scale_text = text(format!("{}", context.args.scale));

        let time_input = text_input("0.5", &context.raw_time)
            .on_input(Message::TimeInterval)
            .padding(10);

        let name_input = text_input("ebonkoi", &context.args.keyword)
            .on_input(Message::ItemName)
            .padding(10);

        // Handle button style based on state
        let action_button = match &context.handle {
            Some(_) => button("Stop").on_press(Message::Stop),
            None => button("Start").on_press(Message::Start),
        };

        // Layout with spacing and padding
        container(
            column![
                title,
                row![
                    column![select_range_button, scale_text]
                        .spacing(10)
                        .width(Length::Fill),
                    text("Interval:"),
                    time_input.width(Length::Fill),
                    text("Name:"),
                    name_input.width(Length::Fill),
                    text(context.err.clone()).style(|theme: &Theme| {
                        text::Style {
                            color: Some(theme.palette().danger),
                        }
                    }),
                ]
                .spacing(20)
                .padding(20),
                action_button,
            ]
            .spacing(20)
            .padding(20)
            .align_x(Alignment::Center),
        )
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
