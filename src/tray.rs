use iced::futures::SinkExt;
use iced::futures::channel::mpsc::Sender;
use tray_item::{IconSource, TrayItem};

#[derive(Debug, Clone)]
pub enum TrayEvents {
    Open,
    Quit,
    Err(String),
}

pub async fn create_icon(mut tx: Sender<TrayEvents>) -> Result<(), Box<dyn std::error::Error>> {
    // Create a new tray item with the specified title and icon
    let mut tray = TrayItem::new("My Tray App", IconSource::Resource("checkmark"))?;

    let (internal_tx, mut internal_rx) = tokio::sync::mpsc::channel::<TrayEvents>(1);
    let internal_tx_clone = internal_tx.clone();

    // Add menu items to the tray
    tray.add_menu_item("Open", move || {
        internal_tx
            .blocking_send(TrayEvents::Open)
            .unwrap_or_else(|e| {
                println!("Failed to send: {e}");
            });
    })?;

    // Add a quit option
    tray.add_menu_item("Quit", move || {
        println!("Exiting application");
        internal_tx_clone
            .blocking_send(TrayEvents::Quit)
            .unwrap_or_else(|e| {
                println!("Failed to send: {e}");
            });
    })?;

    while let Some(evt) = internal_rx.recv().await {
        let should_stop = if let TrayEvents::Quit = evt {
            true
        } else {
            false
        };

        tx.send(evt).await.unwrap_or_else(|e| {
            println!("Cannot send to app: {e}");
        });

        if should_stop {
            break;
        }
    }

    Ok(())
}
