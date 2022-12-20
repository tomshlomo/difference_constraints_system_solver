use feasible_system::FeasibleSystem;

use std::collections::HashSet;

mod common;
use common::{Constraint, ConstraintPriority, VarId};
mod solution;
use solution::Solution;
mod feasible_system;
mod prioritized_multi_constraints;
use prioritized_multi_constraints::PrioritizedMulticConstraints;
pub enum Status {
    Feasible,
    Infeasible,
    Undetermined,
    // undetermined_constraints: &'a Vec<Constraint<T>>,
    // undetermined_constraint: &'a Constraint<T>,
}

pub struct DCS<T: VarId, P: ConstraintPriority> {
    feasible_subsystem: FeasibleSystem<T, P>,
    infeasible_constraints: PrioritizedMulticConstraints<T, P>,
    undetermined_constraints: PrioritizedMulticConstraints<T, P>,
}

impl<T: VarId, P: ConstraintPriority> DCS<T, P> {
    pub fn new() -> Self {
        DCS {
            feasible_subsystem: FeasibleSystem::new(),
            infeasible_constraints: PrioritizedMulticConstraints::new(),
            undetermined_constraints: PrioritizedMulticConstraints::new(),
        }
    }
    pub fn is_fully_determined(&self) -> bool {
        self.undetermined_constraints.is_empty()
    }
    pub fn status(&self) -> Status {
        if !self.infeasible_constraints.is_empty() {
            return Status::Infeasible;
        }
        // let Some(undetermined_constraint) = self.undetermined_constraints.get(0) else {
        if self.undetermined_constraints.is_empty() {
            return Status::Feasible;
        };
        Status::Undetermined
        // undetermined_constraints: &self.undetermined_constraints,
        // undetermined_constraint,
    }

    // pub fn from_scratch<It>(constraints: It) -> (Self, Solution<T>)
    // where
    //     It: Iterator<Item = Constraint<T>>,
    // {
    //     let mut sys = Self::new();
    //     let mut sol = Solution::new();
    //     for constraint in constraints {
    //         if let Some(new_sol) = sys.add_constraint(constraint, &sol) {
    //             sol = new_sol;
    //         };
    //     }
    //     (sys, sol)
    // }
    // pub fn feasible_constraints(&self) -> impl Iterator<Item = Constraint<T>> + '_ {
    //     self.feasible_subsystem.constraints()
    // }
    // pub fn all_constraints(&self) -> impl Iterator<Item = Constraint<T>> + '_ {
    //     self.feasible_constraints()
    //         .chain(self.infeasible_constraints.clone().into_iter())
    //         .chain(self.undetermined_constraints.cl)
    // }
    pub fn check_solution(&self, sol: &Solution<T>) -> bool {
        // for constraint in self.all_feasible_constraints() {
        //     if !sol.check_constraint(&constraint) {
        //         return false;
        //     }
        // }
        // true
        match self.status() {
            Status::Feasible => self.feasible_subsystem.check_solution(sol),
            Status::Infeasible => false,
            Status::Undetermined { .. } => {
                self.undetermined_constraints.check_solution(sol)
                    && self.feasible_subsystem.check_solution(sol)
            }
        }
    }
    // fn add_to_feasible(&mut self, constraint: Constraint<T>) {
    //     self.feasible_constraints.add(constraint);
    // }
    // fn add_to_infeasible(&mut self, constraint: Constraint<T>) {
    // self.infeasible_constraints.add(constraint);
    // }
    pub fn add_constraint(
        &mut self,
        constraint: Constraint<T, P>,
        // sol: &Solution<T>,
        // ) -> Option<Solution<T>> {
    ) {
        self.undetermined_constraints.push(constraint);
        // let new_sol = self.check_and_solve_new_constraint(&constraint, sol);
        // match new_sol {
        //     Some(_) => self.add_to_feasible(constraint),
        //     None => self.add_to_infeasible(constraint),
        // }
        // new_sol
    }
    pub fn solve(&mut self) {
        while self.infeasible_constraints.is_empty() {
            let Some(undetermined_multi_constraint) = self.undetermined_constraints.pop() else {
                return
            };
            if !self
                .feasible_subsystem
                .attempt_add_multi_constraint(&undetermined_multi_constraint)
            // todo: avoid this clone
            {
                self.infeasible_constraints.add_multi_constraint(constraint);
                return;
            }
        }
    }
    pub fn remove_constraint(&mut self, constraint: Constraint<T, P>) {
        panic!();
        if self.infeasible_constraints.remove(&constraint) {
            return;
        }
        // if self.undetermined_constraints.contains(&)
    }
}
impl<T: VarId, P: ConstraintPriority> Default for DCS<T, P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: VarId, P: ConstraintPriority> FromIterator<Constraint<T, P>> for DCS<T, P> {
    fn from_iter<I: IntoIterator<Item = Constraint<T, P>>>(iter: I) -> Self {
        let mut system = DCS::new();
        for constraint in iter {
            system.add_constraint(constraint);
        }
        system
    }
}

