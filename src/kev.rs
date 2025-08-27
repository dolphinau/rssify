use reqwest::get;
use rss::{Guid, Item};
use std::path::PathBuf;
use tokio_postgres;

use crate::{
    error::{Error, RssifyResult},
    source::{self, Source},
};

pub struct KEV;
const URL: &str =
    "https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json";

impl Source for KEV {
    async fn fetch(client: &tokio_postgres::Client) -> RssifyResult<()> {
        if let Err(e) = client
            .query(
                "CREATE TABLE IF NOT EXISTS kev (
                        title TEXT,
                        cveID TEXT,
                        description TEXT,
                        dateAdded TEXT
                    )",
                &[],
            )
            .await
        {
            eprintln!("[x] KEV table creation error: {}", e);
        }

        let text = get(URL).await?.text().await?;

        if let Ok(json) = json::parse(&text) {
            if let Ok(last_db_entry) = client
                .query(
                    "SELECT dateAdded, cveID FROM kev ORDER BY dateAdded desc LIMIT 1",
                    &[],
                )
                .await
            {
                let (last_db_cve_id, last_db_date_added): (&str, &str) = match last_db_entry.first()
                {
                    Some(row) => (
                        row.try_get("cveID")
                            .map_err(|_| Error::invalid_kev_catalogue())?,
                        row.try_get("dateAdded")
                            .map_err(|_| Error::invalid_kev_catalogue())?,
                    ),
                    _ => ("", ""),
                };

                println!(
                    "[DEBUG] Last db entry: {:?} - {:?}",
                    last_db_cve_id, last_db_date_added
                );

                let new_entries = json["vulnerabilities"]
                    .members()
                    .take_while(|entry| entry["cveID"] != last_db_cve_id);

                for entry in new_entries {
                    if let Err(e) = client
                        .query(
                            "INSERT INTO kev (
                            cveID,
                            title,
                            dateAdded,
                            description
                        ) VALUES (
                            $1, $2, $3, $4)",
                            &[
                                &entry["cveID"].as_str(),
                                &format!("{} - {}", entry["cveID"], entry["vulnerabilityName"]),
                                &entry["dateAdded"].as_str(),
                                &format!(
                                    "Description: {}\nRequired actions: {}\nNotes: {}",
                                    entry["shortDescription"],
                                    entry["requiredAction"],
                                    entry["notes"]
                                ),
                            ],
                        )
                        .await
                    {
                        eprintln!("[x] Error insert: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    async fn publish(client: &tokio_postgres::Client, mut path: PathBuf) -> RssifyResult<()> {
        let mut items: Vec<Item> = Vec::new();
        if let Ok(entries) = client
            .query("SELECT * FROM kev ORDER BY dateAdded desc LIMIT 15", &[])
            .await
        {
            for entry in entries {
                let mut guid = Guid::default();
                guid.set_value(entry.get::<_, &str>("cveID"));

                items.push(
                    rss::ItemBuilder::default()
                        .title(Some(entry.get("title")))
                        .link(Some(String::from(URL)))
                        .guid(Some(guid))
                        .pub_date(Some(entry.get("dateAdded")))
                        .description(Some(entry.get("description")))
                        .build(),
                );
            }
        };

        let channel = rss::ChannelBuilder::default()
            .title("CISA KEV")
            .link(URL)
            .description("CISA Catalog of Known Exploited Vulnerabilities")
            .items(items)
            .build();

        path.push("kev.xml");
        source::save_xml(&channel.to_string(), &path)
    }
}
