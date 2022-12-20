use std::collections::{hash_map::Entry, HashMap};

use priority_queue::PriorityQueue;

use crate::{
    common::{Constraint, ConstraintPriority, EdgeDoesNotExist, MultiConstraint, MultiEdge, VarId},
    solution::Solution,
};

pub struct PrioritizedMulticConstraints<T: VarId, P: ConstraintPriority> {
    // todo: no need to hold two maps. the priority queue Item can hold the multi edge, and we will just ignore it in the hash and eq
    queue: PriorityQueue<(T, T), P>,
    edges: HashMap<(T, T), MultiEdge<P>>,
}

impl<T: VarId, P: ConstraintPriority> PrioritizedMulticConstraints<T, P> {
    pub fn new() -> Self {
        PrioritizedMulticConstraints {
            queue: PriorityQueue::new(),
            edges: HashMap::new(),
        }
    }
    pub fn push(&mut self, constraint: Constraint<T, P>) -> bool {
        let node = (constraint.u, constraint.v);
        self.queue.push_increase(node.clone(), constraint.priority); // todo: not sure what happens if new priority is lower than existing one.
        self.edges
            .entry(node)
            .or_default()
            .push(constraint.c, constraint.priority)
    }
    pub fn pop(&mut self) -> Option<MultiConstraint<T, P>> {
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
    pub fn remove(&mut self, constraint: Constraint<T, P>) -> Result<bool, EdgeDoesNotExist> {
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
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    pub fn check_solution(&self, sol: &Solution<T>) -> bool {
        // todo: we are not iterating according to priority. a bit weird.
        for ((u, v), multi_edge) in self.edges.into_iter() {
            let Some((c, _)) = multi_edge.peek() else {continue};
            let constraint = Constraint {
                v,
                u,
                c: *c,
                priority: (),
            };
            if !sol.check_constraint(&constraint) {
                return false;
            }
        }
        true
    }
    pub fn includes(&self, constraint: &Constraint<T, P>) -> bool {
        let Some(multi_edge) = self.edges.get(&(constraint.u, constraint.v)) else {
            return false
        };
        let Some((c, _)) = multi_edge.peek() else {
            return false
        };
        c < &constraint.c
    }
}
