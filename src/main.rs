use chrono::{NaiveDate, NaiveDateTime, TimeZone, prelude::Local};
use clap::Parser;
use regex::Regex;
use reqwest::get;
use scraper::{Html, Selector};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use tokio::runtime::Runtime;
use tokio_postgres;

#[derive(Parser)]
struct Cli {
    path: std::path::PathBuf,
}

fn main() {
    let rt = Runtime::new().unwrap();

    let args = Cli::parse();
    let db_connection_string = &format!(
        "host={} dbname={} user={} password={}",
        env::var("POSTGRES_HOST").unwrap_or(String::from("localhost")),
        env::var("POSTGRES_USER").unwrap_or(String::from("dev")),
        env::var("POSTGRES_USER").unwrap_or(String::from("dev")),
        env::var("POSTGRES_PASSWORD").unwrap_or(String::from("dev"))
    );

    println!("Connection string: {}", db_connection_string);

    rt.block_on(async {
        // Connect to the database.
        if let Ok((client, connection)) =
            tokio_postgres::connect(db_connection_string, tokio_postgres::NoTls).await
        {
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("[x] Connection db error: {}", e);
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
                eprintln!("[x] Table creation error: {}", e);
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

            // TODO: How to manage the RSS xml file
            let mut channel = match File::open(&args.path) {
                Ok(file) => rss::Channel::read_from(BufReader::new(file)).unwrap(),
                _ => rss::ChannelBuilder::default()
                    .title("[$] lwn.net")
                    .link("https://dawl.fr/lwn.net/rss.xml")
                    .description("RSS flux of lwn.net paid articles that are freely released.")
                    .items(vec![])
                    .build(),
            };
            let mut items = channel.clone().into_items();

            if let Ok(saved_articles) = client.query("SELECT * FROM articles", &[]).await {
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

            channel.set_items(items);
            if let Err(e) = save_xml(&channel.to_string(), args.path) {
                eprintln!("[x] Failed to save xml: {}", e);
            }
        } else {
            eprintln!("[x] Failed to connect to the db");
        }
    });
}

fn save_xml(rss_string: &str, path: std::path::PathBuf) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(rss_string.as_bytes())?;
    Ok(())
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
