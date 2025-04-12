use std::{convert::Infallible, path::PathBuf};

use iced::{futures::Stream, stream::try_channel};
use smart_default::SmartDefault;
use tesseract::{InitializeError, SetImageError, plumbing::TessBaseApiGetUtf8TextError};
use thiserror::Error;

pub fn fishing_process_stream(
    args: FishingArgs,
) -> impl Stream<Item = Result<FishingEvt, FishingErr>> {
    try_channel(1, move |tx| async move { Ok(()) })
}

#[derive(Debug, Clone)]
pub enum FishingEvt {}

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
) -> Result<Infallible, FishingErr> {
    let home = std::env::var("HOME").expect("No home");
    let mut path = PathBuf::from(home);
    path.push(".cache/auto_fishing/");

    let _ = tokio::fs::create_dir_all(&path).await;

    path.push("screenshot.png");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

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
