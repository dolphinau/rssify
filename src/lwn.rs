use chrono::{NaiveDate, NaiveDateTime, TimeZone, prelude::Local};
use regex::Regex;
use reqwest::get;
use rss::Item;
use scraper::{Html, Selector};
use std::path::PathBuf;
use tokio_postgres;

use crate::error::{Error, RssifyResult};
use crate::source::{self, Source};

pub struct LWN;

impl LWN {
    async fn fetch_release_date(url: &str) -> RssifyResult<Option<NaiveDateTime>> {
        let response = get(url).await?.text().await?;

        if let Some(article_text) = Html::parse_document(&response)
            .select(&Selector::parse("div.ArticleText").unwrap())
            .next()
        {
            if let Some(yes) = article_text.select(&Selector::parse("p").unwrap()).last() {
                let re = Regex::new(
                    r#"(?m)\(Alternatively, this item will become freely\s*available on ([A-Z][a-z]+ [0-9]{1,2}, [0-9]{4})\)"#,
                ).unwrap();
                if let Some(cap) = re.captures(&yes.inner_html()) {
                    if let Some(date) = cap.get(1) {
                        return match NaiveDate::parse_from_str(date.as_str(), "%B %d, %Y") {
                            Ok(date) => Ok(Some(date.and_hms_opt(0, 0, 0).unwrap())),
                            Err(_) => Err(Error::invalid_naive_date(date.as_str())),
                        };
                    }
                }
            }
        }

        Ok(None)
    }

    async fn fetch_paid_article_urls() -> RssifyResult<Vec<rss::Item>> {
        let response = get("https://lwn.net/headlines/rss").await?.bytes().await?;
        let channel = rss::Channel::read_from(&response[..])?;

        Ok(channel
            .items()
            .iter()
            .filter(|i| i.title().unwrap_or("").starts_with("[$]"))
            .map(|i| i.clone())
            .collect::<Vec<rss::Item>>())
    }
}

impl Source for LWN {
    async fn fetch(client: &tokio_postgres::Client) -> RssifyResult<()> {
        if let Err(e) = client
            .query(
                "CREATE TABLE IF NOT EXISTS lwn (
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
            eprintln!("[x] LWN table creation error: {}", e);
        }

        // Get new [$] articles
        if let Ok(items) = LWN::fetch_paid_article_urls().await {
            for item in items {
                if let Some(link) = item.link() {
                    match client
                        .query_opt("SELECT release_date FROM lwn WHERE link = $1", &[&link])
                        .await
                    {
                        Ok(None) => {
                            if let Ok(Some(date)) = LWN::fetch_release_date(&link).await {
                                if let (Some(title), Some(description), Some(pub_date)) =
                                    (item.title(), item.description(), item.pub_date())
                                {
                                    println!("Adding new article to db: {}", link);

                                    if let Err(e) = client
                                        .query(
                                            "INSERT INTO lwn (
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
                                        eprintln!("[x] Error insert: {}", e);
                                    }
                                }
                            }
                        }
                        _ => (),
                    }
                };
            }
        }

        Ok(())
    }

    async fn publish(client: &tokio_postgres::Client, mut path: PathBuf) -> RssifyResult<()> {
        let mut items: Vec<Item> = Vec::new();

        if let Ok(saved_articles) = client.query("SELECT * FROM lwn", &[]).await {
            for row in saved_articles {
                let date: &str = row.get("release_date");
                if let Ok(date) = NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S") {
                    if Local.from_local_datetime(&date).unwrap() < Local::now() {
                        let link: String = row.get("title");
                        let guid = rss::GuidBuilder::default()
                            .value(link.clone())
                            .permalink(true)
                            .build();

                        items.push(
                            rss::ItemBuilder::default()
                                .title(Some(row.get("title")))
                                .link(Some(link))
                                .guid(Some(guid))
                                .pub_date(Some(Local::now().to_rfc2822()))
                                .description(Some(row.get("description")))
                                .build(),
                        );
                    }
                }
            }
        };

        let channel = rss::ChannelBuilder::default()
            .title("[$] lwn.net")
            .link("https://dawl.fr/lwn.net/rss.xml")
            .description("RSS flux of lwn.net paid articles that are freely released.")
            .items(items)
            .build();

        path.push("lwn.xml");
        source::save_xml(&channel.to_string(), &path)
    }
}
