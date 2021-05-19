use std::io::Read;
use std::str::FromStr;

use hyper::{body::Buf, Client, Uri};
use select::document::Document;
use select::predicate::Name;
use crate::client::CrawlClientError;
use hyper_tls::HttpsConnector;
use hyper::client::HttpConnector;

#[async_trait]
pub trait CrawlClient: Sync + Send {
    async fn crawl_and_fetch_links(&self, link: &str) -> Result<Vec<String>, CrawlClientError>;
}

pub struct CrawleyCrawlClient {
    client: Client<HttpsConnector<HttpConnector>>,
}

impl CrawleyCrawlClient {
    pub fn new(client: Client<HttpsConnector<HttpConnector>>) -> CrawleyCrawlClient {
        CrawleyCrawlClient {
            client
        }
    }
}

#[async_trait]
impl CrawlClient for CrawleyCrawlClient {
    async fn crawl_and_fetch_links(&self, url: &str) -> Result<Vec<String>, CrawlClientError> {
        let uri = Uri::from_str(url)?;
        let response = self.client.get(uri).await?;
        let body = hyper::body::aggregate(response).await?;
        let mut bytes: Vec<u8> = vec![];
        body.reader().read_to_end(&mut bytes)?;
        let links = Document::from(String::from_utf8(bytes)?.as_str())
            .select(Name("a"))
            .filter_map(|anchor| anchor.attr("href").map(|href| href.to_string()))
            .collect::<Vec<String>>();
        Ok(links)
    }
}
