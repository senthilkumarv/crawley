use flurry::HashSet;
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
        let finished_guard = self.processed.guard();
        let queue_gaurd = self.queue.guard();
        if self.processed.contains(link, &finished_guard) || self.queue.contains(link, &queue_gaurd) {
            return false;
        }
        self.queue.insert(link.to_string(), &finished_guard)
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

    pub fn len(&self) -> usize {
        self.queue.len()
    }
}
