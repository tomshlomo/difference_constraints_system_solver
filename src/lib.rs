use pathfinding::prelude::dijkstra;
use priority_queue::PriorityQueue;
use std::cmp::Reverse;
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
    pub fn is_feasible(&self) -> bool {
        self.infeasible_constraints.is_empty()
    }
    pub fn from_scratch<It>(constraints: It) -> (Self, Solution<T>)
    where
        It: Iterator<Item = Constraint<T>>,
    {
        let mut sys = Self::new();
        let mut sol = Solution::new();
        for constraint in constraints {
            if let Some(new_sol) = sys.add_constraint(&constraint, &sol) {
                sol = new_sol;
            };
        }
        (sys, sol)
    }
    pub fn var_feasible_constraints(&self, u: &T) -> Vec<Constraint<T>> {
        // all constraints of the form v - u <= c, for a given u.
        self.feasible_constraints
            .get(u)
            .unwrap_or(&HashMap::new())
            .iter()
            .map(|(v, c)| Constraint::new(v.clone(), u.clone(), *c))
            .collect()
    }
    pub fn all_feasible_constraints(&self) -> Vec<Constraint<T>> {
        self.feasible_constraints
            .keys()
            .into_iter()
            .flat_map(|var| self.var_feasible_constraints(var))
            .collect()
    }
    pub fn check_solution(&self, sol: &Solution<T>) -> bool {
        for constraint in self.all_feasible_constraints() {
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
        self.infeasible_constraints
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
        let mut q: PriorityQueue<&T, (Reverse<i64>, i64)> = PriorityQueue::new();
        let mut visited = HashSet::new();
        let d_u = sol.get_or(&constraint.u, 0);
        let d_v = sol.get_or(&constraint.v, 0);
        q.push(&constraint.v, (Reverse(0), d_v));
        while let Some((x, (v2x_scaled, d_x))) = q.pop() {
            if !visited.insert(x.clone()) {
                continue;
            }
            let v2x_descaled = v2x_scaled.0 - d_v + d_x;
            let new_val = d_u + constraint.c + v2x_descaled;
            let is_affected = d_x > new_val;
            if !is_affected {
                continue;
            }
            if x == &constraint.u {
                return None;
            }
            new_sol.update(x, new_val);
            let Some(succesors) = self.feasible_constraints.get(x) else {
                    continue;
            };
            for (y, x2y_unscaled) in succesors.iter() {
                let d_y = sol.get_or(y, 0);
                let x2y_scaled = x2y_unscaled + d_x - d_y;
                let v2y_scaled = v2x_scaled.0 + x2y_scaled;
                q.push_increase(y, (Reverse(v2y_scaled), d_y));
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
    use std::cmp::min;

    use rand_chacha::ChaCha8Rng;

    use super::*;

    fn as_constraints<T: VarId, I: Iterator<Item = (T, T, i64)>>(
        tuples: I,
    ) -> impl Iterator<Item = Constraint<T>> {
        tuples.map(|(v, u, c)| Constraint::new(v, u, c))
    }
    fn expect_feasible<T: VarId, It: Iterator<Item = Constraint<T>>>(constraints: It) {
        let (sys, sol) = DCS::from_scratch(constraints);
        assert!(sys.is_feasible());
        assert!(sys.check_solution(&sol));
    }
    fn expect_feasible_with_inner_checks<T: VarId, It: Iterator<Item = Constraint<T>>>(
        constraints: It,
    ) {
        let mut sys = DCS::new();
        let mut sol = Solution::new();
        for constraint in constraints {
            sol = sys.add_constraint(&constraint, &sol).unwrap();
            assert!(sys.is_feasible());
            assert!(sys.check_solution(&sol));
        }
    }
    fn expect_infeasible<T: VarId, It: Iterator<Item = Constraint<T>>>(constraints: It) {
        let vec = Vec::from_iter(constraints);
        let (sys, sol) = DCS::from_scratch(vec.clone().into_iter());
        println!("{:#?}", vec);
        assert!(!sys.is_feasible());
        assert!(sys.check_solution(&sol)); // todo: uncomment
    }
    #[test]
    fn test_single_constraint() {
        expect_feasible([Constraint::new("x", "y", 0)].into_iter());
    }

    #[test]
    fn test_simple_feasible() {
        let x = "x";
        let y = "y";
        let z = "z";
        expect_feasible(as_constraints(
            [(y, x, 1), (z, y, 2), (x, z, -3), (z, x, 4)].into_iter(),
        ));
    }

    fn shrink_constraints<T: VarId, It: Iterator<Item = Constraint<T>>>(
        constraints: It,
    ) -> Vec<Constraint<T>> {
        let mut x: HashMap<(T, T), i64> = HashMap::new();
        for constraint in constraints {
            let key = (constraint.v, constraint.u);
            let mut val_to_insert = constraint.c;
            if let Some(c) = x.get(&key) {
                val_to_insert = min(val_to_insert, *c);
            }
            x.insert(key, val_to_insert);
        }
        x.iter()
            .map(|((v, u), c)| Constraint::new(v.clone(), u.clone(), *c))
            .collect()
    }
    #[test]
    fn test_get_implied_ub() {
        let (sys, sol) = DCS::from_scratch(as_constraints(
            [("y", "x", 1), ("z", "y", 2), ("x", "z", -3), ("z", "x", 4)].into_iter(),
        ));
        assert_eq!(sys.get_implied_ub(&"z", &"x", &sol).unwrap(), 3);
    }

    fn generate_random_feasible_constraints(
        num_vars: usize,
        num_constraints: usize,
        seed: u64, // todo: pass rng instead of seed
    ) -> (Vec<Constraint<usize>>, Solution<usize>) {
        use rand::prelude::*;
        use rand::seq::SliceRandom;
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let x: Vec<i64> = (0..num_vars).map(|_| rng.gen_range(0..100)).collect();
        let sol = x.clone().into_iter().enumerate().collect();
        let mut all_constraints = Vec::new();
        for v in 0..num_vars {
            for u in 0..num_vars {
                if u != v {
                    all_constraints.push((v, u));
                }
            }
        }
        let mut out: Vec<Constraint<usize>> = all_constraints
            .iter()
            .choose_multiple(&mut rng, num_constraints)
            .into_iter()
            .map(|(v, u)| Constraint::new(*v, *u, x[*v] - x[*u]))
            .collect();
        out.shuffle(&mut rng);
        (out, sol)
    }

    fn generate_random_infeasible_cycle(cycle_size: usize, seed: u64) -> Vec<Constraint<usize>> {
        // todo: pass rng instead of seed
        use rand::prelude::*;
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let x: Vec<i64> = (0..cycle_size - 1).map(|_| rng.gen_range(0..100)).collect();
        let mut constraints: Vec<Constraint<usize>> = x
            .iter()
            .enumerate()
            .map(|(u, c)| Constraint::new(u + 1, u, *c))
            .collect();
        let infeasibility: i64 = rng.gen_range(1..10);
        let path_length: i64 = x.iter().sum();
        constraints.push(Constraint::new(
            0,
            cycle_size - 1,
            -path_length - infeasibility,
        ));
        constraints
    }

    #[test]
    fn test_random_infeasible_cycles() {
        for num_vars in 2..10 {
            for seed in 0..100 {
                let constraints = generate_random_infeasible_cycle(num_vars, seed);
                expect_infeasible(constraints.into_iter());
            }
        }
    }

    fn generate_random_infeasible_system(
        num_vars: usize,
        num_feasible_constraints: usize,
        num_infeasible_constraints: usize,
        seed: u64, // todo: pass rng instead of seed
    ) -> Vec<Constraint<usize>> {
        use rand::prelude::*;
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        let (feasible_constraints, _) =
            generate_random_feasible_constraints(num_vars, num_feasible_constraints, seed);
        let infeasible_constraints =
            generate_random_infeasible_cycle(num_infeasible_constraints, seed);
        let mut constraints = shrink_constraints(
            feasible_constraints
                .into_iter()
                .chain(infeasible_constraints.into_iter()),
        );
        constraints.shuffle(&mut rng);
        println!("{:?}", constraints);
        constraints
    }

    #[test]
    fn test_random_infeasible_system() {
        for num_vars in 2..10 {
            for num_feasible_constraints in 0..(num_vars * (num_vars - 1) + 1) {
                for num_infeasible_constraints in 2..(num_vars + 1) {
                    for seed in 0..3 {
                        let constraints = generate_random_infeasible_system(
                            num_vars,
                            num_feasible_constraints,
                            num_infeasible_constraints,
                            seed,
                        );
                        println!("{:?}", constraints);
                        expect_infeasible(constraints.into_iter());
                    }
                }
            }
        }
    }
    // fn add_constraints_to_make_infeasible(
    //     feasible_constraints: Vec<Constraint<usize>>,
    //     constraints_to_add: usize,
    //     seed: u64,
    // ) -> Vec<Constraint<usize>> {
    //     let (sys, sol) = DCS::from_scratch(feasible_constraints.into_iter());
    //     assert!(sys.is_feasible());

    // }
    #[test]
    fn test_random_feasible_system() {
        for num_vars in 2..10 {
            for seed in 0..100 {
                let num_constraints = num_vars * (num_vars - 1);
                let (constraints, _) =
                    generate_random_feasible_constraints(num_vars, num_constraints, seed);
                expect_feasible_with_inner_checks(constraints.into_iter());
            }
        }
    }

    #[test]
    fn test_infeasible_system() {
        let constraints = [
            Constraint { v: 0, u: 1, c: 40 },
            Constraint { v: 2, u: 1, c: 6 },
            Constraint { v: 0, u: 2, c: -60 },
            Constraint { v: 1, u: 0, c: -40 },
        ];
        expect_infeasible(constraints.into_iter());
    }
}
