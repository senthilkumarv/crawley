use std::io::Read;
use std::str::FromStr;

use hyper::{body::Buf, Client, Uri};
use select::document::Document;
use select::predicate::Name;
use crate::client::CrawlClientError;
use hyper_tls::HttpsConnector;
use hyper::client::HttpConnector;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait CrawlClient: Sync + Send {
    async fn crawl_and_fetch_links(&self, link: &str) -> Result<Vec<String>, CrawlClientError>;
}

pub struct CrawleyCrawlClient {
    client: Client<HttpsConnector<HttpConnector>>,
}

impl CrawleyCrawlClient {
    pub fn new(client: Client<HttpsConnector<HttpConnector>>) -> CrawleyCrawlClient {
        CrawleyCrawlClient {
            client
        }
    }
}

#[async_trait]
impl CrawlClient for CrawleyCrawlClient {
    async fn crawl_and_fetch_links(&self, url: &str) -> Result<Vec<String>, CrawlClientError> {
        let uri = Uri::from_str(url)?;
        let response = self.client.get(uri).await?;
        if !response.status().is_success() {
            return Err(CrawlClientError::ConnectionError);
        }
        let body = hyper::body::aggregate(response).await?;
        let mut bytes: Vec<u8> = vec![];
        body.reader().read_to_end(&mut bytes)?;
        let links = Document::from(String::from_utf8(bytes)?.as_str())
            .select(Name("a"))
            .filter_map(|anchor| anchor.attr("href").map(|href| href.to_string()))
            .collect::<Vec<String>>();
        Ok(links)
    }
}

#[cfg(test)]
mod tests {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};
    use crate::client::{create_client, CrawlClient, CrawlClientError};

    #[tokio::test]
    async fn should_call_upstream_and_extract_links_on_successful_response() {
        let page1 = r#"
        <section>
        <ol>
            <li><a href="http://domain.com/some_page1.html">page 1</a></li>
            <li><a href="http://domain.com/some_page2.html">page 2</a></li>
        </ol>
        <a href="http://domain.com/home.html">home</a>
        </section>
        "#;
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/base/path/page1.html"))
            .respond_with(ResponseTemplate::new(200).set_body_string(page1))
            .mount(&mock_server)
            .await;

        let client = create_client();
        let response = client.crawl_and_fetch_links(format!("{}/base/path/page1.html", mock_server.uri()).as_str())
            .await;

        assert!(response.is_ok());
        assert_eq!(response.unwrap(), vec![
            "http://domain.com/some_page1.html",
            "http://domain.com/some_page2.html",
            "http://domain.com/home.html"
        ])
    }

    #[tokio::test]
    async fn should_call_upstream_and_throw_error_on_failure() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/base/path/page1.html"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let client = create_client();
        let response = client.crawl_and_fetch_links(format!("{}/base/path/page1.html", mock_server.uri()).as_str())
            .await;

        assert!(response.is_err());
        assert_eq!(response.err(), Some(CrawlClientError::ConnectionError))
    }
}