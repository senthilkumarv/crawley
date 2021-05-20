use hyper::Body;
use hyper::client::{Client, HttpConnector};
use hyper_tls::HttpsConnector;

pub use crawl_client::CrawlClient;
pub use error::CrawlClientError;

use crate::client::crawl_client::CrawleyCrawlClient;

mod crawl_client;
mod error;

#[cfg(test)]
pub use crate::client::crawl_client::MockCrawlClient;

pub fn create_client() -> CrawleyCrawlClient {
    let client = Client::builder()
        .build::<HttpsConnector<HttpConnector>, Body>(HttpsConnector::new());
    CrawleyCrawlClient::new(client)
}