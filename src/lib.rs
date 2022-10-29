use std::{collections::HashMap};

use std::hash::Hash;

pub trait VarId: Eq + Hash {}
impl<T> VarId for T where T: Eq + Hash {}


#[derive(Debug)]
pub struct Solution<'a, T: VarId>(HashMap<&'a T, i64>);

impl<'a, T: VarId> Solution<'a, T> {
    pub fn new() -> Self {
        let map = HashMap::new();
        Solution(map)
    }
    pub fn update(&mut self, var: &'a T, val: i64) {
        self.0.insert(var, val);
    }
    pub fn get(&self, var: &'a T) -> i64 {
        self.0.get(var).unwrap_or(&0).clone()
    }
    pub fn add_var_if_missing(&mut self, var: &'a T) {
        if !self.0.contains_key(&var) {
            self.update(var, 0);
        };
    }
}

impl <'a, T: VarId>FromIterator<(&'a T, i64)> for Solution<'a, T> {
    fn from_iter<I: IntoIterator<Item=(&'a T, i64)>>(iter: I) -> Self {
        let mut sol = Solution::new();
        for (u, a) in iter {
            sol.update(u, a);
        }
        sol
    }
}

pub struct System<'a, T: VarId> (
    HashMap<&'a T, HashMap<&'a T, i64>>
);

impl<'a, T: VarId> System<'a, T> {
    pub fn new() -> Self {
        System(HashMap::new())
    }
    pub fn is_variable(&self, var: &T) -> bool {
        self.0.contains_key(var)
    }
    pub fn add_var_if_missing(&mut self, var: &'a T) {
        if self.is_variable(var) {
            return
        }
        self.0.insert(var, HashMap::new());
    }
    pub fn check(&self, sol: &Solution<'a, T>) -> bool {
        for (u, v2a) in self.0.iter() {
            for (v, a) in v2a.iter() {
                if sol.get(u) > sol.get(v) + a {
                    return false
                }
            }
        }
        true
    }
    pub fn add(&mut self, u: &'a T, v: &'a T, a: i64) {
        self.add_var_if_missing(u);
        self.add_var_if_missing(v);
        self.0.get_mut(u).unwrap().insert(v, a);
    }
    pub fn variables(&self) -> Vec<&'a T> {
        self.0.into_keys().collect()
    }
}

impl <'a, T: VarId>FromIterator<(&'a T, &'a T, i64)> for System<'a, T> {
    fn from_iter<I: IntoIterator<Item=(&'a T, &'a T, i64)>>(iter: I) -> Self {
        let mut sys = System::new();

        for (u, v, a) in iter {
            sys.add(u, v, a);
        }

        sys
    }
}


pub struct Constraint<'a, T: VarId> {
    // u - v <= a
    pub u: &'a T,
    pub v: &'a T,
    pub a: i64,
}

impl<'a, T: VarId> Constraint<'a, T> {
    pub fn new(u: &'a T, v: &'a T, a: i64) -> Self {
        Constraint {
            u: u,
            v: v,
            a: a,
        }
    }
    pub fn check(&self, sol: &Solution<'a, T>) -> bool {
        sol.get(&self.u) <= sol.get(&self.v) + self.a
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check() {
        let x = "x".to_owned();
        let y = "y".to_owned();
        let sys = System::from_iter([
            (&x, &y, 0),
            (&y, &x, 1),
        ]);
        let mut sol = Solution::from_iter([
            (&x, 1),
            (&y, 1),
        ]);
        assert_eq!(sys.check(&sol), true);
        sol.update(&x, 2);
        assert_eq!(sys.check(&sol), false);
        let x2 = "x".to_owned();
        sol.update(&x2, 1);
        assert_eq!(sys.check(&sol), true);
        sol.update(&x2, 2);
        assert_eq!(sys.check(&sol), false);
    }
}
