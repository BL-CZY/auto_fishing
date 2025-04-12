use app::Fishing;
use iced::Theme;

pub mod app;
pub mod fishing;
pub mod indicator;
pub mod tray;
pub mod window;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = tokio::sync::mpsc::channel::<(i32, i32, i32, i32)>(1);

    // std::thread::Builder::new()
    //     .name("auto_fishing_gtk".into())
    //     .spawn(|| {
    //         let rt = tokio::runtime::Builder::new_multi_thread()
    //             .enable_time()
    //             .build()
    //             .expect("Cannot build runtime");
    //
    //         rt.block_on(async move {
    //             indicator::start_gtk(rx);
    //         });
    //     })
    //     .expect("Failed to spawn gtk");

    iced::daemon("fishing", Fishing::update, Fishing::view)
        .subscription(Fishing::subscription)
        .theme(|_, _| Theme::Dark)
        .run_with(|| Fishing::new(tx))?;

    Ok(())
}
