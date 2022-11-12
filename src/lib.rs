use pathfinding::prelude::dijkstra;
use priority_queue::PriorityQueue;
use std::collections::{HashMap, HashSet};
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
        assert!(v != u);
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
        self.0.entry(var).or_insert(0);
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
    pub fn merge(&mut self, other: &Solution<T>) {
        for (key, val) in other.0.iter() {
            self.0.entry(key.clone()).or_insert(*val);
        }
    }
}

impl<T: VarId> Default for Solution<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DCS<T: VarId> {
    feasible_constraints: HashMap<T, HashMap<T, i64>>,
    infeasible_constraints: HashMap<T, HashMap<T, i64>>,
}

impl<T: VarId> DCS<T> {
    pub fn new() -> Self {
        DCS {
            feasible_constraints: HashMap::new(),
            infeasible_constraints: HashMap::new(),
        }
    }
    pub fn is_variable(&self, x: &T) -> bool {
        self.feasible_constraints.contains_key(x)
    }
    pub fn add_unconstrained_variable(&mut self, x: &T) {
        if !self.is_variable(x) {
            self.feasible_constraints.insert(x.clone(), HashMap::new());
        }
    }
    fn var_constraints(&self, u: &T) -> Vec<Constraint<T>> {
        // all constraints of the form v - u <= c, for a given u.
        self.feasible_constraints
            .get(u)
            .unwrap_or(&HashMap::new())
            .iter()
            .map(|(v, c)| Constraint::new(v.clone(), u.clone(), *c))
            .collect()
    }
    fn constraints(&self) -> Vec<Constraint<T>> {
        self.feasible_constraints
            .keys()
            .into_iter()
            .flat_map(|var| self.var_constraints(var))
            .collect()
    }
    pub fn check_solution(&self, sol: &Solution<T>) -> bool {
        for constraint in self.constraints() {
            if !sol.check_constraint(&constraint) {
                return false;
            }
        }
        true
    }
    fn add_to_feasible(&mut self, constraint: &Constraint<T>) {
        self.feasible_constraints
            .entry(constraint.u.clone())
            .or_default()
            .insert(constraint.v.clone(), constraint.c);
    }
    fn add_to_infeasible(&mut self, constraint: &Constraint<T>) {
        self.feasible_constraints
            .entry(constraint.u.clone())
            .or_default()
            .insert(constraint.v.clone(), constraint.c);
    }
    pub fn add_constraint(
        &mut self,
        constraint: &Constraint<T>,
        sol: &Solution<T>,
    ) -> Option<Solution<T>> {
        let new_sol = self.check_and_solve_new_constraint(constraint, sol);
        match new_sol {
            Some(_) => self.add_to_feasible(constraint),
            None => self.add_to_infeasible(constraint),
        }
        new_sol
    }
    pub fn check_and_solve_new_constraint(
        &self,
        constraint: &Constraint<T>,
        sol: &Solution<T>,
    ) -> Option<Solution<T>> {
        let mut new_sol = Solution::new();
        let mut q: PriorityQueue<&T, i64> = PriorityQueue::new();
        q.push(&constraint.v, 0);
        let mut visited = HashSet::new();
        let d_u = sol.get_or(&constraint.u, 0);
        let d_v = sol.get_or(&constraint.v, 0);
        while let Some((x, v2x_scaled)) = q.pop() {
            if !visited.insert(x.clone()) {
                continue;
            }
            let d_x = sol.get_or(x, 0);
            let v2x_descaled = v2x_scaled - d_v + d_x;
            let new_val = d_u + constraint.c + v2x_descaled;
            let is_affected = d_x > new_val;
            if !is_affected {
                continue;
            }
            if x == &constraint.u {
                return None;
            }
            new_sol.update(&x.clone(), new_val); // can I get rid of this clone?
            let Some(succesors) = self.feasible_constraints.get(x) else {
                    continue;
            };
            for (y, x2y_unscaled) in succesors.iter() {
                let d_y = sol.get_or(y, 0);
                let x2y_scaled = x2y_unscaled + d_x - d_y;
                let v2y_scaled = v2x_scaled + x2y_scaled;
                q.push_decrease(y, v2y_scaled);
            }
        }
        new_sol.merge(sol);
        Some(new_sol)
    }
    pub fn get_implied_ub(&self, x: &T, y: &T, sol: &Solution<T>) -> Option<i64> {
        // gives the constraint x - y <= a (with smallest possible a) that is implied by the system
        self.dist(y, x, sol)
    }
    pub fn get_implied_lb(&self, x: &T, y: &T, sol: &Solution<T>) -> Option<i64> {
        // gives the constraint x - y >= a (with larget possible a) that is implied by the system.
        // equivalent to y - x <= -a
        self.get_implied_ub(x, y, sol).map(|ub| -ub)
    }
    fn dist(&self, from_node: &T, to_node: &T, sol: &Solution<T>) -> Option<i64> {
        let result = dijkstra(
            from_node,
            |node| self.scaled_succesors(node, sol),
            |node| node == to_node,
        );
        result.map(|(_, cost)| self.descale_dist(cost, from_node, to_node, sol))
    }
    fn scaled_succesors(&self, node: &T, sol: &Solution<T>) -> Vec<(T, i64)> {
        let def = HashMap::new();
        let s = self.feasible_constraints.get(node).unwrap_or(&def);
        let out = s
            .iter()
            .map(|(y, w)| (y.clone(), sol.get_or(node, 0) + w - sol.get_or(y, 0)))
            .collect();
        out
    }
    fn descale_dist(&self, scaled_dist: i64, from_node: &T, to_node: &T, sol: &Solution<T>) -> i64 {
        -sol.get_or(from_node, 0) + scaled_dist + sol.get_or(to_node, 0)
    }
}

