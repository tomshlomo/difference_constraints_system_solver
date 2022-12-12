use pathfinding::prelude::dijkstra;
use priority_queue::PriorityQueue;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::hash::Hash;

pub trait VarId: Eq + Hash + Debug + Clone + Display {}
impl<T> VarId for T where T: Eq + Hash + Debug + Clone + Display {}

pub trait ConstraintTag: Eq + Hash + Debug + Clone {}
impl<C> ConstraintTag for C where C: Eq + Hash + Debug + Clone {}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Constraint<T: VarId, C: ConstraintTag> {
    // v - u <= c
    pub v: T,
    pub u: T,
    pub c: i64,
    pub tag: C,
}

impl<T: VarId, C: ConstraintTag> Display for Constraint<T, C> {
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
    pub fn check_constraint<C: ConstraintTag>(&self, constraint: &Constraint<T, C>) -> bool {
        if let (Some(u), Some(v)) = (self.get(&constraint.u), self.get(&constraint.v)) {
            return v - u <= constraint.c;
        }
        true
    }
    pub fn merge(&mut self, other: &Solution<T>) {
        // todo: can consume other to avoid clones?
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

struct FromEdges<T: VarId, C: ConstraintTag>(HashMap<T, PriorityQueue<(i64, C), Reverse<i64>>>);
impl<T: VarId, C: ConstraintTag> FromEdges<T, C> {
    fn new() -> Self {
        FromEdges(HashMap::new())
    }
    fn is_empty(&self) -> bool {
        self.0.values().all(|a| a.is_empty()) // todo: cahce
    }
    fn to_pairs(&self) -> impl Iterator<Item = (&T, &i64, &C)> + '_ {
        self.0.iter().filter_map(|(var, heap)| {
            // todo: use Option.map instead if match
            if let Some(((val, tag), _)) = heap.peek() {
                Some((var, val, tag))
            } else {
                None
            }
        })
    }
    fn add(&mut self, var: T, val: i64, tag: C) {
        self.0
            .entry(var)
            .or_default()
            .push((val, tag), Reverse(val));
    }
    fn remove(&mut self, var: &T, val: i64, tag: C) -> bool {
        if let Some(heap) = self.0.get_mut(var) {
            return heap.remove(&(val, tag)).is_some();
            // return heap.remove(val);
        };
        false
    }
}
impl<T: VarId, C: ConstraintTag> Default for FromEdges<T, C> {
    fn default() -> Self {
        Self::new()
    }
}
struct Edges<T: VarId, C: ConstraintTag>(HashMap<T, FromEdges<T, C>>);
impl<T: VarId, C: ConstraintTag> Edges<T, C> {
    fn new() -> Self {
        Edges(HashMap::new())
    }
    fn is_empty(&self) -> bool {
        self.0.values().all(|a| a.is_empty()) // todo: cahce
    }
    fn to_constraints(&self) -> impl Iterator<Item = Constraint<T, C>> + '_ {
        self.0.iter().flat_map(|(u, from_edges)| {
            from_edges.to_pairs().map(|(v, c, tag)| Constraint {
                v: v.clone(),
                u: u.clone(),
                c: *c,
                tag: tag.clone(),
            })
        })
    }
    fn add(&mut self, constraint: Constraint<T, C>) {
        self.0
            .entry(constraint.u)
            .or_default()
            .add(constraint.v, constraint.c, constraint.tag);
    }
    fn remove(&mut self, constraint: Constraint<T, C>) -> bool {
        if let Some(from_u) = self.0.get_mut(&constraint.u) {
            return from_u.remove(&constraint.v, constraint.c, constraint.tag);
        };
        false
    }
}
pub struct DCS<T: VarId, C: ConstraintTag> {
    feasible_constraints: Edges<T, C>,
    infeasible_constraints: Edges<T, C>,
}

