#[macro_use]
extern crate async_trait;

use link::LinkConstructor;
use service::CrawleyScrapeService;

use crate::crawly::Crawly;
use clap::{App, Arg};
use crate::result_publisher::TokioResultPublisher;

mod queue;
mod service;
mod client;
mod link;
mod crawly;
mod result_publisher;

#[tokio::main(flavor = "multi_thread", worker_threads = 25)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    let matches = App::new("Crawley - The web crawler")
        .version("1.0")
        .author("Senthil V Kumar")
        .about("Crawls the web")
        .arg(Arg::new("INPUT")
            .about("Sets the domain to crawl")
            .required(true)
            .index(1))
        .get_matches();
    let url = matches.value_of("INPUT").unwrap_or("");
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<String>>(2048);
    let service = CrawleyScrapeService::new(
        client::create_client(),
        queue::create_queue(url)?,
        TokioResultPublisher::new(tx));
    let crawly = Crawly::new(service);
    let links = crawly.start_crawling(url, &mut rx).await?;
    links.iter().for_each(|link| println!("{:?}", link));
    Ok(())
}



