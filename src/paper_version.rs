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
                return false;
            }
        }
        true
    }
    fn add_succesor(&mut self, from_var: &T, to_variable: &T, c: i64) {
        self.succesors
            .entry(from_var.clone())
            .or_insert(vec![])
            .push((to_variable.clone(), c));
    }
    pub fn add_to_feasible(
        &mut self,
        constraint: &Constraint<T>,
        sol: &Solution<T>,
    ) -> Option<Solution<T>> {
        let mut new_sol = Solution::new();
        let mut q = PriorityQueue::new();
        q.push(constraint.v.clone(), 0);
        let mut visited = HashSet::new();
        let d_u = sol.get_or(&constraint.u, 0);
        while let Some((x, v2x_scaled)) = q.pop() {
            if visited.contains(&x) {
                continue;
            }
            visited.insert(x.clone());
            let v2x_descaled = self.descale_dist(v2x_scaled, &constraint.v, &x, sol);
            let d_x = sol.get_or(&x, 0);
            let is_affected = d_x > d_u + constraint.c + v2x_descaled;
            if is_affected {
                if x == constraint.u {
                    return None;
                }
                new_sol.update(&x, d_u + constraint.c + v2x_descaled);
                for (y, x2y_scaled) in self.scaled_succesors(&x, sol) {
                    q.push_decrease(y, v2x_scaled + x2y_scaled);
                }
            }
        }
        new_sol.merge(sol);
        self.add_succesor(&constraint.u, &constraint.v, constraint.c);
        Some(new_sol)
    }
    pub fn add_to_feasible_verbose(
        &mut self,
        constraint: &Constraint<T>,
        sol: &Solution<T>,
    ) -> Option<Solution<T>> {
        println!("adding constraint {}", constraint);
        println!("current solution is: {:#?}", sol.0);
        self.add_unconstrained_variable(&constraint.v); // are these really necessary?
        self.add_unconstrained_variable(&constraint.u);
        // self.add_succesor(&constraint.u, &constraint.v, constraint.c);
        // if sol.check_constraint(constraint) {
        // disabled this for now since assigning all missing vars to zero is not necearily correct.
        // v - u <= c
        // should probably be something like add_var_id_missing(v, default=sol.get(u) + c)
        //     let mut new_sol = sol.clone();
        //     new_sol.add_var_if_missing(constraint.u.clone());
        //     new_sol.add_var_if_missing(constraint.v.clone());
        //     return Some(new_sol);
        // }
        let mut new_sol = Solution::new();
        // let mut new_sol = sol.clone();
        // new_sol.add_var_if_missing(constraint.u.clone());
        // new_sol.add_var_if_missing(constraint.v.clone());
        let mut q = PriorityQueue::new();
        q.push(constraint.v.clone(), 0);
        let mut visited = HashSet::new();
        let d_u = sol.get_or(&constraint.u, 0);
        while let Some((x, v2x_scaled)) = q.pop() {
            if visited.contains(&x) {
                continue;
            }
            visited.insert(x.clone());
            assert!(v2x_scaled >= 0); // todo: remove the assert
            println!("starting new dijkstra iteration.\nnew node is {:?}, with current solution value = {:?}", x, sol.get_or(&x, 0));
            println!(
                "scaled shortest dist from {:?} to {:?} is {:?}",
                constraint.v, x, v2x_scaled
            );
            let v2x_descaled = self.descale_dist(v2x_scaled, &constraint.v, &x, sol);
            println!(
                "descaled, shortest dist from {:?} to {:?} is {:?}.",
                constraint.v, x, v2x_descaled
            );
            if x == constraint.u {
                println!("current node is the target node {:?}.", constraint.u);
                println!("the system is feasible iff the shortest (descaled) distance from {:?} to {:?} is greater or equal to {:?}.", constraint.v, constraint.u, -constraint.c);
            }
            let d_x = sol.get_or(&x, 0);
            let is_affected = d_x > d_u + constraint.c + v2x_descaled;
            println!("is_affected={}", is_affected);
            if is_affected {
                if x == constraint.u {
                    // self.succesors.get_mut(&constraint.u).unwrap().pop(); // remove the new constraint
                    print!("System is infeasible");
                    return None;
                }
                println!(
                    "updating {:?} from {} to {}",
                    x,
                    d_x,
                    d_u + constraint.c + v2x_descaled
                );
                new_sol.update(&x, d_u + constraint.c + v2x_descaled);
                println!("iterating over succesors of {:?}:", x);
                for (y, x2y_scaled) in self.scaled_succesors(&x, sol) {
                    assert!(x2y_scaled >= 0); // todo: remove the assert
                    println!("scaled dist of {:?} to {:?} is {}", x, y, x2y_scaled);
                    println!("value of {:?} in queue is {:?}", y, q.get_priority(&y));
                    q.push_decrease(y.clone(), v2x_scaled + x2y_scaled); // todo: this clone is just to get the print working
                    println!("new value of {:?} in queue is {:?}", y, q.get_priority(&y));
                }
            }
        }
        new_sol.merge(sol);
        new_sol.add_var_if_missing(constraint.u.clone());
        new_sol.add_var_if_missing(constraint.v.clone());
        println!("new solution is: {:#?}", new_sol.0);
        self.add_succesor(&constraint.u, &constraint.v, constraint.c);
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
        let def = vec![];
        let s = self.succesors.get(node).unwrap_or(&def);
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
            if let Some(new_sol) = sys.add_to_feasible(&constraint, &sol) {
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
