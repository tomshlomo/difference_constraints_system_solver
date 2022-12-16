use pathfinding::prelude::dijkstra;
use priority_queue::PriorityQueue;
use std::cmp::Reverse;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use crate::common::{Constraint, VarId};
use crate::solution::Solution;

pub struct EdgeDoesNotExist;

#[derive(Default)]
struct MultiEdge {
    queue: PriorityQueue<i64, Reverse<i64>>,
    counts: HashMap<i64, usize>, // todo: maybe use counter crate
}

impl MultiEdge {
    fn push(&mut self, c: i64) -> bool {
        let Some(old_priority) = self.queue.push(c, Reverse(c)) else {
            self.counts.insert(c, 1);
            return true;
        };
        let count = self.counts.entry(c).or_insert(0);
        *count += 1;
        (old_priority.0 > c) && (count == &1)
    }
    fn peek(&self) -> Option<&i64> {
        self.queue.peek().map(|(k, _)| k)
    }
    fn remove(&mut self, c: i64) -> Result<bool, EdgeDoesNotExist> {
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
    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

// #[derive(Default)]
struct FromEdges<T: VarId>(HashMap<T, MultiEdge>);
impl<T: VarId> FromEdges<T> {
    fn is_empty(&self) -> bool {
        self.0.values().all(|a| a.is_empty()) // todo: cahce
    }
    fn to_pairs(&self) -> impl Iterator<Item = (&T, &i64)> + '_ {
        self.0
            .iter()
            .filter_map(|(var, multi_edge)| multi_edge.peek().map(|val| (var, val)))
    }
    fn add(&mut self, var: T, val: i64) -> bool {
        self.0.entry(var).or_default().push(val)
    }
    fn remove(&mut self, var: T, val: i64) -> Result<bool, EdgeDoesNotExist> {
        let Entry::Occupied(mut occupied_entry) = self.0.entry(var) else {
            return  Err(EdgeDoesNotExist);
        };
        let multi_edge = occupied_entry.get_mut();
        let out = multi_edge.remove(val);
        if multi_edge.is_empty() {
            occupied_entry.remove_entry();
        }
        out
    }
}

impl<T: VarId> Default for FromEdges<T> {
    fn default() -> Self {
        FromEdges(HashMap::new())
    }
}
struct Edges<T: VarId>(HashMap<T, FromEdges<T>>);
impl<T: VarId> Edges<T> {
    fn new() -> Self {
        Edges(HashMap::new())
    }
    fn is_empty(&self) -> bool {
        self.0.values().all(|a| a.is_empty()) // todo: cahce
    }
    fn to_constraints(&self) -> impl Iterator<Item = Constraint<T>> + '_ {
        self.0.iter().flat_map(|(u, from_edges)| {
            from_edges.to_pairs().map(|(v, c)| Constraint {
                v: v.clone(),
                u: u.clone(),
                c: *c,
            })
        })
    }
    fn add(&mut self, constraint: Constraint<T>) -> bool {
        self.0
            .entry(constraint.u)
            .or_default()
            .add(constraint.v, constraint.c)
    }
    fn remove(&mut self, constraint: Constraint<T>) -> Result<bool, EdgeDoesNotExist> {
        let Entry::Occupied(mut occupied_entry) = self.0.entry(constraint.u) else {
            return  Err(EdgeDoesNotExist);
        };
        let from_edges = occupied_entry.get_mut();
        let out = from_edges.remove(constraint.v, constraint.c);
        if from_edges.is_empty() {
            occupied_entry.remove_entry();
        }
        out
    }
}

pub struct FeasibleSystem<T: VarId> {
    edges: Edges<T>,
    pub solution: Solution<T>,
}

