use iced::futures::Stream;
use iced::stream::try_channel;
use iced::widget::horizontal_space;
use iced::{Element, Subscription, Task, window};

use crate::tray::{TrayEvents, create_icon};
use crate::window::Window;

#[derive(Default)]
pub struct Fishing {
    window: Option<Window>,
}

#[derive(Debug, Clone)]
pub enum Message {
    CreateWindow,
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    Tray(TrayEvents),
}

impl Fishing {
    pub fn new() -> (Self, Task<Message>) {
        let (_id, open) = window::open(window::Settings::default());

        (Self { window: None }, open.map(Message::WindowOpened))
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

            Message::Tray(evt) => match evt {
                TrayEvents::Open => {
                    if self.window.is_none() {
                        Task::done(Message::CreateWindow)
                    } else {
                        Task::none()
                    }
                }
                TrayEvents::Quit => iced::exit(),
                TrayEvents::Err(e) => {
                    println!("Received an error from tray: {e}");
                    Task::none()
                }
            },
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
        iced::Subscription::batch(vec![
            window_close_sub(),
            iced::Subscription::run(|| tray_events())
                .map(|val| val.map_or_else(|e| TrayEvents::Err(e), |e| e))
                .map(Message::Tray),
        ])
    }
}

fn window_close_sub() -> Subscription<Message> {
    window::close_events().map(Message::WindowClosed)
}

fn tray_events() -> impl Stream<Item = Result<TrayEvents, String>> {
    try_channel(1, move |output| async move {
        create_icon(output).await.map_err(|e| e.to_string())?;
        Ok(())
    })
}
