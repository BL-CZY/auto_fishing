#[cfg(target_os = "linux")]
pub use linux::create_icon;

#[cfg(target_os = "macos")]
pub use macos::create_icon;

#[derive(Debug, Clone)]
pub enum TrayEvents {
    Open,
    Quit,
    Err(String),
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use tokio::sync::mpsc;
    use trayicon::{MenuBuilder, TrayIconBuilder};

    pub async fn create_icon(mut tx: Sender<TrayEvents>) -> Result<(), Box<dyn Error>> {
        let (internal_tx, mut internal_rx) = mpsc::channel::<TrayEvents>(1);
        let internal_tx_arc = Arc::new(Mutex::new(internal_tx.clone()));

        // Launch tray icon in a separate thread
        thread::spawn(move || {
            // Create a new menu builder
            let menu = MenuBuilder::new()
                .item("Open", move || {
                    let tx = internal_tx_arc.lock().unwrap();
                    tokio::spawn(async move {
                        tx.send(TrayEvents::Open).await.unwrap_or_else(|e| {
                            println!("Failed to send Open event: {e}");
                        });
                    });
                })
                .separator()
                .item("Quit", move || {
                    let tx = internal_tx.clone();
                    tokio::spawn(async move {
                        tx.send(TrayEvents::Quit).await.unwrap_or_else(|e| {
                            println!("Failed to send Quit event: {e}");
                        });
                    });
                });

            // Create tray icon with menu
            let tray = TrayIconBuilder::new()
                .with_menu(menu)
                .with_tooltip("My Tray App")
                .with_icon_from_resource("checkmark")
                .build()
                .expect("Failed to create tray icon");

            // Run event loop
            tray.run_event_loop();
        });

        // Forward events from the internal channel to the application channel
        while let Some(evt) = internal_rx.recv().await {
            let should_stop = matches!(evt, TrayEvents::Quit);

            tx.send(evt).await.unwrap_or_else(|e| {
                println!("Cannot send to app: {e}");
            });

            if should_stop {
                break;
            }
        }

        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::TrayEvents;

    use iced::futures::SinkExt;
    use iced::futures::channel::mpsc::Sender;
    use tray_item::{IconSource, TrayItem};

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
}
