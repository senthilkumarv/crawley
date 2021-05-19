#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate log;


use link::LinkConstructor;
use service::CrawleyScrapeService;

use crate::crawly::Crawly;

mod queue;
mod service;
mod client;
mod link;
mod crawly;
mod result_publisher;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    let url = "https://www.paulirish.com/2010/the-protocol-relative-url/";
    let crawly = Crawly::new(
        CrawleyScrapeService::new(client::create_client()),
        queue::create_queue(url)?,
    );
    let links = crawly.start_crawling(url).await?;
    links.iter().for_each(|link| println!("{:?}", link));
    Ok(())
}



