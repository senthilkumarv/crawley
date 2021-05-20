pub use scrape_service::{ScrapeService, CrawleyScrapeService};
pub use error::ScraperError;

#[cfg(test)]
pub use crate::service::scrape_service::MockScrapeService;

mod scrape_service;
mod error;