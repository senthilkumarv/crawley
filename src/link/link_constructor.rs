use std::convert::TryFrom;
use hyper::Uri;
use std::str::FromStr;
use url::{Url, ParseError};
use crate::link::LinkConstructionError;

pub struct LinkConstructor {
    scheme: String,
    authority: String,
    path: String,
    parent: String,
}

impl TryFrom<&str> for LinkConstructor {
    type Error = LinkConstructionError;

    fn try_from(parent: &str) -> Result<Self, Self::Error> {
        let uri = Uri::from_str(parent)?;
        let scheme = uri.scheme_str()
            .map(Ok)
            .unwrap_or_else(|| Err(LinkConstructionError::MissingScheme))?;
        let authority = uri.authority().map(|authority| authority.as_str()).unwrap_or("");
        let path = if uri.path().ends_with('/') {
            uri.path().to_string()
        } else {
            let path_parts = uri.path().split('/').collect::<Vec<&str>>();
            let path: Vec<&str> = (0..(path_parts.len() - 1))
                .filter_map(|part| path_parts.get(part).cloned())
                .collect();
            format!("{}/", path.join("/"))
        };
        Ok(LinkConstructor {
            scheme: scheme.to_string(),
            authority: authority.to_string(),
            path,
            parent: parent.to_string()
        })
    }
}