impl<T: VarId, C: ConstraintTag> DCS<T, C> {
    pub fn new() -> Self {
        DCS {
            feasible_constraints: Edges::new(),
            infeasible_constraints: Edges::new(),
        }
    }
    pub fn is_feasible(&self) -> bool {
        self.infeasible_constraints.is_empty()
    }
    pub fn from_scratch<It>(constraints: It) -> (Self, Solution<T>)
    where
        It: Iterator<Item = Constraint<T, C>>,
    {
        let mut sys = Self::new();
        let mut sol = Solution::new();
        for constraint in constraints {
            if let Some(new_sol) = sys.add_constraint(constraint, &sol) {
                sol = new_sol;
            };
        }
        (sys, sol)
    }
    pub fn all_infeasible_constraints(&self) -> impl Iterator<Item = Constraint<T, C>> + '_ {
        self.infeasible_constraints.to_constraints()
    }
    pub fn all_feasible_constraints(&self) -> impl Iterator<Item = Constraint<T, C>> + '_ {
        self.feasible_constraints.to_constraints()
    }
    pub fn check_solution(&self, sol: &Solution<T>) -> bool {
        for constraint in self.all_feasible_constraints() {
            if !sol.check_constraint(&constraint) {
                return false;
            }
        }
        true
    }
    fn add_to_feasible(&mut self, constraint: Constraint<T, C>) {
        self.feasible_constraints.add(constraint);
    }
    fn add_to_infeasible(&mut self, constraint: Constraint<T, C>) {
        self.infeasible_constraints.add(constraint);
    }
    pub fn add_constraint(
        &mut self,
        constraint: Constraint<T, C>,
        sol: &Solution<T>,
    ) -> Option<Solution<T>> {
        let new_sol = self.check_and_solve_new_constraint(&constraint, sol);
        match new_sol {
            Some(_) => self.add_to_feasible(constraint),
            None => self.add_to_infeasible(constraint),
        }
        new_sol
    }
    pub fn check_and_solve_new_constraint(
        &self,
        constraint: &Constraint<T, C>,
        sol: &Solution<T>,
    ) -> Option<Solution<T>> {
        let mut new_sol = Solution::new();
        let mut q: PriorityQueue<&T, (Reverse<i64>, i64)> = PriorityQueue::new();
        let mut visited = HashSet::new();
        let d_u = sol.get_or(&constraint.u, 0);
        let d_v = sol.get_or(&constraint.v, 0);
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
            new_sol.update(x, new_val);
            let Some(succesors) = self.feasible_constraints.0.get(x) else {
                    continue;
            };
            // equivalent to `for (y, x2y_scaled) in self.scaled_succesors(y, sol)`, but with less lookups.
            for (y, x2y_unscaled, _) in succesors.to_pairs() {
                let d_y = sol.get_or(y, 0);
                let x2y_scaled = x2y_unscaled + d_x - d_y;
                let v2y_scaled = v2x_scaled.0 + x2y_scaled;
                if !visited.contains(y) {
                    q.push_increase(y, (Reverse(v2y_scaled), d_y));
                }
            }
        }
        new_sol.merge(sol);
        Some(new_sol)
    }
    pub fn remove_constraint(
        &mut self,
        constraint_to_remove: Constraint<T, C>,
        sol: &Solution<T>,
    ) -> Solution<T> {
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
        let mut new_sol = sol.clone(); // todo: try not to clone. maybe just consume sol (or mut it)
        if self.remove_from_infeasible(constraint_to_remove.clone()) {
            return new_sol;
        }
        if !self.remove_from_feasible(constraint_to_remove) {
            return new_sol;
        }
        // todo: not a great implemenataion. wroking with constraint objects below seems redundant
        for constraint in self
            .all_infeasible_constraints()
            .collect::<Vec<Constraint<T, C>>>()
        {
            if let Some(new_sol2) = self.check_and_solve_new_constraint(&constraint, &new_sol) {
                new_sol = new_sol2;
                self.remove_from_infeasible(constraint.clone());
                self.add_to_feasible(constraint);
            }
        }
        new_sol
    }
    pub fn remove_constraints<I: Iterator<Item = Constraint<T, C>>>(
        &mut self,
        constraints: I,
        sol: &Solution<T>,
    ) -> Solution<T> {
        let mut new_sol = sol.clone(); // todo: avoid this clone. maybe just consume sol (or mut it)
        for constraint in constraints {
            new_sol = self.remove_constraint(constraint, &new_sol);
        }
        new_sol
    }
    fn remove_from_infeasible(&mut self, constraint: Constraint<T, C>) -> bool {
        self.infeasible_constraints.remove(constraint)
    }
    fn remove_from_feasible(&mut self, constraint: Constraint<T, C>) -> bool {
        self.feasible_constraints.remove(constraint)
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
        // todo: return an iterator instead of Vec
        let Some(from_edges) = self.feasible_constraints.0.get(node) else {
            return vec![]
        };
        let d_node = sol.get_or(node, 0);
        let out = from_edges
            .to_pairs()
            .map(|(y, w, _)| (y.clone(), d_node + w - sol.get_or(y, 0)))
            .collect();
        out
    }
    fn descale_dist(&self, scaled_dist: i64, from_node: &T, to_node: &T, sol: &Solution<T>) -> i64 {
        -sol.get_or(from_node, 0) + scaled_dist + sol.get_or(to_node, 0)
    }
}

