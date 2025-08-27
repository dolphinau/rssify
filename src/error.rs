use std::str::Utf8Error;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RssifyErrorKind {
    InvalidKevCatalogue,
    SaveXMLError,
    InvalidNaiveDate,
    Unknown,
}

#[derive(Debug)]
pub struct RssifyError {
    /// Kind of error
    kind: RssifyErrorKind,
    /// Associated message of the context
    pub message: String,
}

impl RssifyError {
    pub fn new(kind: RssifyErrorKind, message: &str) -> Self {
        RssifyError {
            kind,
            message: String::from(message),
        }
    }

    pub fn kind(&self) -> RssifyErrorKind {
        self.kind
    }
}

#[derive(Debug)]
pub enum Error {
    /// Rssify error
    RssifyError(RssifyError),
    Utf8Error(Utf8Error),
    ReqwestError(reqwest::Error),
    RssError(rss::Error),
}

impl Error {
    pub fn new(kind: RssifyErrorKind, message: &str) -> Self {
        Error::RssifyError(RssifyError::new(kind, message))
    }

    pub fn invalid_kev_catalogue() -> Self {
        Error::RssifyError(RssifyError::new(
            RssifyErrorKind::InvalidKevCatalogue,
            "[KEV] Invalid KEV catalogue: failed to parse the JSON entry",
        ))
    }

    pub fn save_xml_error(path: &str) -> Self {
        Error::RssifyError(RssifyError::new(
            RssifyErrorKind::SaveXMLError,
            &format!("Failed to safe XML feed to {}", path),
        ))
    }

    pub fn invalid_naive_date(date: &str) -> Self {
        Error::RssifyError(RssifyError::new(
            RssifyErrorKind::InvalidNaiveDate,
            &format!("Failed to parse NaiveDate {}", date),
        ))
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::ReqwestError(e)
    }
}

impl From<rss::Error> for Error {
    fn from(e: rss::Error) -> Self {
        Self::RssError(e)
    }
}

pub type RssifyResult<T> = Result<T, Error>;
