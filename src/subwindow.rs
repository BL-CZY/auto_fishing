use iced::{widget::horizontal_space, Element};

use crate::app::Message;

pub struct SubWindow {}

impl SubWindow {
    pub fn view(&self, _id: iced::window::Id) -> Element<Message> {
        horizontal_space().into()
    }
}
