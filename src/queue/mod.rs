use std::convert::TryFrom;

use url::ParseError;

pub use crawl_queue::CrawlQueue;
use queue_addition_decider::{AllowOnlySameDomainDecider, IgnoreJavaScriptLinksDecider};

mod crawl_queue;
mod queue_addition_decider;

pub fn create_queue(parent: &str) -> Result<CrawlQueue, ParseError> {
    let queue = CrawlQueue::new(vec![
        Box::new(IgnoreJavaScriptLinksDecider),
        Box::new(AllowOnlySameDomainDecider::try_from(parent)?)
    ]);
    Ok(queue)
}