use tokio::sync::mpsc::channel;

use crate::result_publisher::{create_tokio_publisher, TokioResultPublisher};
use crate::service::{ScrapeService};

pub struct Crawly<Scraper: ScrapeService<TokioResultPublisher<Vec<String>>>> {
    scraper: Scraper,
}

impl<Scraper: ScrapeService<TokioResultPublisher<Vec<String>>>> Crawly<Scraper> {
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
            println!("Received {:?}", res);
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
    use crate::service::{MockScrapeService};
    use crate::result_publisher::{TokioResultPublisher};
    use crate::crawly::Crawly;

    #[tokio::test]
    async fn should_call_service_every_time_there_is_data_in_the_channel() {
        let mut mock_service = MockScrapeService::<TokioResultPublisher<Vec<String>>>::new();
        mock_service
            .expect_scrape_links()
            .times(1)
            .returning(|_, _| Ok(vec![]));
        mock_service
            .expect_has_more_items_to_scrape()
            .returning(|| false);
        mock_service
            .expect_result()
            .times(1)
            .returning(|| vec!["https://test.com/start.html".to_string()]);
        let crawly = Crawly::new(mock_service);

        let result = crawly.start_crawling("https://test.com/start.html").await;

        assert_eq!(result.unwrap(), vec!["https://test.com/start.html"])
    }
}