#[cfg(test)]
mod tests {
    use rand_chacha::ChaCha8Rng;

    use super::*;

    type MyConstraint = Constraint<usize>;
    type MyConstraints = Vec<MyConstraint>;
    type MySol = Solution<usize>;

    fn as_constraints<T: VarId, I: IntoIterator<Item = (T, T, i64)>>(
        tuples: I,
    ) -> Vec<Constraint<T>> {
        tuples
            .into_iter()
            .map(|(v, u, c)| Constraint { v, u, c })
            .collect()
    }
    fn expect_feasible<T: VarId>(constraints: Vec<Constraint<T>>) {
        let mut sys: DCS<T> = constraints.clone().into_iter().collect();
        sys.solve();
        assert!(matches!(sys.status(), Status::Feasible));
        assert!(sys
            .feasible_subsystem
            .solution
            .check_constraints(constraints.iter())
            .is_none());
    }
    fn expect_feasible_with_inner_checks<T: VarId>(constraints: Vec<Constraint<T>>) {
        let mut sys = DCS::new();
        for (i, constraint) in constraints.iter().enumerate() {
            sys.add_constraint(constraint.clone());
            sys.solve();
            assert!(matches!(sys.status(), Status::Feasible));
            assert!(sys
                .feasible_subsystem
                .solution
                .check_constraints(constraints[..i + 1].iter())
                .is_none());
        }
    }
    fn expect_infeasible<T: VarId>(constraints: Vec<Constraint<T>>) -> DCS<T> {
        let mut sys: DCS<T> = constraints.into_iter().collect();
        sys.solve();
        assert!(matches!(sys.status(), Status::Infeasible));
        let feasible_constraints: Vec<_> = sys.feasible_subsystem.constraints().collect();
        assert!(sys
            .feasible_subsystem
            .solution
            .check_constraints(feasible_constraints.iter())
            .is_none());
        sys
    }
    #[test]
    fn test_single_constraint() {
        expect_feasible(vec![Constraint {
            v: "x",
            u: "y",
            c: 0,
        }]);
    }

    #[test]
    fn test_simple_feasible() {
        let x = "x";
        let y = "y";
        let z = "z";
        expect_feasible_with_inner_checks(as_constraints([
            (y, x, 1),
            (z, y, 2),
            (x, z, -3),
            (z, x, 4),
            (z, x, 4),
            (z, x, 4),
            (x, z, -2),
            (x, z, -2),
        ]));
    }

