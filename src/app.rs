use std::sync::Arc;

use iced::futures::{SinkExt, Stream};
use iced::stream::try_channel;
use iced::widget::horizontal_space;
use iced::{Element, Subscription, Task, window};
use smart_default::SmartDefault;

use crate::fishing::{FishingArgs, FishingErr, FishingEvt, fishing_process_stream};
use crate::tray::{TrayEvents, TrayInput, create_icon};
use crate::window::Window;

#[derive(SmartDefault)]
pub struct Context {
    pub args: FishingArgs,
    #[default(false)]
    pub is_fishing: bool,
    pub err: String,

    pub handle: Option<Arc<tokio::task::JoinHandle<()>>>,

    #[default("0.5")]
    pub raw_time: String,

    pub is_capturing: bool,
    pub input_sender: Option<tokio::sync::mpsc::Sender<TrayInput>>,
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
    ScaleVal(String),
    TimeInterval(String),
    ItemName(String),

    Start,
    Stop,
    FishingEvt(FishingEvt),
    FishingErr(Arc<FishingErr>),
}

impl Fishing {
    pub fn new() -> (Self, Task<Message>) {
        let mut settings = window::Settings::default();
        settings.platform_specific.application_id = "fishing".into();
        settings.size = iced::Size::new(800.0, 600.0);
        let (_id, open) = window::open(settings);

        (Self::default(), open.map(Message::WindowOpened))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CreateWindow => {
                let mut settings = window::Settings::default();

                #[cfg(target_os = "linux")]
                {
                    settings.platform_specific.application_id = "fishing".into();
                }

                settings.size = iced::Size::new(800.0, 600.0);

                window::open(settings).1.map(Message::WindowOpened)
            }
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
                TrayEvents::Toggle => match self.context.handle {
                    Some(_) => Task::done(Message::Stop),
                    None => Task::done(Message::Start),
                },
                TrayEvents::Err(e) => {
                    println!("Received an error from tray: {e}");
                    Task::none()
                }
                TrayEvents::PassSender(tx) => {
                    self.context.input_sender = Some(tx);
                    Task::none()
                }
            },

            Message::GetScale => {
                self.context.is_capturing = true;

                Task::none()
            }

            Message::TimeInterval(str) => {
                let Ok(num) = str.parse::<f32>() else {
                    self.context.raw_time = str;
                    return Task::none();
                };

                self.context.args.time_interval = num;
                self.context.raw_time = str;

                Task::none()
            }

            Message::ItemName(name) => {
                self.context.args.keyword = name;
                Task::none()
            }

            Message::Start => {
                if !self.context.handle.is_none() {
                    return Task::none();
                }

                self.context.err = "".into();
                self.context.is_fishing = true;

                let Some(tx) = &self.context.input_sender else {
                    return Task::none();
                };

                let tx = tx.clone();

                tokio::spawn(async move {
                    tx.send(TrayInput::Started).await.unwrap_or_else(|e| {
                        println!("Cannot send: {e}");
                    });
                });

                Task::none()
            }

            Message::Stop => {
                let Some(handle) = &self.context.handle else {
                    return Task::none();
                };

                handle.abort();
                self.context.handle = None;
                self.context.is_fishing = false;

                let Some(tx) = &self.context.input_sender else {
                    return Task::none();
                };

                let tx = tx.clone();

                tokio::spawn(async move {
                    tx.send(TrayInput::Stopped).await.unwrap_or_else(|e| {
                        println!("Cannot send: {e}");
                    });
                });

                Task::none()
            }

            Message::ScaleVal(str) => {
                self.context.args.scale = str;
                self.context.is_capturing = false;
                Task::none()
            }

            Message::FishingEvt(evt) => match evt {
                FishingEvt::PassHandle(handle) => {
                    self.context.handle = Some(handle);
                    Task::none()
                }
            },
            Message::FishingErr(err) => {
                self.context.err = err.to_string();
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
            scale_capture(self.context.is_capturing),
            fishing_process(self.context.is_fishing, &self.context.args),
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

fn scale_capture(is_capturing: bool) -> Subscription<Message> {
    if !is_capturing {
        return Subscription::none();
    }

    Subscription::run(scale_capture_stream)
        .map(|val| val.map_or_else(|e| Message::ScaleVal(e), |v| v))
}

fn scale_capture_stream() -> impl Stream<Item = Result<Message, String>> {
    try_channel(1, move |mut output| async move {
        let out = match tokio::process::Command::new("slurp").output().await {
            Ok(out) => out,
            Err(e) => {
                println!("Cannot get scale: {e}");
                return Err(e.to_string());
            }
        };

        let result = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let _ = output.send(Message::ScaleVal(result)).await;

        Ok(())
    })
}

fn fishing_process(is_fishing: bool, args: &FishingArgs) -> Subscription<Message> {
    if !is_fishing {
        return Subscription::none();
    }

    Subscription::run_with_id(0, fishing_process_stream(args.clone()))
        .map(|res| res.map_or_else(|e| Message::FishingErr(e), |evt| Message::FishingEvt(evt)))
}
