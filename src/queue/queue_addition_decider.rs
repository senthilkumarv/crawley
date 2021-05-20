use std::str::FromStr;
use std::convert::TryFrom;
use url::{Url};
use crate::link::LinkConstructionError;

#[cfg_attr(test, mockall::automock)]
pub trait QueueAdditionDecider: Sync + Send{
    fn can_add_to_queue(&self, link: &str) -> bool;
}

pub struct AllowOnlySameDomainDecider {
    parent_domain: String
}

impl TryFrom<&str> for AllowOnlySameDomainDecider {
    type Error = LinkConstructionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Url::parse(value)
            .map(|url| AllowOnlySameDomainDecider { parent_domain: url.domain().unwrap_or("").to_string() })
            .map_err(|err| err.into())
    }
}

impl QueueAdditionDecider for AllowOnlySameDomainDecider {
    fn can_add_to_queue(&self, link: &str) -> bool {
        Url::from_str(link)
            .map(|uri| uri.host_str().unwrap_or("") == self.parent_domain)
            .unwrap_or_else(|_| false)
    }
}

pub struct IgnoreJavaScriptLinksDecider;

impl QueueAdditionDecider for IgnoreJavaScriptLinksDecider {
    fn can_add_to_queue(&self, link: &str) -> bool {
        !link.starts_with("javascript:")
    }
}

#[cfg(test)]
mod tests {
    use crate::queue::queue_addition_decider::{MockQueueAdditionDecider, AllowOnlySameDomainDecider, QueueAdditionDecider, IgnoreJavaScriptLinksDecider};
    use crate::queue::CrawlQueue;
    use mockall::predicate::eq;
    use std::convert::TryFrom;

    #[test]
    fn should_not_add_item_when_one_of_the_deciders_fails() {
        let mut decider1 = MockQueueAdditionDecider::new();
        decider1.expect_can_add_to_queue()
            .with(eq("http://domain.com"))
            .returning(|_| false);
        decider1.expect_can_add_to_queue()
            .with(eq("http://test.com"))
            .returning(|_| true);
        let queue = CrawlQueue::new(vec![Box::new(decider1)]);

        let added = queue.add_all(vec![
            "http://test.com".to_string(),
            "http://domain.com".to_string()
        ]);

        assert_eq!(added, vec!["http://test.com"])
    }

    #[test]
    fn should_process_all_deciders_to_decide_on_addition() {
        let mut decider1 = MockQueueAdditionDecider::new();
        decider1.expect_can_add_to_queue()
            .times(2)
            .return_const(true);
        let mut decider2 = MockQueueAdditionDecider::new();
        decider2.expect_can_add_to_queue()
            .times(2)
            .return_const(true);
        let mut decider3 = MockQueueAdditionDecider::new();
        decider3.expect_can_add_to_queue()
            .times(1)
            .with(eq("http://test.com"))
            .returning(|_| true);
        decider3.expect_can_add_to_queue()
            .times(1)
            .with(eq("http://domain.com"))
            .returning(|_| false);
        let queue = CrawlQueue::new(vec![
            Box::new(decider1),
            Box::new(decider2),
            Box::new(decider3)
        ]);

        let added = queue.add_all(vec![
            "http://test.com".to_string(),
            "http://domain.com".to_string()
        ]);

        assert_eq!(added, vec!["http://test.com"])
    }

    #[test]
    fn should_allow_only_links_from_same_domain_in_allow_only_same_domain_decider() {
        let decider = AllowOnlySameDomainDecider::try_from("http://www.domain.com/page1.html").unwrap();

        let decision = decider.can_add_to_queue("http://www.domain.com/page2.html");

        assert!(decision)
    }

    #[test]
    fn should_not_allow_only_links_from_different_domain_in_allow_only_same_domain_decider() {
        let decider = AllowOnlySameDomainDecider::try_from("http://www.domain.com/page1.html").unwrap();

        let decision = decider.can_add_to_queue("http://sub.domain.com/page2.html");

        assert!(!decision)
    }

    #[test]
    fn should_not_allow_javascript_links_in_ignore_javascript_links_decider() {
        let decider = IgnoreJavaScriptLinksDecider;

        let decision = decider.can_add_to_queue("javascript:void()");

        assert!(!decision)
    }

    #[test]
    fn should_allow_non_javascript_links_in_ignore_javascript_links_decider() {
        let decider = IgnoreJavaScriptLinksDecider;

        let decision = decider.can_add_to_queue("http://sub.domain.com/page2.html");

        assert!(decision)
    }
}