use app::Susie;
use iced::Theme;

pub mod app;
pub mod subwindow;
pub mod window;

fn main() -> iced::Result {
    iced::daemon("fishing", Susie::update, Susie::view)
        .subscription(Susie::subscription)
        .theme(|_, _| Theme::Dark)
        .run_with(Susie::new)
}
