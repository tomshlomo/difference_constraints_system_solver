use std::{cmp::min, collections::HashMap};

use pathfinding::{
    directed::dijkstra::dijkstra,
    prelude::{astar, bfs, dijkstra_all},
};
use std::hash::Hash;

trait VarId: Eq + Hash {}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
enum Node<'a, T: VarId> {
    Source,
    Variable(&'a T),
}
struct DCS<'a, T: VarId> {
    // graph: DiGraphMap<Node, i64>,
    succesors: HashMap<Node<'a, T>, Vec<(Node<'a, T>, i64)>>,
}

#[derive(Debug)]
struct Solution<'a, T: VarId>(HashMap<Node<'a, T>, i64>);

impl<'a, T: VarId> Solution<'a, T> {
    fn new() -> Self {
        let mut map = HashMap::new();
        map.insert(Node::Source, 0);
        Solution(map)
    }
    fn update(&mut self, node: Node<'a, T>, val: i64) {
        self.0.insert(node, val);
    }

    fn get(&self, node: &Node<'a, T>) -> i64 {
        self.0.get(node).unwrap().clone()
    }
    fn add_var_if_missing(&mut self, node: Node<'a, T>) {
        if !self.0.contains_key(&node) {
            self.update(node, 0);
        };
    }
}

struct Constraint<'a, T: VarId> {
    // u - v <= a
    u: Node<'a, T>,
    v: Node<'a, T>,
    a: i64,
}

impl<'a, T: VarId> Constraint<'a, T> {
    fn new(u: &'a T, v: &'a T, a: i64) -> Self {
        Constraint {
            u: Node::Variable(u),
            v: Node::Variable(v),
            a: a,
        }
    }
    fn new_source(v: &'a T) -> Self {
        Constraint {
            u: Node::Source,
            v: Node::Variable(v),
            a: 0,
        }
    }
    fn check(&self, sol: &Solution<'a, T>) -> bool {
        sol.get(&self.u) <= sol.get(&self.v) + self.a
    }
}

struct System<'a, T: VarId> (
    HashMap<&'a T, HashMap<&'a T, i64>>
);
    // constraints: HashMap<Node<'a, T>>>
    // constraints: HashMap<Node<'a, T>, Vec<Constraint<'a, T>>>,
// }

impl<'a, T: VarId> System<'a, T> {
    fn new() -> Self {
        // let mut d = HashMap::new();
        // d.insert(Node::Source, vec![]);
        // System { constraints: d }
        System(HashMap::new())
    }
    fn is_variable(&self, var: &T) -> bool {
        self.0.contains_key(var)
    }
    fn add_var_if_missing(&mut self, var: &'a T) {
        if self.is_variable(var) {
            return
        }
        self.0.insert(var, HashMap::new());
        // let constraint_from_source = Constraint::new_source(var);
        // self.constraints.get_mut(&Node::Source).unwrap().push(
        //     constraint_from_source
        // );
    }
    fn check(&self, sol: &Solution<'a, T>) -> bool {
        for (u, v2a) in self.0.into_iter() {
            for (v, a) in v2a.into_iter() {
                if sol.get(u) > sol.get(v) + a {
                    return false
                }
            }
        }
        true
    }
    // pub fn new<It>(it: It) -> Self
    // where
    //     It: IntoIterator<Item = &'a Constraint<'a, T>>,
    // {
    //     let mut constraints = HashMap::new();
    //     for constraint in it.into_iter() {
    //         let v = constraints.entry(constraint.u).or_insert(vec![]).append(constraint);
    //     }
    //     System {
    //         constraints: constraints,
    //     }
    // }
}

fn main() {
    // solution:
    // a = 0
    // b = 1
    // c = 3
    // constraints:
    // b >= a + 1 -> a - b <= -1
    // c >= b + 2 -> b - c <= -2
    // c <= a + 3 -> c - a <= 3
}