impl<T: VarId> Default for DCS<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use rand_chacha::ChaCha8Rng;

    use super::*;
    fn check_multiple_constraints<T: VarId, I: Iterator<Item = (T, T, i64)>>(
        constraints: I,
    ) -> (DCS<T>, Solution<T>) {
        let mut sys = DCS::new();
        let mut sol = Solution::new();
        for (v, u, c) in constraints {
            let constraint = Constraint::new(v, u, c);
            if let Some(new_sol) = sys.add_constraint(&constraint, &sol) {
                assert!(sys.check_solution(&new_sol));
                sol = new_sol;
            } else {
                panic!()
            }
        }
        (sys, sol)
    }

    #[test]
    fn test_check() {
        let x = "x".to_owned();
        let y = "y".to_owned();
        let mut sys = DCS::new();
        let sol = Solution::new();
        let new_sol = sys.add_constraint(&Constraint::new(x, y, 0), &sol);
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

    #[test]
    fn test_get_implied_ub() {
        let (sys, sol) = check_multiple_constraints(
            [("y", "x", 1), ("z", "y", 2), ("x", "z", -3), ("z", "x", 4)].into_iter(),
        );
        assert_eq!(sys.get_implied_ub(&"z", &"x", &sol).unwrap(), 3);
    }

    fn generate_feasible_system(
        num_vars: usize,
        num_constraints: usize,
        seed: u64,
    ) -> Vec<(usize, usize, i64)> {
        use rand::prelude::*;
        use rand::seq::SliceRandom;
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let x: Vec<i64> = (0..num_vars).map(|_| rng.gen_range(0..100)).collect();
        let mut all_constraints = Vec::new();
        for v in 0..num_vars {
            for u in 0..num_vars {
                if u != v {
                    all_constraints.push((v, u));
                }
            }
        }
        let mut out: Vec<(usize, usize, i64)> = all_constraints
            .iter()
            .choose_multiple(&mut rng, num_constraints)
            .into_iter()
            .map(|(v, u)| (*v, *u, x[*v] - x[*u]))
            .collect();
        out.shuffle(&mut rng);
        out
    }

    #[test]
    fn test_random_system() {
        for num_vars in 2..10 {
            for seed in 0..100 {
                let num_constraints = num_vars * (num_vars - 1);
                check_multiple_constraints(
                    generate_feasible_system(num_vars, num_constraints, seed).into_iter(),
                );
            }
        }
    }

    #[test]
    fn test_push_decrease() {
        let mut q = PriorityQueue::new();
        q.push("x", 1);
        q.push("y", 10);
        q.push_decrease("x", 2);
        assert_eq!(q.get_priority("x"), Some(&1));
        assert_eq!(q.get_priority("y"), Some(&10));

        let mut q = PriorityQueue::new();
        q.push("x", 1);
        q.push("y", 10);
        q.push_decrease("x", -5);
        assert_eq!(q.get_priority("x"), Some(&(-5)));
        assert_eq!(q.get_priority("y"), Some(&10));
    }
}
