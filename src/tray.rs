#[derive(Debug, Clone)]
pub enum TrayEvents {
    Open,
    Toggle,
    Quit,
    PassSender(tokio::sync::mpsc::Sender<TrayInput>),
    Err(String),
}

#[derive(Debug, Clone)]
pub enum TrayInput {
    Started,
    Stopped,
    IconUpdate(String),
}

use iced::futures::{SinkExt, channel::mpsc::Sender};
use tray_item::{IconSource, TrayItem};

pub async fn create_icon(mut tx: Sender<TrayEvents>) -> Result<(), Box<dyn std::error::Error>> {
    // external input from the app
    let (input_tx, mut input_rx) = tokio::sync::mpsc::channel::<TrayInput>(1);

    tx.send(TrayEvents::PassSender(input_tx))
        .await
        .unwrap_or_else(|e| {
            println!("Cannot send: {e}");
        });

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

    let internal_tx_clone1 = internal_tx_clone.clone();
    tray.add_menu_item("Toggle", move || {
        internal_tx_clone1
            .blocking_send(TrayEvents::Toggle)
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

    'looping: loop {
        tokio::select! {
            Some(evt) = internal_rx.recv() => {
                let should_stop = if let TrayEvents::Quit = evt {
                    true
                } else {
                    false
                };

                tx.send(evt).await.unwrap_or_else(|e| {
                    println!("Cannot send to app: {e}");
                });

                if should_stop {
                    break 'looping;
                }
            }

            Some(evt) = input_rx.recv() => {
                process_evt(evt, &mut tray);
            }
        }
    }

    Ok(())
}

fn process_evt(evt: TrayInput, tray: &mut TrayItem) {
    let res = match evt {
        TrayInput::Started => tray.set_icon(IconSource::Resource("emblem-pause")),
        TrayInput::Stopped => tray.set_icon(IconSource::Resource("checkmark")),
        TrayInput::IconUpdate(_) => Ok(()),
    };

    if res.is_err() {
        println!("{:?}", res);
    }
}
