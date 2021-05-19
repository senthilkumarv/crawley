use std::fmt::{Display, Formatter};
use std::error::Error;
use crate::link::LinkConstructionError;
use crate::client::CrawlClientError;

#[derive(Debug)]
pub enum ScraperError {
    InvalidUrl(String),
    ClientError,
}

impl Display for ScraperError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        let display_string = match self {
            ScraperError::InvalidUrl(_) => format!("Invalid link or url"),
            ScraperError::ClientError => format!("There was an error fetching from url")
        };
        writeln!(fmt, "{:?}", display_string)
    }
}

impl Error for ScraperError {}

impl From<LinkConstructionError> for ScraperError {
    fn from(err: LinkConstructionError) -> Self {
        ScraperError::InvalidUrl(err.to_string())
    }
}

impl From<CrawlClientError> for ScraperError {
    fn from(_: CrawlClientError) -> Self {
        ScraperError::ClientError
    }
}