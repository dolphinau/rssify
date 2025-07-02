use std::error::Error;

use chrono::NaiveDate;
use regex::Regex;
use reqwest::get;
use rss::Channel;
use scraper::{Html, Selector};
use tokio::{runtime::Runtime, sync::mpsc::unbounded_channel};

fn main() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        if let Ok(articles) = fetch_paid_article_urls().await {
            for article in articles {
                if let Ok(Some(date)) = fetch_release_date(&article).await {
                    // TODO
                    println!("Snooze {} to {}", article, date);
                }
            }
        }
    });
}

async fn fetch_release_date(url: &str) -> Result<Option<NaiveDate>, Box<dyn Error>> {
    let response = get(url).await?.text().await?;

    if let Some(article_text) = Html::parse_document(&response)
        .select(&Selector::parse("div.ArticleText")?)
        .next()
    {
        if let Some(yes) = article_text.select(&Selector::parse("p")?).last() {
            let re = Regex::new(
                r#"(?m)\(Alternatively, this item will become freely\n\s* available on ([A-Z][a-z]+ [0-9]{2}, [0-9]{4})\)"#,
            )?;
            if let Some(cap) = re.captures(&yes.inner_html()) {
                if let Some(date) = cap.get(1) {
                    let date = NaiveDate::parse_from_str(date.as_str(), "%B %d, %Y")?;
                    return Ok(Some(date));
                }
            }
        }
    }

    Ok(None)
}

async fn fetch_paid_article_urls() -> Result<Vec<String>, Box<dyn Error>> {
    let response = get("https://lwn.net/headlines/rss").await?.bytes().await?;
    let channel = Channel::read_from(&response[..])?;

    Ok(channel
        .items()
        .iter()
        .filter(|i| i.title().unwrap_or("").starts_with("[$]"))
        .filter_map(|i| i.link())
        .map(|s| s.to_string())
        .collect::<Vec<String>>())
}
