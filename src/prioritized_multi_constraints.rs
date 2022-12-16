use std::collections::{hash_map::Entry, HashMap};

use priority_queue::PriorityQueue;

use crate::common::{Constraint, EdgeDoesNotExist, MultiConstraint, MultiEdge, VarId};

pub struct PrioritizedMulticConstraints<T: VarId, P: Ord> {
    queue: PriorityQueue<(T, T), P>,
    edges: HashMap<(T, T), MultiEdge>,
}

impl<T: VarId, P: Ord> PrioritizedMulticConstraints<T, P> {
    pub fn new() -> Self {
        PrioritizedMulticConstraints {
            queue: PriorityQueue::new(),
            edges: HashMap::new(),
        }
    }
    pub fn push(&mut self, constraint: Constraint<T>, priority: P) -> bool {
        let node = (constraint.u, constraint.v);
        self.queue.push_increase(node.clone(), priority); // todo: not sure what happens if new priority is lower than existing one.
        self.edges.entry(node).or_default().push(constraint.c)
    }
    pub fn pop(&mut self) -> Option<MultiConstraint<T>> {
        let Some((node, _)) = self.queue.pop() else {
            return None
        };
        let multi_edge = self.edges.remove(&node).unwrap(); // todo: buggy unwrap?
        Some(MultiConstraint {
            v: node.1,
            u: node.0,
            c: multi_edge,
        })
    }
    pub fn remove(&mut self, constraint: Constraint<T>) -> Result<bool, EdgeDoesNotExist> {
        let node = (constraint.u, constraint.v);
        let Entry::Occupied(mut occupied_entry) = self.edges.entry(node) else {
            return  Err(EdgeDoesNotExist);
        };
        let multi_edge = occupied_entry.get_mut();
        let out = multi_edge.remove(constraint.c);
        if multi_edge.is_empty() {
            self.queue.remove(occupied_entry.key());
            occupied_entry.remove_entry();
        }
        out
    }
}
