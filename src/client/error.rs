use std::fmt::{Display, Formatter};
use std::error::Error;
use hyper::http::uri::InvalidUri;
use std::string::FromUtf8Error;

#[derive(Debug, Eq, PartialEq)]
pub enum CrawlClientError {
    InvalidUri,
    ConnectionError,
    IOError,
    EncodingError
}

impl Display for CrawlClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display_string = match self {
            CrawlClientError::InvalidUri => "Invalid link or url",
            CrawlClientError::ConnectionError => "There was an error connection to the page",
            CrawlClientError::IOError => "There was an error sending or receiving data",
            CrawlClientError::EncodingError => "There was an error parsing encoded data",
        };
        writeln!(f, "{:?}", display_string)
    }
}

impl Error for CrawlClientError {}

impl From<InvalidUri> for CrawlClientError {
    fn from(_: InvalidUri) -> Self {
        CrawlClientError::InvalidUri
    }
}

impl From<hyper::Error> for CrawlClientError {
    fn from(_: hyper::Error) -> Self {
        CrawlClientError::ConnectionError
    }
}

impl From<std::io::Error> for CrawlClientError {
    fn from(_: std::io::Error) -> Self {
        CrawlClientError::IOError
    }
}

impl From<FromUtf8Error> for CrawlClientError {
    fn from(_: FromUtf8Error) -> Self {
        CrawlClientError::EncodingError
    }
}