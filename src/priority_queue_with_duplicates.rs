use priority_queue::PriorityQueue;
use std::{collections::HashMap, hash::Hash};

pub struct PriorityQueueWithDuplicates<I: Hash + Eq, P: Ord> {
    items: HashMap<I, (usize, usize)>,
    queue: PriorityQueue<usize, P>,
}

impl<I: Hash + Eq, P: Ord> PriorityQueueWithDuplicates<I, P> {
    pub fn new(self) -> Self {
        PriorityQueueWithDuplicates {
            items: HashMap::new(),
            queue: PriorityQueue::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn insert(&mut self, item: I, priority: P) {
        let len = self.len();
        let (index, copies) = self.items.entry(item).or_insert((len, 0));
        self.queue.push(*index, priority);
        *copies += 1;
    }
    pub fn peek(self) -> Option<(&usize, &P)> {
        self.queue.peek()
    }
}
