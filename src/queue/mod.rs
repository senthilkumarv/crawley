use std::convert::TryFrom;

pub use crawl_queue::CrawlQueue;
use queue_addition_decider::{AllowOnlySameDomainDecider, IgnoreJavaScriptLinksDecider};
use crate::link::LinkConstructionError;

mod crawl_queue;
mod queue_addition_decider;
mod already_exists_decider;

pub fn create_queue(parent: &str) -> Result<CrawlQueue, LinkConstructionError> {
    let queue = CrawlQueue::new(vec![
        Box::new(IgnoreJavaScriptLinksDecider),
        Box::new(AllowOnlySameDomainDecider::try_from(parent)?)
    ]);
    Ok(queue)
}