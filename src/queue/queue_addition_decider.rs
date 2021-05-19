use std::str::FromStr;
use std::convert::TryFrom;
use url::{Url, ParseError};

pub trait QueueAdditionDecider {
    fn can_add_to_queue(&self, link: &str) -> bool;
}

pub struct AllowOnlySameDomainDecider {
    parent_domain: String
}

impl TryFrom<&str> for AllowOnlySameDomainDecider {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Url::parse(value)
            .map(|url| AllowOnlySameDomainDecider { parent_domain: url.domain().unwrap_or_else(|| "").to_string() })
    }
}

impl QueueAdditionDecider for AllowOnlySameDomainDecider {
    fn can_add_to_queue(&self, link: &str) -> bool {
        Url::from_str(link)
            .map(|uri| uri.host().map(|host| host.to_string()).unwrap_or_else(|| "".to_string()) == self.parent_domain)
            .unwrap_or_else(|_| false)
    }
}

pub struct IgnoreJavaScriptLinksDecider;

impl QueueAdditionDecider for IgnoreJavaScriptLinksDecider {
    fn can_add_to_queue(&self, link: &str) -> bool {
        !link.starts_with("javascript:")
    }
}
