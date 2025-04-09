use app::Fishing;
use iced::Theme;

pub mod app;
pub mod tray;
pub mod window;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    iced::daemon("fishing", Fishing::update, Fishing::view)
        .subscription(Fishing::subscription)
        .theme(|_, _| Theme::Dark)
        .run_with(Fishing::new)?;

    Ok(())
}