    // fn shrink_constraints<T: VarId, It: Iterator<Item = Constraint<T>>>(
    //     constraints: It,
    // ) -> Vec<Constraint<T>> {
    //     let mut x: HashMap<(T, T), (i64)> = HashMap::new();
    //     for constraint in constraints {
    //         let key = (constraint.v, constraint.u);
    //         let val_to_insert = constraint.c;
    //         if let Some((c, _)) = x.get(&key) {
    //             if val_to_insert.0 > *c {
    //                 continue;
    //             }
    //         }
    //         x.insert(key, val_to_insert);
    //     }
    //     x.iter()
    //         .map(|((v, u), (c))| Constraint {
    //             v: v.clone(),
    //             u: u.clone(),
    //             c: *c,
    //         })
    //         .collect()
    // }
    #[test]
    fn test_get_implied_ub() {
        let mut sys: DCS<_> = as_constraints(
            [("y", "x", 1), ("z", "y", 2), ("x", "z", -3), ("z", "x", 4)].into_iter(),
        )
        .into_iter()
        .collect();
        sys.solve();
        assert_eq!(
            sys.feasible_subsystem.get_implied_ub(&"z", &"x").unwrap(),
            3
        );
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
            .map(|(u, c)| Constraint { v: u + 1, u, c: *c })
            .collect();
        let infeasibility: i64 = rng.gen_range(1..10);
        let path_length: i64 = x.iter().sum();
        constraints.push(Constraint {
            v: 0,
            u: cycle_size - 1,
            c: -path_length - infeasibility,
        });
        constraints
    }

    #[test]
    fn test_random_infeasible_cycles() {
        for num_vars in 2..10 {
            for seed in 0..100 {
                let constraints = generate_random_infeasible_cycle(num_vars, seed);
                expect_infeasible(constraints);
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

        let (mut constraints, _) = generate_random_feasible_constraints(
            num_vars,
            num_feasible_constraints,
            num_dup_constraints,
            seed,
        );
        let infeasible_constraints =
            generate_random_infeasible_cycle(num_infeasible_constraints, seed);
        // let mut constraints = shrink_constraints(
        //     feasible_constraints
        //         .into_iter()
        //         .chain(infeasible_constraints.clone().into_iter()),
        // );
        constraints.append(&mut infeasible_constraints.clone());
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
                        let mut sys = expect_infeasible(all_constraints);

                        // todo: uncomment after reimplementing remove constraints
                        // let sol = sys.remove_constraints(infeasible_constraints.into_iter(), &sol);
                        // assert!(sys.is_feasible());
                        // assert!(sys.check_solution(&sol));
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
                    println!("{}, {}, {}", num_vars, num_dup_constraints, seed);
                    expect_feasible_with_inner_checks(constraints);
                }
            }
        }
    }

    #[test]
    fn test_infeasible_system() {
        let constraints = vec![
            Constraint { v: 0, u: 1, c: 40 },
            Constraint { v: 0, u: 1, c: 41 },
            Constraint { v: 2, u: 1, c: 6 },
            Constraint { v: 2, u: 1, c: 6 },
            Constraint { v: 2, u: 1, c: 6 },
            Constraint { v: 0, u: 2, c: -60 },
            Constraint { v: 1, u: 0, c: -40 },
            Constraint { v: 1, u: 0, c: -40 },
        ];
        expect_infeasible(constraints);
    }

    #[test]
    fn test_remove_constraint() {
        let constraints = [
            Constraint { v: 0, u: 1, c: 40 },
            Constraint { v: 2, u: 1, c: 6 },
            Constraint { v: 0, u: 2, c: -60 },
            Constraint { v: 1, u: 0, c: -40 },
        ];
        let sys: DCS<_> = constraints.clone().into_iter().collect();
        // sys.remove_constraint(constraints[3].clone(), &sol);
        // assert!(sys.is_feasible());
        // assert!(sys.check_solution(&sol));
        // sys.add_constraint(
        //     Constraint {
        //         v: 1,
        //         u: 0,
        //         c: -30,
        //
        //     },
        //     &sol,
        // );
        // assert!(!sys.is_feasible());
        // assert!(sys.check_solution(&sol));
        // sys.remove_constraint(constraints[2].clone(), &sol);
        // assert!(sys.is_feasible());
        // assert!(sys.check_solution(&sol));
    }
}