impl<T: VarId> FeasibleSystem<T> {
    pub fn new() -> Self {
        FeasibleSystem {
            edges: Edges::new(),
            solution: Solution::new(),
        }
    }
    pub fn constraints(&self) -> impl Iterator<Item = Constraint<T>> + '_ {
        self.edges.to_constraints()
    }
    pub fn check_solution(&self, sol: &Solution<T>) -> bool {
        for constraint in self.constraints() {
            if !sol.check_constraint(&constraint) {
                return false;
            }
        }
        true
    }
    pub fn attempt_add_constraint(&mut self, constraint: Constraint<T>) -> bool {
        if self
            .solution
            .check_constraint_and_add_vars_if_missing(&constraint)
        {
            self.edges.add(constraint);
            return true;
        }
        let new_sol = self.check_and_solve_new_constraint(&constraint);
        if let Some(sol_diff) = new_sol {
            self.edges.add(constraint);
            self.solution.batch_update(sol_diff);
            return true;
        }
        false
    }
    pub fn check_and_solve_new_constraint(
        &self,
        constraint: &Constraint<T>,
    ) -> Option<HashMap<T, i64>> {
        // todo: maybe this function should be outside the class
        let mut sol_diff = HashMap::new();
        let mut q: PriorityQueue<&T, (Reverse<i64>, i64)> = PriorityQueue::new();
        let mut visited = HashSet::new();
        let d_u = self.solution.get_or(&constraint.u, 0);
        let d_v = self.solution.get_or(&constraint.v, 0);
        q.push(&constraint.v, (Reverse(0), d_v));
        while let Some((x, (v2x_scaled, d_x))) = q.pop() {
            visited.insert(x);
            let v2x_descaled = v2x_scaled.0 - d_v + d_x;
            let new_val = d_u + constraint.c + v2x_descaled;
            let is_affected = d_x > new_val;
            if !is_affected {
                continue;
            }
            if x == &constraint.u {
                return None;
            }
            sol_diff.insert(x.clone(), new_val);
            let Some(succesors) = self.edges.0.get(x) else {
                    continue;
            };
            // equivalent to `for (y, x2y_scaled) in self.scaled_succesors(y)`, but with less lookups.
            for (y, x2y_unscaled) in succesors.to_pairs() {
                let d_y = self.solution.get_or(y, 0);
                let x2y_scaled = x2y_unscaled + d_x - d_y;
                let v2y_scaled = v2x_scaled.0 + x2y_scaled;
                if !visited.contains(y) {
                    q.push_increase(y, (Reverse(v2y_scaled), d_y));
                }
            }
        }
        Some(sol_diff)
    }
    pub fn remove_constraint(
        &mut self,
        constraint_to_remove: Constraint<T>,
    ) -> Result<bool, EdgeDoesNotExist> {
        // todo: there are two types of remove with different tradeoffs.
        // one (implemented below) that after removing an infeasible constraints, does not need to check the other feasible constraints.
        // however it does need to check every new constraint, even if the system is infeasible already.
        // the other doesn't need check new constraints when the system is already infeasible, but need to check every infeasible constraint after removal.
        // idea: hold 3 sets of constraints: feasible, infeasible, undetermined.
        // when adding a constraint, always add to undetermined.
        // self.status() -> Status { if self.infeasible.not_empty => InFeasible, else: {if self.undetermined.is_empty -> Feasible, Unfeasible}}
        // solve makes an undetermined system determined.
        // if the infeasible set is empty, it tries to empty the underetmined set.
        // addind constraints always adds them to the underetmined set.
        // removing a constraint: if undetermined, simply remove.
        // otherwise, move all infeasible constraints to undetermined.
        self.edges.remove(constraint_to_remove)
    }
    pub fn remove_constraints<I: Iterator<Item = Constraint<T>>>(
        &mut self,
        constraints: I,
    ) -> Result<bool, EdgeDoesNotExist> {
        let mut any_removed = false;
        let mut return_error = false;
        for constraint in constraints {
            if let Ok(any_removed_tmp) = self.remove_constraint(constraint) {
                any_removed |= any_removed_tmp
            } else {
                return_error = true;
            }
        }
        if return_error {
            Err(EdgeDoesNotExist) // todo: should be NonExistingConstraints, and hold which ones
        } else {
            Ok(any_removed)
        }
    }
    pub fn get_implied_ub(&self, x: &T, y: &T) -> Option<i64> {
        // gives the constraint x - y <= a (with smallest possible a) that is implied by the system
        self.dist(y, x)
    }
    pub fn get_implied_lb(&self, x: &T, y: &T) -> Option<i64> {
        // gives the constraint x - y >= a (with larget possible a) that is implied by the system.
        // equivalent to y - x <= -a
        self.get_implied_ub(x, y).map(|ub| -ub)
    }
    fn dist(&self, from_node: &T, to_node: &T) -> Option<i64> {
        let result = dijkstra(
            from_node,
            |node| self.scaled_succesors(node),
            |node| node == to_node,
        );
        result.map(|(_, cost)| self.descale_dist(cost, from_node, to_node))
    }
    fn scaled_succesors(&self, node: &T) -> Vec<(T, i64)> {
        // todo: return an iterator instead of Vec
        let Some(from_edges) = self.edges.0.get(node) else {
            return vec![]
        };
        let d_node = self.solution.get_or(node, 0);
        let out = from_edges
            .to_pairs()
            .map(|(y, w)| (y.clone(), d_node + w - self.solution.get_or(y, 0)))
            .collect();
        out
    }
    fn descale_dist(&self, scaled_dist: i64, from_node: &T, to_node: &T) -> i64 {
        -self.solution.get_or(from_node, 0) + scaled_dist + self.solution.get_or(to_node, 0)
    }
}
