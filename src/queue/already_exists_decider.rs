use flurry::HashSet;
use crate::queue::queue_addition_decider::QueueAdditionDecider;

pub struct ItemAlreadyExistsDecider<'a> {
    queue: &'a HashSet<String>,
    processed: &'a HashSet<String>,
}

impl <'a> QueueAdditionDecider for ItemAlreadyExistsDecider<'a> {
    fn can_add_to_queue(&self, link: &str) -> bool {
        let finished_guard = self.processed.guard();
        let queue_gaurd = self.queue.guard();
        !self.processed.contains(link, &finished_guard) && !self.queue.contains(link, &queue_gaurd)
    }
}

impl <'a> ItemAlreadyExistsDecider<'a> {
    pub fn new(queue: &'a HashSet<String>, processed: &'a HashSet<String>) -> ItemAlreadyExistsDecider<'a> {
        ItemAlreadyExistsDecider {
            queue: &queue,
            processed: &processed
        }
    }
}

#[cfg(test)]
mod tests {
    use flurry::HashSet;
    use crate::queue::already_exists_decider::ItemAlreadyExistsDecider;
    use crate::queue::queue_addition_decider::QueueAdditionDecider;

    #[test]
    fn should_decide_true_when_link_not_already_in_queue() {
        let queue = HashSet::<String>::new();
        queue.insert("http://domain.com/page1.html".to_string(), &queue.guard());
        let processed = HashSet::<String>::new();
        let decider = ItemAlreadyExistsDecider::new(&queue, &processed);

        let decision = decider.can_add_to_queue("http://domain.com/page2.html");

        assert!(decision)
    }

    #[test]
    fn should_decide_true_when_link_not_already_in_processed() {
        let queue = HashSet::<String>::new();
        let processed = HashSet::<String>::new();
        processed.insert("http://domain.com/page1.html".to_string(), &processed.guard());
        let decider = ItemAlreadyExistsDecider::new(&queue, &processed);

        let decision = decider.can_add_to_queue("http://domain.com/page2.html");

        assert!(decision)
    }

    #[test]
    fn should_decide_false_when_link_already_in_queue() {
        let queue = HashSet::<String>::new();
        queue.insert("http://domain.com/page1.html".to_string(), &queue.guard());
        let processed = HashSet::<String>::new();
        let decider = ItemAlreadyExistsDecider::new(&queue, &processed);

        let decision = decider.can_add_to_queue("http://domain.com/page1.html");

        assert!(!decision)
    }

    #[test]
    fn should_decide_false_when_link_already_in_processed() {
        let queue = HashSet::<String>::new();
        let processed = HashSet::<String>::new();
        processed.insert("http://domain.com/page1.html".to_string(), &processed.guard());
        let decider = ItemAlreadyExistsDecider::new(&queue, &processed);

        let decision = decider.can_add_to_queue("http://domain.com/page1.html");

        assert!(!decision)
    }
}