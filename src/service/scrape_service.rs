use crate::client::{CrawlClient};
use crate::{LinkConstructor, result_publisher::ResultPublisher};
use std::convert::TryFrom;

use crate::service::error::ScraperError;

#[async_trait]
pub trait ScrapeService {
    async fn scrape(&self, link: &str, publisher: &dyn ResultPublisher<Vec<String>, ScraperError>) -> Result<Vec<String>, ScraperError>;
}

pub struct CrawleyScrapeService<C: CrawlClient> {
    client: C,
}

impl<C: CrawlClient> CrawleyScrapeService<C> {
    pub fn new(client: C) -> CrawleyScrapeService<C> {
        CrawleyScrapeService {
            client
        }
    }
}

#[async_trait]
impl<C: CrawlClient> ScrapeService for CrawleyScrapeService<C> {
    async fn scrape(&self, link: &str, publisher: &dyn ResultPublisher<Vec<String>, ScraperError>) -> Result<Vec<String>, ScraperError> {
        info!("Visiting {}", link);
        let constructor = LinkConstructor::try_from(link)?;
        let response = self.client.crawl_and_fetch_links(link).await;
        let links: Vec<String> = response?
            .iter()
            .filter_map(|href| constructor.construct(href).ok())
            .collect();
        publisher.notify(links.clone()).await
    }
}
