use tokio::sync::mpsc::channel;

use crate::result_publisher::{create_tokio_publisher};
use crate::service::{ScrapeService};

pub struct Crawly<Scraper: ScrapeService> {
    scraper: Scraper,
}

impl<Scraper: ScrapeService> Crawly<Scraper> {
    pub fn new(scraper: Scraper) -> Crawly<Scraper> {
        Crawly {
            scraper,
        }
    }

    pub async fn start_crawling(&self, start_url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let (tx, mut rx) = channel::<Vec<String>>(2048);
        let _ = tx.send(vec![start_url.to_string()]).await?;
        let publisher = create_tokio_publisher(tx);
        while let Some(res) = rx.recv().await {
            info!("Result: {:?}", res);
            let _ = self.scraper.scrape_links(res, &publisher).await?;
            if !self.scraper.has_more_items_to_scrape() {
                rx.close();
            }
        }
        Ok(self.scraper.result().iter().map(|link| link.clone()).collect())
    }
}
