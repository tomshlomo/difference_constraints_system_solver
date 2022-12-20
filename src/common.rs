use std::cmp::Reverse;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;

use priority_queue::PriorityQueue;

pub trait VarId: Eq + Hash + Debug + Clone + Display {}
impl<T> VarId for T where T: Eq + Hash + Debug + Clone + Display {}

pub trait ConstraintPriority: Ord + Clone {}
impl<P> ConstraintPriority for P where P: Ord + Clone {}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Constraint<T: VarId, P: ConstraintPriority> {
    // v - u <= c
    pub v: T,
    pub u: T,
    pub c: i64,
    pub priority: P,
}

impl<T: VarId, P: ConstraintPriority + Display> Display for Constraint<T, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} <= {} with priority {}",
            self.v, self.u, self.c, self.priority
        )
    }
}

pub struct EdgeDoesNotExist;

pub struct MultiEdge<P: ConstraintPriority> {
    // todo: no need to hold two maps. the priority queue Item can hold the counts, and we will just ignore it in the hash and eq. maybe the "derivative" crate can help
    queue: PriorityQueue<i64, (Reverse<i64>, P)>,
    counts: HashMap<i64, usize>,
}

impl<P: ConstraintPriority> MultiEdge<P> {
    pub fn push(&mut self, c: i64, priority: P) -> bool {
        let Some(old_priority) = self.queue.push(c,( Reverse(c), priority)) else {
            self.counts.insert(c, 1);
            return true;
        };
        let count = self.counts.entry(c).or_insert(0);
        *count += 1;
        (old_priority.0 .0 > c) && (count == &1)
    }
    pub fn merge(&mut self, other: Self) -> bool {
        let out = match (self.peek(), other.peek()) {
            (Some((c_self, _)), Some((c_other, _))) => c_other < c_self,
            (None, Some((c_other, _))) => true,
            (_, None) => false,
        };
        self.queue.append(&mut other.queue);
        for (var, count) in other.counts.into_iter() {
            *self.counts.entry(var).or_insert(0) += count;
        }
        out
    }
    pub fn peek(&self) -> Option<(&i64, &P)> {
        self.queue.peek().map(|(k, p)| (k, &p.1))
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
        if let Some((new_min, _)) = self.peek() {
            return Ok(new_min > &c);
        };
        Ok(true)
    }
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
impl<P: ConstraintPriority> Default for MultiEdge<P> {
    fn default() -> Self {
        MultiEdge {
            queue: PriorityQueue::new(),
            counts: HashMap::new(),
        }
    }
}
pub struct MultiConstraint<T: VarId, P: ConstraintPriority> {
    pub v: T,
    pub u: T,
    pub c: MultiEdge<P>,
}

impl<T: VarId, P: ConstraintPriority> MultiConstraint<T, P> {
    pub fn to_constraint(&self) -> Option<&Constraint<T, P>> {
        let Some((c, p)) = self.c.peek() else {return None};
        Some(&Constraint {
            v: self.v,
            u: self.u,
            c: *c,
            priority: *p,
        })
    }
}