impl LinkConstructor {
    pub fn construct(&self, href: &str) -> Result<String, LinkConstructionError> {
        if href.starts_with("javascript:") {
            return Err(LinkConstructionError::BadUri)
        }
        if href.starts_with('#') {
            return Ok(self.parent.clone())
        }
        let href_to_parse = if href.starts_with("//") {
            format!("{}:{}", self.scheme, href)
        } else {
            href.to_string()
        };
        let url = match Url::parse(href_to_parse.as_str()) {
            Ok(url) => Ok(url.to_string()),
            Err(ParseError::RelativeUrlWithoutBase) => if href.starts_with('/') {
                Ok(format!("{}://{}{}", self.scheme, self.authority, href))
            } else {
                Ok(format!("{}://{}{}{}", self.scheme, self.authority, self.path, href))
            },
            Err(_) => Err(LinkConstructionError::BadUri)
        }?;
        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use crate::link::link_constructor::{LinkConstructor, LinkConstructionError};
    use std::convert::TryFrom;

    #[test]
    fn should_create_valid_link_construction_with_valid_domain() {
        let href = "https://crawler.io/base/path1/";

        let constructor = LinkConstructor::try_from(href);

        assert_eq!(constructor.is_ok(), true);
        let unwrapped_constructor = constructor.unwrap();
        assert_eq!(unwrapped_constructor.authority, "crawler.io");
        assert_eq!(unwrapped_constructor.scheme, "https");
        assert_eq!(unwrapped_constructor.path, "/base/path1/");
    }

    #[test]
    fn should_create_valid_link_construction_with_valid_domain_and_include_port_if_specified() {
        let href = "https://crawler.io:9089/base/path1/";

        let constructor = LinkConstructor::try_from(href);

        assert_eq!(constructor.is_ok(), true);
        let unwrapped_constructor = constructor.unwrap();
        assert_eq!(unwrapped_constructor.authority, "crawler.io:9089");
        assert_eq!(unwrapped_constructor.scheme, "https");
        assert_eq!(unwrapped_constructor.path, "/base/path1/");
    }

    #[test]
    fn should_create_valid_link_construction_with_valid_domain_and_should_omit_resource_from_path() {
        let href = "https://crawler.io/base/path1/index.html";

        let constructor = LinkConstructor::try_from(href);

        assert_eq!(constructor.is_ok(), true);
        let unwrapped_constructor = constructor.unwrap();
        assert_eq!(unwrapped_constructor.authority, "crawler.io");
        assert_eq!(unwrapped_constructor.scheme, "https");
        assert_eq!(unwrapped_constructor.path, "/base/path1/");
    }

    #[test]
    fn should_fail_construction_when_the_base_path_does_not_have_valid_scheme() {
        let href = "/crawler.io/base/path1/index.html";

        let constructor = LinkConstructor::try_from(href);

        assert_eq!(constructor.is_ok(), false);
        assert_eq!(constructor.err().unwrap(), LinkConstructionError::MissingScheme)
    }

    #[test]
    fn should_fail_construction_when_the_base_path_is_invalid() {
        let href = "ssdsdsd/crawler.io/base/path1/index.html";

        let constructor = LinkConstructor::try_from(href);

        assert_eq!(constructor.is_ok(), false);
        assert_eq!(constructor.err().unwrap(), LinkConstructionError::ParseError("InvalidUri(InvalidFormat)".to_string()))
    }

    #[test]
    fn should_construct_link_for_links_with_entire_url() {
        let href = "https://crawler.io/base/path1/page.html";
        let constructor = LinkConstructor::try_from(href).unwrap();

        let constructed = constructor.construct("https://domain.crawler.io/base/page1.html");

        assert_eq!(constructed.is_ok(), true);
        assert_eq!(constructed.unwrap(), "https://domain.crawler.io/base/page1.html");
    }

    #[test]
    fn should_construct_link_for_links_that_start_with_relative_url() {
        let href = "https://crawler.io/base/path1/page.html";
        let constructor = LinkConstructor::try_from(href).unwrap();

        let constructed = constructor.construct("01_getting_started/01_chapter.html");

        assert_eq!(constructed.is_ok(), true);
        assert_eq!(constructed.unwrap(), "https://crawler.io/base/path1/01_getting_started/01_chapter.html");
    }

    #[test]
    fn should_construct_link_for_links_that_start_with_absolute_path() {
        let href = "https://crawler.io/base/path1/page.html";
        let constructor = LinkConstructor::try_from(href).unwrap();

        let constructed = constructor.construct("/01_getting_started/01_chapter.html");

        assert_eq!(constructed.is_ok(), true);
        assert_eq!(constructed.unwrap(), "https://crawler.io/01_getting_started/01_chapter.html");
    }

    #[test]
    fn should_construct_link_for_links_with_absolute_url_without_scheme() {
        let href = "https://crawler.io/base/path1/page.html";
        let constructor = LinkConstructor::try_from(href).unwrap();

        let constructed = constructor.construct("//crawler.io/base/path1/page2.html");

        assert_eq!(constructed.is_ok(), true);
        assert_eq!(constructed.unwrap(), "https://crawler.io/base/path1/page2.html");
    }

    #[test]
    fn should_construct_link_for_links_that_start_with_relative_url_with_just_page_name() {
        let href = "https://crawler.io/base/path1/page.html";
        let constructor = LinkConstructor::try_from(href).unwrap();

        let constructed = constructor.construct("chapter.html");

        assert_eq!(constructed.is_ok(), true);
        assert_eq!(constructed.unwrap(), "https://crawler.io/base/path1/chapter.html");
    }

    #[test]
    fn should_construct_link_for_links_that_start_with_relative_url_with_dot_dot_relative_url() {
        let href = "https://crawler.io/base/path1/page.html";
        let constructor = LinkConstructor::try_from(href).unwrap();

        let constructed = constructor.construct("../chapter.html");

        assert_eq!(constructed.is_ok(), true);
        assert_eq!(constructed.unwrap(), "https://crawler.io/base/path1/../chapter.html");
    }

    #[test]
    fn should_construct_link_for_self_referencing_paths() {
        let href = "https://crawler.io/base/path1/index.html";
        let constructor = LinkConstructor::try_from(href).unwrap();

        let constructed = constructor.construct("#bottom");

        assert_eq!(constructed.is_ok(), true);
        assert_eq!(constructed.unwrap(), "https://crawler.io/base/path1/index.html");
    }
}