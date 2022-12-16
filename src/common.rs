use std::cmp::Reverse;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;

use priority_queue::PriorityQueue;

pub trait VarId: Eq + Hash + Debug + Clone + Display {}
impl<T> VarId for T where T: Eq + Hash + Debug + Clone + Display {}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Constraint<T: VarId> {
    // v - u <= c
    pub v: T,
    pub u: T,
    pub c: i64,
}

impl<T: VarId> Display for Constraint<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {} <= {}", self.v, self.u, self.c)
    }
}

pub struct EdgeDoesNotExist;

#[derive(Default)]
pub struct MultiEdge {
    queue: PriorityQueue<i64, Reverse<i64>>,
    counts: HashMap<i64, usize>, // todo: maybe use counter crate
}

impl MultiEdge {
    pub fn push(&mut self, c: i64) -> bool {
        let Some(old_priority) = self.queue.push(c, Reverse(c)) else {
            self.counts.insert(c, 1);
            return true;
        };
        let count = self.counts.entry(c).or_insert(0);
        *count += 1;
        (old_priority.0 > c) && (count == &1)
    }
    pub fn peek(&self) -> Option<&i64> {
        self.queue.peek().map(|(k, _)| k)
    }
    pub fn remove(&mut self, c: i64) -> Result<bool, EdgeDoesNotExist> {
        let Entry::Occupied(mut occupied_entry) = self.counts.entry(c) else {
            return  Err(EdgeDoesNotExist);
        };
        let count = occupied_entry.get_mut();
        if count == &0 {
            return Err(EdgeDoesNotExist);
        };
        if *count > 1 {
            *count -= 1;
            return Ok(false);
        }
        // count == 1, we need to delete
        occupied_entry.remove_entry();
        self.queue.remove(&c);
        if let Some(new_min) = self.peek() {
            return Ok(new_min > &c);
        };
        Ok(true)
    }
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

pub struct MultiConstraint<T> {
    pub v: T,
    pub u: T,
    pub c: MultiEdge,
}
