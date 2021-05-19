use flurry::HashSet;

use crate::queue::already_exists_decider::ItemAlreadyExistsDecider;
use crate::queue::queue_addition_decider::QueueAdditionDecider;

pub struct CrawlQueue {
    deciders: Vec<Box<dyn QueueAdditionDecider>>,
    processed: HashSet<String>,
    queue: HashSet<String>,
}

impl CrawlQueue {
    pub fn new(deciders: Vec<Box<dyn QueueAdditionDecider>>) -> CrawlQueue {
        let processed = HashSet::new();
        let queue = HashSet::new();
        CrawlQueue {
            deciders,
            processed,
            queue,
        }
    }

    pub fn add_all(&self, links: Vec<String>) -> Vec<String> {
        links.iter()
            .filter_map(|link| if self.add_to_queue(link) {
                Some(link.clone())
            } else { None })
            .collect()
    }

    pub fn add_to_queue(&self, link: &str) -> bool {
        if self.deciders.iter().any(|decider| !decider.can_add_to_queue(link)) {
            return false;
        }
        let default_decider = ItemAlreadyExistsDecider::new(&self.queue, &self.processed);
        if !default_decider.can_add_to_queue(link) {
            return false;
        }
        self.queue.insert(link.to_string(), &self.queue.guard())
    }

    pub fn mark_as_done(&self, link: &str) {
        let queue_guard = self.queue.guard();
        let processed_guard = self.processed.guard();
        self.queue.remove(link, &queue_guard);
        self.processed.insert(link.to_string(), &processed_guard);
    }

    pub fn finished(&self) -> Vec<String> {
        let finished_guard = self.processed.guard();
        self.processed.iter(&finished_guard).map(|link| link.clone()).collect()
    }

    pub fn items(&self) -> Vec<String> {
        let queue_guard = self.queue.guard();
        self.queue.iter(&queue_guard).map(|link| link.clone()).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use flurry::HashSet;

    use crate::queue::CrawlQueue;

    fn hash_set_to_vec(set: HashSet<String>) -> Vec<String> {
        set.iter(&set.guard()).map(|item| item.clone()).collect::<Vec<String>>()
    }

    fn vec_to_hash_set(vec: Vec<&str>) -> HashSet<String> {
        let set = HashSet::<String>::new();
        let guard = set.guard();
        vec.iter().for_each(|item| { set.insert(item.to_string(), &guard); });
        set
    }

    #[test]
    fn should_add_item_to_queue() {
        let queue = CrawlQueue::new(vec![]);

        let added_items = queue.add_to_queue("https://domain.com");

        assert!(added_items);
        assert_eq!(hash_set_to_vec(queue.queue), vec!["https://domain.com"])
    }

    #[test]
    fn should_not_add_item_to_queue_when_already_in_queue() {
        let queue = CrawlQueue {
            deciders: vec![],
            processed: HashSet::new(),
            queue: vec_to_hash_set(vec!["https://domain.com"]),
        };

        let added_items = queue.add_to_queue("https://domain.com");

        assert!(!added_items);
        assert_eq!(hash_set_to_vec(queue.queue), vec!["https://domain.com"])
    }

    #[test]
    fn should_not_add_item_to_queue_when_already_processed() {
        let queue = CrawlQueue {
            deciders: vec![],
            processed: vec_to_hash_set(vec!["https://domain.com"]),
            queue: HashSet::new(),
        };

        let added_items = queue.add_to_queue("https://domain.com");

        assert!(!added_items);
        assert_eq!(hash_set_to_vec(queue.queue), Vec::<&str>::new())
    }

    #[test]
    fn add_all_should_add_items_and_return_the_items_that_are_added() {
        let queue = CrawlQueue {
            deciders: vec![],
            processed: HashSet::new(),
            queue: vec_to_hash_set(vec!["https://domain.com"]),
        };

        let added = queue.add_all(vec![
            "https://domain.com".to_string(),
            "https://domain1.com".to_string()]);

        assert_eq!(added, vec!["https://domain1.com"])
    }

    #[test]
    fn should_move_item_from_queue_to_processed_when_marked_as_done() {
        let queue = CrawlQueue {
            deciders: vec![],
            processed: HashSet::new(),
            queue: vec_to_hash_set(vec!["https://domain.com"]),
        };

        queue.mark_as_done("https://domain.com");

        assert_eq!(hash_set_to_vec(queue.processed), vec!["https://domain.com"]);
        assert_eq!(hash_set_to_vec(queue.queue), Vec::<&str>::new());
    }

    #[test]
    fn finished_should_return_all_items_in_processed() {
        let queue = CrawlQueue {
            deciders: vec![],
            processed: vec_to_hash_set(vec!["https://processed.com"]),
            queue: vec_to_hash_set(vec!["https://queue.com"]),
        };

        let finished = queue.finished();

        assert_eq!(finished, vec!["https://processed.com"])
    }

    #[test]
    fn items_should_return_all_items_in_queue() {
        let queue = CrawlQueue {
            deciders: vec![],
            processed: vec_to_hash_set(vec!["https://processed.com"]),
            queue: vec_to_hash_set(vec!["https://queue.com"]),
        };

        let finished = queue.items();

        assert_eq!(finished, vec!["https://queue.com"])
    }

    #[test]
    fn is_empty_should_return_false_when_queue_is_not_empty() {
        let queue = CrawlQueue {
            deciders: vec![],
            processed: vec_to_hash_set(vec!["https://processed.com"]),
            queue: vec_to_hash_set(vec!["https://queue.com", "https://queue2.com"]),
        };

        let is_empty = queue.is_empty();

        assert_eq!(is_empty, false)
    }

    #[test]
    fn is_empty_should_return_true_when_queue_is_empty() {
        let queue = CrawlQueue {
            deciders: vec![],
            processed: vec_to_hash_set(vec!["https://processed.com"]),
            queue: vec_to_hash_set(vec![]),
        };

        let is_empty = queue.is_empty();

        assert_eq!(is_empty, true)
    }
}