use iced::widget::horizontal_space;
use iced::{Element, Subscription, Task, window};

use crate::window::Window;

#[derive(Default)]
pub struct Susie {
    window: Option<Window>,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    CreateWindow,
    WindowOpened(window::Id),
    WindowClosed(window::Id),
}

impl Susie {
    pub fn new() -> (Self, Task<Message>) {
        let (_id, open) = window::open(window::Settings::default());

        (Self::default(), open.map(Message::WindowOpened))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CreateWindow => window::open(window::Settings::default())
                .1
                .map(Message::WindowOpened),
            Message::WindowOpened(id) => {
                self.window = Some(Window { id });
                Task::none()
            }

            Message::WindowClosed(_) => {
                self.window = None;

                Task::none()
            }
        }
    }

    pub fn view(&self, _window_id: window::Id) -> Element<Message> {
        if let Some(window) = &self.window {
            window.view().into()
        } else {
            horizontal_space().into()
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        window::close_events().map(Message::WindowClosed)
    }
}
