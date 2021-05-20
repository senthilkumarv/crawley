pub use error::ScraperError;
pub use scrape_service::{CrawleyScrapeService, ScrapeService};

#[cfg(test)]
pub use crate::service::scrape_service::MockScrapeService;

mod scrape_service;
mod error;