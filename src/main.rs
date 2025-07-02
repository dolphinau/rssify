use chrono::NaiveDate;
use regex::Regex;
use reqwest::get;
use scraper::{Html, Selector};
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(fetch_release_date("https://lwn.net/Articles/1025629/"));
}

async fn fetch_release_date(url: &str) -> Option<NaiveDate> {
    let response = get(url).await.unwrap();
    let response_text = response.text().await.unwrap();

    if let Some(article_text) = Html::parse_document(&response_text)
        .select(&Selector::parse("div.ArticleText").unwrap())
        .next()
    {
        if let Some(yes) = article_text.select(&Selector::parse("p").unwrap()).last() {
            let re = Regex::new(
                r#"(?m)\(Alternatively, this item will become freely\n\s* available on ([A-Z][a-z]+ [0-9]{2}, [0-9]{4})\)"#,
            )
            .unwrap();
            if let Some(cap) = re.captures(&yes.inner_html()) {
                if let Some(date) = cap.get(1) {
                    return NaiveDate::parse_from_str(date.as_str(), "%B %d, %Y").ok();
                }
            }
        }
    }

    None
}

async fn fetch_paid_articles() -> Option<Vec<String>> {
    let response = get("https://lwn.net/headlines/rss").await.unwrap();
    let response_text = response.text().await.unwrap();

    None
}
