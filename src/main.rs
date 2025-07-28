use std::error::Error;

use chrono::NaiveDate;
use regex::Regex;
use reqwest::get;
use scraper::{Html, Selector};
use tokio::{
    runtime::Runtime,
    sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
};
use tokio_postgres;

fn main() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        // Connect to the database.
        if let Ok((client, connection)) = tokio_postgres::connect(
            "host=localhost dbname=dev user=root password=root",
            tokio_postgres::NoTls,
        )
        .await
        {
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("connection error: {}", e);
                }
            });

            // Get new [$] articles
            if let Ok(items) = fetch_paid_article_urls().await {
                for item in items {
                    if let Some(link) = item.link() {
                        match client
                            .query_opt("SELECT date FROM articles WHERE id = $1", &[&link])
                            .await
                        {
                            Ok(None) => {
                                if let Ok(Some(date)) = fetch_release_date(&link).await {
                                    println!("Adding new article to db: {}", link);
                                    if let Err(e) = client
                                        .query(
                                            "INSERT INTO articles (id, date) VALUES ($1, $2)",
                                            &[&link, &date.to_string()],
                                        )
                                        .await
                                    {
                                        eprintln!("Error insert: {}", e);
                                    }
                                }
                            }
                            _ => (),
                        }
                    };
                }
            }

            // TODO: Check for new free articles
            // client
            //     .query("SELECT * FROM articles")
            //     .await
            //     .unwrap()
            //     .iter()
            //     .map(|row| {
            //         let id = row.get("id");
            //         let date = row.get("date");
            //
            //          if date < today {
            //              article.publish
            //          }
            //     })
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
                r#"(?m)\(Alternatively, this item will become freely\s*available on ([A-Z][a-z]+ [0-9]{1,2}, [0-9]{4})\)"#,
            )?;
            if let Some(cap) = re.captures(&yes.inner_html()) {
                if let Some(date) = cap.get(1) {
                    return Ok(Some(NaiveDate::parse_from_str(date.as_str(), "%B %d, %Y")?));
                }
            }
        }
    }

    Ok(None)
}

async fn fetch_paid_article_urls() -> Result<Vec<rss::Item>, Box<dyn Error>> {
    let response = get("https://lwn.net/headlines/rss").await?.bytes().await?;
    let channel = rss::Channel::read_from(&response[..])?;

    Ok(channel
        .items()
        .iter()
        .filter(|i| i.title().unwrap_or("").starts_with("[$]"))
        .map(|i| i.clone())
        .collect::<Vec<rss::Item>>())
}
