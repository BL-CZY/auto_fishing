use std::{convert::Infallible, path::PathBuf};

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

pub async fn start_fishing(
    scale: String,
    time_interval: f32,
    keyword: String,
) -> Result<Infallible, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME").expect("No home");
    let mut path = PathBuf::from(home);
    path.push(".cache/auto_fishing/");

    let _ = tokio::fs::create_dir_all(&path).await;

    path.push("screenshot.png");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    loop {
        let res = tokio::process::Command::new("grim")
            .arg("-g")
            .arg(&format!("{}", &scale))
            .arg(&path)
            .output()
            .await;

        if let Err(e) = res {
            println!("{e}");
            return Err(Box::new(e));
        }

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
