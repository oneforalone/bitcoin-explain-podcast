use std::{
    fs::{create_dir_all, File},
    io::copy,
    path::Path,
    thread,
    time::Duration,
};

use thread::sleep;

use reqwest::Client;
use scraper::{Html, Selector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url_prefix = "https://podcast.sprovoost.nl/@nado/episodes?year=";
    let years = vec!["2020", "2021", "2022", "2023"];
    for year in years.into_iter() {
        create_dir_all(year).unwrap();

        let url = format!("{url_prefix}{year}");
        let resp = reqwest::get(&url).await?;
        let text = resp.text().await?;

        sleep(Duration::from_secs(1));

        let document = Html::parse_document(&text);
        let ep_selector =
            Selector::parse(r#"body > div > main > div > article > div > play-episode-button"#)
                .unwrap();
        // the file format does not unified, so i just simplify it.
        let mut order = 30;
        for name in document.select(&ep_selector) {
            let ep_url = name.value().attr("src").expect("src not found").to_string();
            let ep_url = ep_url.strip_suffix("?_from=-+Website+-").unwrap();

            let ep_name: Vec<&str> = ep_url.split('/').collect();
            let ep_name = ep_name.last().unwrap();

            let ep_file_path = format!("{}/{:02}-{}", year, order, ep_name);
            order = order - 1;

            let mut ep_file = File::create(&ep_file_path)?;
            let f_len = ep_file.metadata().unwrap().len();

            if Path::new(&ep_file_path).exists() && (f_len != 0) {
                println!("Episode file {ep_file_path} already exists.");
                continue;
            }

            println!("Episode Name: {ep_name}, URL: {ep_url}, Local Path: {ep_file_path}");

            let client = Client::builder().cookie_store(true).build().unwrap();

            let req = client
                .get(ep_url)
                .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36")
                .send()
                .await?;

            println!("Getting {ep_url}: {}", req.status());

            sleep(Duration::from_secs(1));

            let episode = req.bytes().await?;

            print!("Saving {ep_name} to {ep_file_path}...");

            copy(&mut episode.as_ref(), &mut ep_file)?;

            println!("Done");
        }
    }

    Ok(())
}
