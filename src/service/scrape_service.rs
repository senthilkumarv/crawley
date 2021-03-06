use crate::client::{CrawlClient};
use crate::{LinkConstructor, result_publisher::ResultPublisher};
use std::convert::TryFrom;

use crate::service::error::ScraperError;
use crate::queue::CrawlQueue;
use futures::{FutureExt, StreamExt};
use futures::stream::FuturesUnordered;
use std::iter::FromIterator;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ScrapeService {
    fn has_more_items_to_scrape(&self) -> bool;
    fn result(&self) -> Vec<String>;

    async fn scrape_links(&self, links: Vec<String>) -> Result<Vec<String>, ScraperError>;
}

pub struct CrawleyScrapeService<C: CrawlClient, P: ResultPublisher<Vec<String>, ScraperError>> {
    client: C,
    queue: CrawlQueue,
    publisher: P,
}

impl<C: CrawlClient, P: ResultPublisher<Vec<String>, ScraperError>> CrawleyScrapeService<C, P> {
    pub fn new(client: C, queue: CrawlQueue, publisher: P) -> CrawleyScrapeService<C, P> {
        CrawleyScrapeService {
            client,
            queue,
            publisher
        }
    }
}

impl <C: CrawlClient, P: ResultPublisher<Vec<String>, ScraperError>> CrawleyScrapeService<C, P> {
    async fn scrape(&self, link: &str) -> Result<Vec<String>, ScraperError> {
        let constructor = LinkConstructor::try_from(link)?;
        let response = self.client.crawl_and_fetch_links(link).await;

        let links: Vec<String> = response?
            .iter()
            .filter_map(|href| constructor.construct(href).ok())
            .collect();
        let new_ones = self.queue.add_all(links.clone());
        self.publisher.notify(new_ones).await
    }
}

#[async_trait]
impl<C: CrawlClient, P: ResultPublisher<Vec<String>, ScraperError>> ScrapeService for CrawleyScrapeService<C, P> {
    fn has_more_items_to_scrape(&self) -> bool {
        !self.queue.is_empty()
    }

    fn result(&self) -> Vec<String> {
        self.queue.finished()
    }

    async fn scrape_links(&self, links: Vec<String>) -> Result<Vec<String>, ScraperError> {
        let items_added = self.queue.add_all(links.clone());
        let unvisited_links = self.queue.items();
        log::info!("Received {} Added {}", links.len(), items_added.len());
        let futures: Vec<_> = unvisited_links.iter().map(|link| {
            self.scrape(link)
                .then(move |links| {
                    self.queue.mark_as_done(link);
                    futures::future::ready(links)
                })
        }).collect();
        let mut all_futures = FuturesUnordered::from_iter(futures);
        let mut results: Vec<String> = vec![];
        while let Some(res) = all_futures.next().await {
            if let Ok(mut links) = res {
                results.append(&mut links);
            }
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use crate::client::MockCrawlClient;
    use crate::result_publisher::MockResultPublisher;
    use mockall::predicate::eq;
    use crate::service::{CrawleyScrapeService, ScraperError, ScrapeService};
    use crate::queue::{CrawlQueue, create_queue};

    #[tokio::test]
    async fn should_call_client_to_fetch_links_from_the_page() {
        let mut client = MockCrawlClient::new();
        client
            .expect_crawl_and_fetch_links()
            .with(eq("http://test.com/page1.html"))
            .returning(|_| Ok(vec!["http://test.com/page2.html".to_string(), "https://github.com/test.html".to_string(), "http://test.com/page3.html".to_string()]));
        let mut publisher = MockResultPublisher::<Vec<String>, ScraperError>::new();
        publisher
            .expect_notify()
            .with(eq(vec!["http://test.com/page2.html".to_string(), "https://github.com/test.html".to_string(), "http://test.com/page3.html".to_string()]))
            .returning(|_| Box::pin(futures::future::ok(vec!["".to_string()])));

        let service = CrawleyScrapeService::new(client, CrawlQueue::new(vec![]), publisher);

        let result = service.scrape("http://test.com/page1.html").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![""])
    }

    #[tokio::test]
    async fn should_call_client_after_transforming_relative_links_to_absolute() {
        let mut client = MockCrawlClient::new();
        client
            .expect_crawl_and_fetch_links()
            .with(eq("http://test.com/page1.html"))
            .returning(|_| Ok(vec!["page2.html".to_string(), "https://github.com/test.html".to_string(), "../page3.html".to_string()]));
        let mut publisher = MockResultPublisher::<Vec<String>, ScraperError>::new();
        publisher
            .expect_notify()
            .with(eq(vec!["http://test.com/page2.html".to_string(), "https://github.com/test.html".to_string(), "http://test.com/../page3.html".to_string()]))
            .returning(|_| Box::pin(futures::future::ok(vec!["".to_string()])));

        let service = CrawleyScrapeService::new(client, CrawlQueue::new(vec![]), publisher);

        let result = service.scrape("http://test.com/page1.html").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![""])
    }

    #[tokio::test]
    async fn should_call_client_to_fetch_links_for_all_given_links() {
        let mut client = MockCrawlClient::new();
        client
            .expect_crawl_and_fetch_links()
            .with(eq("http://test.com/base/page1.html"))
            .returning(|_| Ok(vec!["page2.html".to_string(), "https://github.com/test.html".to_string(), "../page3.html".to_string()]));
        client
            .expect_crawl_and_fetch_links()
            .with(eq("http://test.com/base/page2.html"))
            .returning(|_| Ok(vec!["page4.html".to_string(), "https://github.com/test.html".to_string(), "../page5.html".to_string()]));
        client
            .expect_crawl_and_fetch_links()
            .with(eq("http://test.com/base/page3.html"))
            .returning(|_| Ok(vec!["page6.html".to_string(), "https://github.com/test.html".to_string(), "../page7.html".to_string()]));
        let mut publisher = MockResultPublisher::<Vec<String>, ScraperError>::new();
        publisher
            .expect_notify()
            .returning(|a| Box::pin(futures::future::ok(a)));
        let service = CrawleyScrapeService::new(client, create_queue("http://test.com/").unwrap(), publisher);

        let result = service.scrape_links(vec![
            "http://test.com/base/page1.html",
            "http://test.com/base/page2.html",
            "http://test.com/base/page3.html"
        ].iter().map(|link| link.to_string()).collect()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 5);
    }
}