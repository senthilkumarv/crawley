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
            let _ = self.scraper.scrape_links(res, &publisher).await?;
            if !self.scraper.has_more_items_to_scrape() {
                rx.close();
            }
        }
        Ok(self.scraper.result().iter().map(|link| link.clone()).collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::service::{ScrapeService, ScraperError};
    use crate::result_publisher::{ResultPublisher};
    use crate::crawly::Crawly;
    use crate::queue::CrawlQueue;

    struct MockScrapeService {
        queue: CrawlQueue,
    }
    #[async_trait]
    impl ScrapeService for MockScrapeService {
        fn has_more_items_to_scrape(&self) -> bool {
            !self.queue.is_empty()
        }

        fn result(&self) -> Vec<String> {
            self.queue.finished()
        }

        async fn scrape_links(&self, links: Vec<String>, publisher: &dyn ResultPublisher<Vec<String>, ScraperError>) -> Result<Vec<String>, ScraperError> {
            links.iter().for_each(|link| self.queue.mark_as_done(link.as_str()));
            self.queue.add_to_queue("https://test.com/page2.html");
            publisher.notify(self.queue.items()).await
        }
    }

    #[tokio::test]
    async fn should_call_service_every_time_there_is_data_in_the_channel() {
        let crawly = Crawly::new(MockScrapeService {
            queue: CrawlQueue::new(vec![])
        });

        let result = crawly.start_crawling("https://test.com/start.html").await;

        assert_eq!(result.unwrap(), vec!["https://test.com/start.html", "https://test.com/page2.html"])
    }
}
