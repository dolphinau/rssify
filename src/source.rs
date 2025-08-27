use crate::error::{Error, RssifyResult};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub trait Source {
    fn fetch(client: &tokio_postgres::Client) -> impl Future<Output = RssifyResult<()>>;
    fn publish(
        client: &tokio_postgres::Client,
        path: PathBuf,
    ) -> impl Future<Output = RssifyResult<()>>;
}

pub fn save_xml(rss_string: &str, path: &std::path::PathBuf) -> RssifyResult<()> {
    if let Ok(mut file) = File::create(path) {
        if file.write_all(rss_string.as_bytes()).is_ok() {
            return Ok(());
        }
    }

    Err(Error::save_xml_error(path.to_str().unwrap()))
}
