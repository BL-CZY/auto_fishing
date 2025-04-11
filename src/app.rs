use iced::futures::Stream;
use iced::stream::try_channel;
use iced::widget::horizontal_space;
use iced::{Element, Subscription, Task, window};
use smart_default::SmartDefault;

use crate::fishing::start_fishing;
use crate::tray::{TrayEvents, create_icon};
use crate::window::Window;

#[derive(SmartDefault)]
pub struct Context {
    #[default("--")]
    pub scale: String,
    #[default(0.5)]
    pub interval: f32,
    #[default("Ebonkoi")]
    pub name: String,
    pub handle: Option<tokio::task::JoinHandle<()>>,
    #[default("0.5")]
    pub raw_time: String,
}

#[derive(Default)]
pub struct Fishing {
    window: Option<Window>,
    context: Context,
}

#[derive(Debug, Clone)]
pub enum Message {
    CreateWindow,
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    Tray(TrayEvents),
    GetScale,
    TimeInterval(String),
    ItemName(String),
    Start,
    Stop,
}

impl Fishing {
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

            Message::GetScale => {
                let out = match std::process::Command::new("slurp").output() {
                    Ok(out) => out,
                    Err(e) => {
                        self.context.scale = e.to_string();
                        return Task::none();
                    }
                };

                let result = String::from_utf8_lossy(&out.stdout).trim().to_string();
                self.context.scale = result;

                Task::none()
            }

            Message::TimeInterval(str) => {
                let Ok(num) = str.parse::<f32>() else {
                    self.context.raw_time = str;
                    return Task::none();
                };

                self.context.interval = num;
                self.context.raw_time = str;

                Task::none()
            }

            Message::ItemName(name) => {
                self.context.name = name;
                Task::none()
            }

            Message::Start => {
                if !self.context.handle.is_none() {
                    return Task::none();
                }

                let scale = self.context.scale.clone();
                let time_interval = self.context.interval.clone();
                let keyword = self.context.name.clone();

                let handle = tokio::spawn(async move {
                    let res = start_fishing(scale, time_interval, keyword).await;
                    println!("{:?}", res);
                });
                self.context.handle = Some(handle);
                Task::none()
            }

            Message::Stop => {
                let Some(handle) = &self.context.handle else {
                    return Task::none();
                };

                handle.abort();
                self.context.handle = None;

                Task::none()
            }
        }
    }

    pub fn view(&self, _window_id: window::Id) -> Element<Message> {
        if let Some(window) = &self.window {
            window.view(&self.context).into()
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
