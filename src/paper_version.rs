use priority_queue::PriorityQueue;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;

pub trait VarId: Eq + Hash + Debug + Clone + Display {}
impl<T> VarId for T where T: Eq + Hash + Debug + Clone + Display {}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Constraint<T: VarId> {
    // v - u <= c
    pub v: T,
    pub u: T,
    pub c: i64,
}
impl<T: VarId> Constraint<T> {
    pub fn new(v: T, u: T, c: i64) -> Self {
        Constraint { v, u, c }
    }
}

impl<T: VarId> Display for Constraint<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {} <= {}", self.v, self.u, self.c)
    }
}

#[derive(Debug, Clone)]
pub struct Solution<T: VarId>(HashMap<T, i64>);

impl<T: VarId> Solution<T> {
    pub fn new() -> Solution<T> {
        let map = HashMap::new();
        Solution(map)
    }
    fn update(&mut self, var: &T, val: i64) {
        self.0.insert(var.clone(), val);
    }
    pub fn add_var_if_missing(&mut self, var: T) {
        // todo: use entry API
        if !self.0.contains_key(&var) {
            self.update(&var, 0);
        };
    }
    pub fn get_or(&self, var: &T, default: i64) -> i64 {
        *self.get(var).unwrap_or(&default)
    }
    fn get(&self, var: &T) -> Option<&i64> {
        self.0.get(var)
    }
    pub fn check_constraint(&self, constraint: &Constraint<T>) -> bool {
        // v - u <= c
        if let (Some(u_val), Some(v_val)) = (self.get(&constraint.u), self.get(&constraint.v)) {
            return v_val - u_val <= constraint.c;
        }
        true
    }
}

impl<T: VarId> Default for Solution<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DCS<T: VarId> {
    succesors: HashMap<T, Vec<(T, i64)>>,
}

impl<T: VarId> DCS<T> {
    //todo: implement implied_lb and implied_ub
    pub fn new() -> Self {
        let succesors = HashMap::new();
        DCS { succesors }
    }
    pub fn is_variable(&self, x: &T) -> bool {
        self.succesors.contains_key(x)
    }
    pub fn add_unconstrained_variable(&mut self, x: &T) {
        if !self.is_variable(x) {
            self.succesors.insert(x.clone(), vec![]);
        }
    }
    fn var_constraints(&self, u: &T) -> Vec<Constraint<T>> {
        // todo: just return an iterator, not a vec
        self.succesors
            .get(u)
            .unwrap_or(&vec![])
            .iter()
            .map(|(v, c)| Constraint::new(v.clone(), u.clone(), *c))
            .collect()
    }
    fn constraints(&self) -> Vec<Constraint<T>> {
        // todo: not great
        self.succesors
            .keys()
            .into_iter()
            .flat_map(|var| self.var_constraints(var))
            .collect()
    }
    pub fn check_solution(&self, sol: &Solution<T>) -> bool {
        for constraint in self.constraints() {
            if !sol.check_constraint(&constraint) {
                println!(
                    "constraint {} is not satisfied by the solution {:#?}",
                    constraint, sol.0
                );
                return false;
            }
        }
        true
    }
    fn add_succesor(&mut self, from_var: &T, to_variable: &T, c: i64) {
        self.succesors
            .get_mut(from_var)
            .expect("variable should be added")
            .push((to_variable.clone(), c));
    }
    pub fn add_to_feasible(
        &mut self,
        constraint: &Constraint<T>,
        sol: &Solution<T>,
    ) -> Option<Solution<T>> {
        println!("adding constraint {}", constraint);
        println!("current solution is: {:#?}", sol.0);
        self.add_unconstrained_variable(&constraint.v);
        self.add_unconstrained_variable(&constraint.u);
        self.add_succesor(&constraint.u, &constraint.v, constraint.c);
        let mut new_sol = sol.clone();
        new_sol.add_var_if_missing(constraint.u.clone());
        new_sol.add_var_if_missing(constraint.v.clone());
        if new_sol.check_constraint(constraint) {
            return Some(new_sol)
        }

        let mut q = PriorityQueue::new();
        q.push(&constraint.v, 0);
        loop {
            if let Some((x, dist_x)) = q.pop() {
                let new_val = sol.get(&constraint.u).unwrap_or(&0) + constraint.c + dist_x;
                let d_x = *sol.get(x).unwrap_or(&0);
                if new_val < d_x {
                    if x == &constraint.u {
                        self.succesors.get_mut(&constraint.u).unwrap().pop(); // remove the new constraint
                        print!("System is infeasible");
                        return None;
                    }
                    new_sol.update(x, new_val);
                    for (y, x2y) in self.succesors.get(x).unwrap() {
                        let scaled_path_len = dist_x + d_x + x2y - sol.get_or(y, 0);
                        q.push_decrease(y, scaled_path_len);
                    }
                }
            } else {
                println!("new solution is: {:#?}", new_sol.0);
                return Some(new_sol);
            }
        }
    }
}

impl<T: VarId> Default for DCS<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn check_multiple_constraints<T: VarId, I: Iterator<Item = (T, T, i64)>>(
        constraints: I,
    ) -> Solution<T> {
        let mut sys = DCS::new();
        let mut sol = Solution::new();
        for (v, u, c) in constraints {
            let constraint = Constraint::new(v, u, c);
            if let Some(new_sol) = sys.add_to_feasible(&constraint, &sol) {
                assert!(sys.check_solution(&new_sol));
                sol = new_sol;
            } else {
                panic!()
            }
        }
        sol
    }

    #[test]
    fn test_check() {
        let x = "x".to_owned();
        let y = "y".to_owned();
        let mut sys = DCS::new();
        let sol = Solution::new();
        let new_sol = sys.add_to_feasible(&Constraint::new(x, y, 0), &sol);
        assert!(sys.check_solution(&new_sol.unwrap()))
    }

    #[test]
    fn test_check2() {
        check_multiple_constraints(
            [
                ("y".to_owned(), "x".to_owned(), 1),
                ("z".to_owned(), "y".to_owned(), 2),
                ("x".to_owned(), "z".to_owned(), -3),
                ("z".to_owned(), "x".to_owned(), 4),
            ]
            .into_iter(),
        );
        println!("hello")
    }

    #[test]
    fn test_check3() {
        let x = "x";
        let y = "y";
        let z = "z";
        check_multiple_constraints([(y, x, 1), (z, y, 2), (x, z, -3), (z, x, 4)].into_iter());
        println!("hello")
    }
}
