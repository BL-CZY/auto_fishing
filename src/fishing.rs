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
            let mut tx_clone0 = tx_clone.clone();
            let Err(e) = start_fishing(args, tx_clone).await;

            tx_clone0
                .send(FishingEvt::Err(Arc::new(e)))
                .await
                .unwrap_or_else(|e| {
                    println!("Canot send error: {e}");
                });
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
    Err(Arc<FishingErr>),
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
    #[error("String: {0}")]
    String(String),
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
    #[default(1.0)]
    pub time_interval: f32,
    #[default("Ebonkoi")]
    pub keyword: String,
    pub indicator_tx: Option<tokio::sync::mpsc::Sender<(i32, i32, i32, i32)>>,
}

pub async fn start_fishing(
    FishingArgs {
        scale,
        time_interval,
        keyword,
        indicator_tx,
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

    // let (x, y, w, h) = parse_coordinates(&scale).map_err(|e| FishingErr::String(e.to_string()))?;
    // indicator_tx
    //     .clone()
    //     .unwrap()
    //     .send((w, h, x, y))
    //     .await
    //     .unwrap_or_else(|e| {
    //         println!("Cannot send indicator: {e}");
    //     });

    let keywords: Vec<&str> = keyword.split(",").collect();

    loop {
        tokio::process::Command::new("grim")
            .arg("-g")
            .arg(&format!("{}", &scale))
            .arg(&path)
            .output()
            .await?;

        println!("grim done");

        println!("Spawned indicator");

        let ocr = tesseract::Tesseract::new(None, Some("eng"))?;
        let mut ocr = ocr.set_image(&path.to_str().expect("No image"))?;
        let text = ocr.get_text()?;

        println!("OCR: {}", text);

        let mut contains = false;

        for kwd in keywords.iter() {
            if text.contains(kwd) {
                contains = true;
                break;
            }
        }

        if contains {
            click().await;

            tokio::time::sleep(tokio::time::Duration::from_secs_f64(1.0)).await;
            println!("click");

            click().await;

            println!("YOOO");
        }

        tokio::time::sleep(tokio::time::Duration::from_secs_f32(time_interval)).await;

        println!("slept");
    }
}

fn parse_coordinates(input: &str) -> Result<(i32, i32, i32, i32), Box<dyn std::error::Error>> {
    // Split by space to separate the two parts
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() != 2 {
        return Err("Input should have two parts separated by space".into());
    }

    // Parse the first part which is separated by comma
    let first_parts: Vec<&str> = parts[0].split(',').collect();
    if first_parts.len() != 2 {
        return Err("First part should contain two numbers separated by comma".into());
    }

    // Parse the second part which is separated by 'x'
    let second_parts: Vec<&str> = parts[1].split('x').collect();
    if second_parts.len() != 2 {
        return Err("Second part should contain two numbers separated by 'x'".into());
    }

    // Parse all four values as i32
    let a = first_parts[0].parse::<i32>()?;
    let b = first_parts[1].parse::<i32>()?;
    let c = second_parts[0].parse::<i32>()?;
    let d = second_parts[1].parse::<i32>()?;

    Ok((a, b, c, d))
}
