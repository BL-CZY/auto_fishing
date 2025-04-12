use std::{convert::Infallible, path::PathBuf, sync::Arc};

use iced::{
    futures::{SinkExt, Stream},
    stream::try_channel,
};
use smart_default::SmartDefault;
use tesseract::{InitializeError, SetImageError, plumbing::TessBaseApiGetUtf8TextError};
use thiserror::Error;
use tokio::task::JoinHandle;

pub fn fishing_process_stream(
    args: FishingArgs,
) -> impl Stream<Item = Result<FishingEvt, Arc<FishingErr>>> {
    try_channel(1, move |mut tx| async move {
        let tx_clone = tx.clone();

        let handle = tokio::spawn(async move {
            let tx_clone0 = tx_clone.clone();
            let _ = start_fishing(args, tx_clone0).await?;
        });

        tx.send(FishingEvt::PassHandle(Arc::new(handle)))
            .await
            .unwrap_or_else(|e| {
                println!("Cannot send handle: {e}");
            });

        Ok(())
    })
}

#[derive(Debug, Clone)]
pub enum FishingEvt {
    PassHandle(Arc<JoinHandle<()>>),
    CountDown(i32),
}

#[derive(Debug, Error)]
pub enum FishingErr {
    #[error("IO Error: {0}")]
    IoErr(#[from] std::io::Error),
    #[error("OCR Init Error: {0}")]
    InitErr(#[from] InitializeError),
    #[error("OCR Imagee Error: {0}")]
    ImgErr(#[from] SetImageError),
    #[error("OCR Error: {0}")]
    OCRErr(#[from] TessBaseApiGetUtf8TextError),
}

async fn click() {
    let res = tokio::process::Command::new("ydotool")
        .arg("click")
        .arg("0xC0")
        .output()
        .await;

    if let Err(e) = res {
        println!("Cannot click: {e}");
    }
}

#[derive(Debug, Clone, SmartDefault)]
pub struct FishingArgs {
    #[default("--")]
    pub scale: String,
    #[default(0.5)]
    pub time_interval: f32,
    #[default("Ebonkoi")]
    pub keyword: String,
}

pub async fn start_fishing(
    FishingArgs {
        scale,
        time_interval,
        keyword,
    }: FishingArgs,
    mut tx: iced::futures::channel::mpsc::Sender<FishingEvt>,
) -> Result<Infallible, FishingErr> {
    let home = std::env::var("HOME").expect("No home");
    let mut path = PathBuf::from(home);
    path.push(".cache/auto_fishing/");

    let _ = tokio::fs::create_dir_all(&path).await;

    path.push("screenshot.png");

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    tx.send(FishingEvt::CountDown(2)).await.unwrap_or_else(|e| {
        println!("Cannot send fishing event: {e}");
    });

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    tx.send(FishingEvt::CountDown(1)).await.unwrap_or_else(|e| {
        println!("Cannot send fishing event: {e}");
    });

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    tx.send(FishingEvt::CountDown(0)).await.unwrap_or_else(|e| {
        println!("Cannot send fishing event: {e}");
    });

    loop {
        tokio::process::Command::new("grim")
            .arg("-g")
            .arg(&format!("{}", &scale))
            .arg(&path)
            .output()
            .await?;

        let ocr = tesseract::Tesseract::new(None, Some("eng"))?;
        let mut ocr = ocr.set_image(&path.to_str().expect("No image"))?;
        let text = ocr.get_text()?;

        if text.contains(&keyword) {
            click().await;

            tokio::time::sleep(tokio::time::Duration::from_secs_f64(0.5)).await;
            println!("click");

            click().await;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs_f32(time_interval)).await;
    }
}
