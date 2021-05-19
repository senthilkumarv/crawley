use futures::TryFutureExt;
use tokio::sync::mpsc::channel;

use crate::queue::CrawlQueue;
use crate::result_publisher::{create_tokio_publisher, ResultPublisher};
use crate::service::{ScraperError, ScrapeService};

pub struct Crawly<Scraper: ScrapeService> {
    scraper: Scraper,
    queue: CrawlQueue,
}

impl<Scraper: ScrapeService> Crawly<Scraper> {
    pub fn new(scraper: Scraper, queue: CrawlQueue) -> Crawly<Scraper> {
        Crawly {
            scraper,
            queue,
        }
    }

    pub async fn start_crawling(&self, start_url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let (tx, mut rx) = channel::<Vec<String>>(2048);
        let _ = tx.send(vec![start_url.to_string()]).await?;
        let publisher = create_tokio_publisher(tx);
        while let Some(res) = rx.recv().await {
            info!("Received event with {} links, processing {}", res.clone().len(), self.queue.len());
            let _ = self.crawl_all(res, &publisher).await?;
            if self.queue.is_empty() {
                rx.close();
            }
        }
        Ok(self.queue.finished().iter().map(|link| link.clone()).collect())
    }

    async fn crawl_all(&self, links: Vec<String>, publisher: &dyn ResultPublisher<Vec<String>, ScraperError>) -> Result<(), Box<dyn std::error::Error>> {
        self.queue.add_all(links);
        let unvisited_links = self.queue.items();
        let parallels: Vec<_> = unvisited_links.iter().map(|link| {
            self.queue.mark_as_done(link.as_str());
            self.scraper.scrape(link, publisher)
                .and_then(|result| {
                    self.queue.add_all(result.clone());
                    futures::future::ok(result)
                })
        }).collect();
        futures::future::join_all(parallels).await;
        Ok(())
    }
}
