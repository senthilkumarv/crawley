use std::fmt::{Display, Formatter, Debug};
use hyper::http::uri::InvalidUri;
use url::ParseError;
use std::error::Error;

#[derive(Debug, Eq, PartialEq)]
pub enum LinkConstructionError {
    MissingScheme,
    BadUri,
    ParseError(String),
}

impl Display for LinkConstructionError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        let display_string = match self {
            LinkConstructionError::MissingScheme => "Uri is missing scheme".to_string(),
            LinkConstructionError::BadUri => "Bad Uri. Cannot parse".to_string(),
            LinkConstructionError::ParseError(kind) => format!("{}", kind)
        };
        write!(fmt, ": {}", display_string)
    }
}

impl From<InvalidUri> for LinkConstructionError {
    fn from(error: InvalidUri) -> Self {
        LinkConstructionError::ParseError(format!("{:?}", error))
    }
}

impl From<ParseError> for LinkConstructionError {
    fn from(error: ParseError) -> Self {
        LinkConstructionError::ParseError(format!("Bad URL({:?}). Should be in format of scheme://domain/path.", error))
    }
}

impl Error for LinkConstructionError {}
