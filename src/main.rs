use std::error::Error;

use chrono::{NaiveDate, NaiveDateTime, TimeZone, prelude::Local};
use regex::Regex;
use reqwest::get;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::BufReader;
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

            if let Err(e) = client
                .query(
                    "CREATE TABLE IF NOT EXISTS articles (
                        link TEXT,
                        title TEXT,
                        description TEXT,
                        pub_date TEXT,
                        release_date TEXT
                    )",
                    &[],
                )
                .await
            {
                eprintln!("table creation error: {}", e);
            }

            // Get new [$] articles
            if let Ok(items) = fetch_paid_article_urls().await {
                for item in items {
                    if let Some(link) = item.link() {
                        match client
                            .query_opt(
                                "SELECT release_date FROM articles WHERE link = $1",
                                &[&link],
                            )
                            .await
                        {
                            Ok(None) => {
                                if let Ok(Some(date)) = fetch_release_date(&link).await {
                                    if let (Some(title), Some(description), Some(pub_date)) =
                                        (item.title(), item.description(), item.pub_date())
                                    {
                                        println!("Adding new article to db: {}", link);

                                        if let Err(e) = client
                                            .query(
                                                "INSERT INTO articles (
                                                link,
                                                title,
                                                description,
                                                pub_date,
                                                release_date
                                            ) VALUES (
                                                $1, $2, $3, $4, $5)",
                                                &[
                                                    &link,
                                                    &title,
                                                    &description,
                                                    &pub_date,
                                                    &date.to_string(),
                                                ],
                                            )
                                            .await
                                        {
                                            eprintln!("Error insert: {}", e);
                                        }
                                    }
                                }
                            }
                            _ => (),
                        }
                    };
                }
            }

            // TODO: How to manage the RSS xml file

            // TODO: Check for new free articles
            if let Ok(saved_articles) = client.query("SELECT * FROM articles", &[]).await {
                saved_articles.iter().for_each(|row| {
                    let date: &str = row.get("release_date");
                    if let Ok(date) = NaiveDateTime::parse_from_str(date, "%Y-%m-%d") {
                        println!("date: {}", date);
                        if Local.from_local_datetime(&date).unwrap() < Local::now() {
                            // TODO: item.publish
                        }
                    }
                });
            }
        }
    });
}

async fn fetch_release_date(url: &str) -> Result<Option<NaiveDateTime>, Box<dyn Error>> {
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
                    return Ok(Some(
                        NaiveDate::parse_from_str(date.as_str(), "%B %d, %Y")?
                            .and_hms_opt(0, 0, 0)
                            .unwrap(),
                    ));
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
