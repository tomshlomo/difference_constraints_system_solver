use std::collections::HashMap;
use std::fmt::Debug;

use crate::common::{Constraint, VarId};

#[derive(Debug, Clone)]
pub struct Solution<T: VarId>(HashMap<T, i64>);

impl<'a, T: VarId + 'a> Solution<T> {
    pub fn new() -> Solution<T> {
        let map = HashMap::new();
        Solution(map)
    }
    pub fn update(&mut self, var: &T, val: i64) {
        self.0.insert(var.clone(), val);
    }
    pub fn get_or(&self, var: &T, default: i64) -> i64 {
        *self.get(var).unwrap_or(&default)
    }
    pub fn get(&self, var: &T) -> Option<&i64> {
        self.0.get(var)
    }
    pub fn check_constraint(&self, constraint: &Constraint<T>) -> bool {
        if let (Some(u), Some(v)) = (self.get(&constraint.u), self.get(&constraint.v)) {
            return v - u <= constraint.c;
        }
        true
    }
    pub fn check_constraints<I: Iterator<Item = &'a Constraint<T>>>(
        &self,
        mut constraints: I,
    ) -> Option<&'a Constraint<T>> {
        constraints.find(|constraint| !self.check_constraint(constraint))
    }
    pub fn batch_update(&mut self, map: HashMap<T, i64>) {
        self.0.extend(map);
    }
    pub fn check_constraint_and_add_vars_if_missing(&mut self, constraint: &Constraint<T>) -> bool {
        match (self.get(&constraint.v), self.get(&constraint.u)) {
            (Some(d_v), Some(d_u)) => d_v - d_u <= constraint.c,
            (None, Some(d_u)) => {
                self.update(&constraint.v, constraint.c + d_u);
                true
            }
            (Some(d_v), None) => {
                self.update(&constraint.u, d_v - constraint.c);
                true
            }
            (None, None) => {
                self.update(&constraint.v, constraint.c);
                self.update(&constraint.u, 0);
                true
            }
        }
    }
}

impl<T: VarId> FromIterator<(T, i64)> for Solution<T> {
    fn from_iter<I: IntoIterator<Item = (T, i64)>>(iter: I) -> Self {
        Solution(HashMap::from_iter(iter))
    }
}

impl<T: VarId> Default for Solution<T> {
    fn default() -> Self {
        Self::new()
    }
}
