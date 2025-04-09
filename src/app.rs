use std::collections::BTreeMap;

use iced::widget::horizontal_space;
use iced::{Element, Subscription, Task, Vector, window};

use crate::subwindow::SubWindow;
use crate::window::Window;

#[derive(Default)]
pub struct Susie {
    main_window: Window,
    windows: BTreeMap<window::Id, SubWindow>,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    CreateWindow,
    WindowOpened(window::Id),
    WindowClosed(window::Id),
}

impl Susie {
    pub fn new() -> (Self, Task<Message>) {
        let (id, open) = window::open(window::Settings::default());

        (
            Self {
                main_window: Window {
                    id,
                    ..Default::default()
                },
                windows: BTreeMap::new(),
            },
            open.map(Message::WindowOpened),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CreateWindow => {
                let last_window = if let Some(l) = self.windows.keys().last() {
                    l
                } else {
                    &self.main_window.id
                };

                window::get_position(*last_window)
                    .then(|last_position| {
                        let position =
                            last_position.map_or(window::Position::Default, |last_position| {
                                window::Position::Specific(last_position + Vector::new(20.0, 20.0))
                            });

                        let (_id, open) = window::open(window::Settings {
                            position,
                            ..window::Settings::default()
                        });

                        open
                    })
                    .map(Message::WindowOpened)
            }
            Message::WindowOpened(id) => {
                if id == self.main_window.id {
                    return Task::none();
                }

                let window = SubWindow {};

                self.windows.insert(id, window);
                Task::none()
            }

            Message::WindowClosed(id) => {
                if id == self.main_window.id {
                    return iced::exit();
                }

                self.windows.remove(&id);

                Task::none()
            }
        }
    }

    pub fn view(&self, window_id: window::Id) -> Element<Message> {
        if window_id == self.main_window.id {
            return self.main_window.view().into();
        }

        if let Some(window) = self.windows.get(&window_id) {
            window.view(window_id).into()
        } else {
            horizontal_space().into()
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        window::close_events().map(Message::WindowClosed)
    }
}
