use std::{
    fs::{create_dir_all, File},
    io::copy,
    path::Path,
};

use reqwest::Client;
use scraper::{Html, Selector};

use chrono::{Datelike, Utc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url_prefix = "https://podcast.sprovoost.nl/@nado/episodes?year=";
    let mut year = Utc::now().year();

    loop {
        let url = format!("{url_prefix}{year}");
        let resp = reqwest::get(&url).await?;
        let text = resp.text().await?;

        let document = Html::parse_document(&text);
        let ep_selector =
            Selector::parse(r#"body > div > main > div > article > div > play-episode-button"#)
                .unwrap();

        let mut order = document.select(&ep_selector).count();

        if order == 0 {
            break;
        }

        create_dir_all(year.to_string().as_str()).unwrap();

        for name in document.select(&ep_selector) {
            let ep_url = name.value().attr("src").expect("src not found").to_string();
            let ep_url = ep_url.strip_suffix("?_from=-+Website+-").unwrap();

            let ep_name: Vec<&str> = ep_url.split('/').collect();
            let ep_name = ep_name.last().unwrap();

            let ep_file_path = format!("{}/{:02}-{}", year, order, ep_name);
            order -= 1;

            if Path::new(&ep_file_path).exists() {
                let ep_file = File::open(&ep_file_path)?;
                if ep_file.metadata()?.len() != 0 {
                    println!("Episode file {ep_file_path} already downloaded. Skipping...");
                    continue;
                }
            }

            let mut ep_file = File::create(&ep_file_path)?;

            print!("Episode Name: {ep_name}, URL: {ep_url}. ");

            let client = Client::builder().cookie_store(true).build().unwrap();

            let req = client
                .get(ep_url)
                .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36")
                .send()
                .await?;

            let episode = req.bytes().await?;

            print!("Saving to {ep_file_path}...");

            copy(&mut episode.as_ref(), &mut ep_file)?;

            println!("Done");
        }
        year -= 1;
    }

    Ok(())
}
