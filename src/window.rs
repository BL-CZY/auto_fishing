use iced::{
    Element,
    widget::{button, column, row, text, text_input},
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
        let btn = match &context.handle {
            Some(_) => button("Stop").on_press(Message::Stop),
            None => button("Start").on_press(Message::Start),
        };

        column![
            text!("Welcome to the fishing util!"),
            row![
                column![
                    button("Select range").on_press(Message::GetScale),
                    text!("{}", context.scale)
                ],
                text_input("0.5", &context.raw_time).on_input(Message::TimeInterval),
                text_input("ebonkoi", &context.name).on_input(Message::ItemName),
            ],
        ]
        .push(btn)
        .into()
    }
}