impl<T: VarId, C: ConstraintTag> Default for DCS<T, C> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {

    use rand_chacha::ChaCha8Rng;

    use super::*;

    type MyConstraint = Constraint<usize, ()>;
    type MyConstraints = Vec<MyConstraint>;
    type MySol = Solution<usize>;

    fn as_constraints<T: VarId, I: Iterator<Item = (T, T, i64)>>(
        tuples: I,
    ) -> impl Iterator<Item = Constraint<T, ()>> {
        tuples.map(|(v, u, c)| Constraint { v, u, c, tag: () })
    }
    fn expect_feasible<T: VarId, C: ConstraintTag, It: Iterator<Item = Constraint<T, C>>>(
        constraints: It,
    ) {
        let (sys, sol) = DCS::from_scratch(constraints);
        assert!(sys.is_feasible());
        assert!(sys.check_solution(&sol));
    }
    fn expect_feasible_with_inner_checks<
        T: VarId,
        C: ConstraintTag,
        It: Iterator<Item = Constraint<T, C>>,
    >(
        constraints: It,
    ) {
        let mut sys = DCS::new();
        let mut sol = Solution::new();
        for constraint in constraints {
            sol = sys.add_constraint(constraint, &sol).unwrap();
            assert!(sys.is_feasible());
            assert!(sys.check_solution(&sol));
        }
    }
    fn expect_infeasible<T: VarId, C: ConstraintTag, It: Iterator<Item = Constraint<T, C>>>(
        constraints: It,
    ) -> (DCS<T, C>, Solution<T>) {
        let vec = Vec::from_iter(constraints);
        let (sys, sol) = DCS::from_scratch(vec.clone().into_iter());
        println!("{:#?}", vec);
        assert!(!sys.is_feasible());
        assert!(sys.check_solution(&sol)); // todo: uncomment
        (sys, sol)
    }
    #[test]
    fn test_single_constraint() {
        expect_feasible(
            [Constraint {
                v: "x",
                u: "y",
                c: 0,
                tag: (),
            }]
            .into_iter(),
        );
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

    fn shrink_constraints<T: VarId, C: ConstraintTag, It: Iterator<Item = Constraint<T, C>>>(
        constraints: It,
    ) -> Vec<Constraint<T, C>> {
        let mut x: HashMap<(T, T), (i64, C)> = HashMap::new();
        for constraint in constraints {
            let key = (constraint.v, constraint.u);
            let val_to_insert = (constraint.c, constraint.tag);
            if let Some((c, _)) = x.get(&key) {
                if val_to_insert.0 > *c {
                    continue;
                }
            }
            x.insert(key, val_to_insert);
        }
        x.iter()
            .map(|((v, u), (c, tag))| Constraint {
                v: v.clone(),
                u: u.clone(),
                c: *c,
                tag: tag.clone(),
            })
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
        num_dup_constraints: usize,
        seed: u64, // todo: pass rng instead of seed
    ) -> (MyConstraints, MySol) {
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
        let mut out: MyConstraints = all_constraints
            .iter()
            .choose_multiple(&mut rng, num_constraints)
            .into_iter()
            .map(|(v, u)| Constraint {
                v: *v,
                u: *u,
                c: x[*v] - x[*u],
                tag: (),
            })
            .collect();
        out.shuffle(&mut rng);
        for i in 0..num_dup_constraints {
            let constraint = out[i].clone();
            let extra_c = rng.gen_range(0..10);
            out.push(Constraint {
                v: constraint.v,
                u: constraint.u,
                c: constraint.c + extra_c,
                tag: (),
            })
        }
        // for constraint in out[..num_constraints] {}
        (out, sol)
    }

    fn generate_random_infeasible_cycle(cycle_size: usize, seed: u64) -> MyConstraints {
        // todo: pass rng instead of seed
        use rand::prelude::*;
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let x: Vec<i64> = (0..cycle_size - 1).map(|_| rng.gen_range(0..100)).collect();
        let mut constraints: MyConstraints = x
            .iter()
            .enumerate()
            .map(|(u, c)| Constraint {
                v: u + 1,
                u,
                c: *c,
                tag: (),
            })
            .collect();
        let infeasibility: i64 = rng.gen_range(1..10);
        let path_length: i64 = x.iter().sum();
        constraints.push(Constraint {
            v: 0,
            u: cycle_size - 1,
            c: -path_length - infeasibility,
            tag: (),
        });
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
        num_dup_constraints: usize,
        seed: u64, // todo: pass rng instead of seed
    ) -> (MyConstraints, MyConstraints) {
        use rand::prelude::*;
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        let (feasible_constraints, _) = generate_random_feasible_constraints(
            num_vars,
            num_feasible_constraints,
            num_dup_constraints,
            seed,
        );
        let infeasible_constraints =
            generate_random_infeasible_cycle(num_infeasible_constraints, seed);
        let mut constraints = shrink_constraints(
            feasible_constraints
                .into_iter()
                .chain(infeasible_constraints.clone().into_iter()),
        );
        // let mut constraints: MyConstraints = feasible_constraints
        //     .into_iter()
        //     .chain(infeasible_constraints.clone().into_iter())
        //     .collect();
        constraints.shuffle(&mut rng);
        println!("{:?}", constraints);
        (constraints, infeasible_constraints)
    }

    #[test]
    fn test_random_infeasible_system() {
        for num_vars in 2usize..10 {
            for num_feasible_constraints in 0..(num_vars * (num_vars - 1) + 1) {
                for num_infeasible_constraints in 2..(num_vars + 1) {
                    for seed in 0..3 {
                        let num_dup_constraints = (num_feasible_constraints as i64
                            - 2 * seed as i64)
                            .clamp(0, num_feasible_constraints as i64)
                            as usize;
                        let (all_constraints, infeasible_constraints) =
                            generate_random_infeasible_system(
                                num_vars,
                                num_feasible_constraints,
                                num_infeasible_constraints,
                                num_dup_constraints,
                                seed,
                            );
                        println!("{:?}", all_constraints);
                        let (mut sys, sol) = expect_infeasible(all_constraints.into_iter());
                        // let vars = infeasible_constraints
                        //     .iter()
                        //     .map(|constraint| (constraint.v, constraint.u));
                        let sol = sys.remove_constraints(infeasible_constraints.into_iter(), &sol);
                        assert!(sys.is_feasible());
                        assert!(sys.check_solution(&sol));
                    }
                }
            }
        }
    }

    #[test]
    fn test_random_feasible_system() {
        for num_vars in 2..10 {
            for num_dup_constraints in 0..num_vars + 1 {
                for seed in 0..10 {
                    let num_constraints = num_vars * (num_vars - 1);
                    let (constraints, _) = generate_random_feasible_constraints(
                        num_vars,
                        num_constraints,
                        num_dup_constraints,
                        seed,
                    );
                    expect_feasible_with_inner_checks(constraints.into_iter());
                }
            }
        }
    }

    #[test]
    fn test_infeasible_system() {
        let constraints = [
            Constraint {
                v: 0,
                u: 1,
                c: 40,
                tag: (),
            },
            Constraint {
                v: 2,
                u: 1,
                c: 6,
                tag: (),
            },
            Constraint {
                v: 0,
                u: 2,
                c: -60,
                tag: (),
            },
            Constraint {
                v: 1,
                u: 0,
                c: -40,
                tag: (),
            },
        ];
        expect_infeasible(constraints.into_iter());
    }

    #[test]
    fn test_remove_constraint() {
        let constraints = [
            Constraint {
                v: 0,
                u: 1,
                c: 40,
                tag: (),
            },
            Constraint {
                v: 2,
                u: 1,
                c: 6,
                tag: (),
            },
            Constraint {
                v: 0,
                u: 2,
                c: -60,
                tag: (),
            },
            Constraint {
                v: 1,
                u: 0,
                c: -40,
                tag: (),
            },
        ];
        let (mut sys, sol) = DCS::from_scratch(constraints.clone().into_iter());
        sys.remove_constraint(constraints[3].clone(), &sol);
        assert!(sys.is_feasible());
        assert!(sys.check_solution(&sol));
        sys.add_constraint(
            Constraint {
                v: 1,
                u: 0,
                c: -30,
                tag: (),
            },
            &sol,
        );
        assert!(!sys.is_feasible());
        assert!(sys.check_solution(&sol));
        sys.remove_constraint(constraints[2].clone(), &sol);
        assert!(sys.is_feasible());
        assert!(sys.check_solution(&sol));
    }
}
