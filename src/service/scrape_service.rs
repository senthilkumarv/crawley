use crate::client::{CrawlClient};
use crate::{LinkConstructor, result_publisher::ResultPublisher};
use std::convert::TryFrom;

use crate::service::error::ScraperError;
use crate::queue::CrawlQueue;
use futures::{FutureExt};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ScrapeService<T: ResultPublisher<Vec<String>, ScraperError>> {
    fn has_more_items_to_scrape(&self) -> bool;
    fn result(&self) -> Vec<String>;

    async fn scrape_links(&self, links: Vec<String>, publisher: &T) -> Result<Vec<String>, ScraperError>;
}

pub struct CrawleyScrapeService<C: CrawlClient> {
    client: C,
    queue: CrawlQueue,
}

impl<C: CrawlClient> CrawleyScrapeService<C> {
    pub fn new(client: C, queue: CrawlQueue) -> CrawleyScrapeService<C> {
        CrawleyScrapeService {
            client,
            queue
        }
    }
}

impl <C: CrawlClient> CrawleyScrapeService<C> {
    async fn scrape(&self, link: &str, publisher: &dyn ResultPublisher<Vec<String>, ScraperError>) -> Result<Vec<String>, ScraperError> {
        let constructor = LinkConstructor::try_from(link)?;
        let response = self.client.crawl_and_fetch_links(link).await;

        let links = response?
            .iter()
            .filter_map(|href| constructor.construct(href).ok())
            .collect();
        publisher.notify(links).await
    }
}

#[async_trait]
impl<C: CrawlClient, T: ResultPublisher<Vec<String>, ScraperError>> ScrapeService<T> for CrawleyScrapeService<C> {
    fn has_more_items_to_scrape(&self) -> bool {
        !self.queue.is_empty()
    }

    fn result(&self) -> Vec<String> {
        self.queue.finished()
    }

    async fn scrape_links(&self, links: Vec<String>, publisher: &T) -> Result<Vec<String>, ScraperError> {
        self.queue.add_all(links);
        let unvisited_links = self.queue.items();
        let futures: Vec<_> = unvisited_links.iter().map(|link| {
            self.scrape(link, publisher)
                .then(move |links| {
                    self.queue.mark_as_done(link);
                    futures::future::ready(links)
                })
        }).collect();
        let links = futures::future::join_all(futures).await
            .iter()
            .filter_map(|result| result.as_ref().ok().map(|links| links.clone()))
            .flatten()
            .collect::<Vec<String>>();
        self.queue.add_all(links.clone());
        Ok(links)
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

        let service = CrawleyScrapeService::new(client, CrawlQueue::new(vec![]));

        let result = service.scrape("http://test.com/page1.html", &publisher).await;

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

        let service = CrawleyScrapeService::new(client, CrawlQueue::new(vec![]));

        let result = service.scrape("http://test.com/page1.html", &publisher).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![""])
    }

    #[tokio::test]
    async fn should_call_client_to_fetch_links_for_all_given_links() {
        let mut client = MockCrawlClient::new();
        client
            .expect_crawl_and_fetch_links()
            .with(eq("http://test.com/page1.html"))
            .returning(|_| Ok(vec!["page2.html".to_string(), "https://github.com/test.html".to_string(), "../page3.html".to_string()]));
        client
            .expect_crawl_and_fetch_links()
            .with(eq("http://test.com/page2.html"))
            .returning(|_| Ok(vec!["page4.html".to_string(), "https://github.com/test.html".to_string(), "../page5.html".to_string()]));
        client
            .expect_crawl_and_fetch_links()
            .with(eq("http://test.com/page3.html"))
            .returning(|_| Ok(vec!["page6.html".to_string(), "https://github.com/test.html".to_string(), "../page7.html".to_string()]));
        let mut publisher = MockResultPublisher::<Vec<String>, ScraperError>::new();
        publisher
            .expect_notify()
            .returning(|a| Box::pin(futures::future::ok(a)));
        let service = CrawleyScrapeService::new(client, create_queue("http://test.com/").unwrap());

        let result = service.scrape_links(vec![
            "http://test.com/page1.html",
            "http://test.com/page2.html",
            "http://test.com/page3.html"
        ].iter().map(|link| link.to_string()).collect(), &publisher).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 9);
    }
}