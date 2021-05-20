use tokio::sync::mpsc::{Receiver};

use crate::service::ScrapeService;

pub struct Crawly<Scraper: ScrapeService> {
    scraper: Scraper,
}

impl<Scraper: ScrapeService> Crawly<Scraper> {
    pub fn new(scraper: Scraper) -> Crawly<Scraper> {
        Crawly {
            scraper,
        }
    }

    pub async fn start_crawling(&self, start_url: &str, rx: &mut Receiver<Vec<String>>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let _ = self.scraper.scrape_links(vec![start_url.to_string()]).await;
        while let Some(res) = rx.recv().await {
            let _ = self.scraper.scrape_links(res).await?;
            if !self.scraper.has_more_items_to_scrape() {
                rx.close();
            }
        }
        Ok(self.scraper.result().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc::channel;

    use crate::crawly::Crawly;
    use crate::service::MockScrapeService;

    #[tokio::test]
    async fn should_call_service_every_time_there_is_data_in_the_channel() {
        let (tx, mut rx) = channel::<Vec<String>>(2048);
        let mut service = MockScrapeService::new();
        service
            .expect_scrape_links()
            .times(5)
            .returning(|_| Ok(vec![]));
        service
            .expect_has_more_items_to_scrape()
            .returning(|| false);
        service
            .expect_result()
            .returning(|| vec![]);

        let crawly = Crawly::new(service);

        let _ = tx.send(vec!["page1".to_string()]).await;
        let _ = tx.send(vec!["page2".to_string()]).await;
        let _ = tx.send(vec!["page3".to_string()]).await;
        let _ = tx.send(vec!["page4".to_string()]).await;

        let result = crawly.start_crawling("https://test.com/start.html", &mut rx).await;

        assert_eq!(result.unwrap(), Vec::<String>::new())
    }
}
