use std::{collections::HashMap, cmp::min};
use std::hash::Hash;
use std::fmt::Debug;
use pathfinding::{prelude::{astar, bfs, dijkstra_all}, directed::dijkstra::dijkstra};


pub trait VarId: Eq + Hash + Debug {}
impl<T> VarId for T where T: Eq + Hash + Debug {}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum Node<'a, T> where T: VarId {
    Source,
    Variable(&'a T),
}
struct DCS<'a, T> where T: VarId{
    // graph: DiGraphMap<Node, i64>,
    succesors: HashMap<Node<'a, T>, Vec<(Node<'a, T>, i64)>>,
}

#[derive(Debug)]
struct Solution<'a, T: VarId>(HashMap<Node<'a, T>, i64>);

impl <'a, T: VarId> Solution<'a, T> {
    fn new() -> Solution<'a, T> {
        let mut map = HashMap::new();
        map.insert(Node::Source, 0);
        Solution(map)
    }
    fn update(&mut self, node: Node<'a, T>, val: i64) {
        self.0.insert(node, val);
    }
    fn add_var_if_missing(&mut self, var: &'a T) {
        let node = Node::Variable(var);
        if !self.0.contains_key(&node){
            self.update(node, 0);
        };
    }
}
impl  <'a, T: VarId> DCS<'a, T>{
    fn new() -> Self {
        let mut succesors = HashMap::new();
        succesors.insert(Node::Source, vec![]);
        DCS{
            succesors: succesors, 
        }
    }
    fn is_variable(&self, x: &Node<'a, T>) -> bool {
        // self.graph.contains_node(Node::Variable(x))
        self.succesors.contains_key(x)
    }
    fn add_unconstrained_variable(&mut self, x: Node<'a, T>) {
        if !self.is_variable(&x) {
            // self.graph.add_edge(Node::Source, Node::Variable(x), 0);
            self.succesors.get_mut(&Node::Source).expect("error").push((x, 0));
        }
    }
    fn scaled_succesors(&self, node: &Node<'a, T>, sol: &Solution<'a, T>) -> Vec<(Node<'a, T>, i64)> {
        println!("getting scaled succesors for {:?}", node);
        let def = vec![];
        let s = *self.succesors.get(node).unwrap_or(&def);
        let out: Vec<(Node<'a, T>, i64)> = s.into_iter().map(
            |(y, w)| {
                (
                    y,
                    sol.0[node] + w - sol.0[&y]
                )
            }
        ).collect();    
        println!("\tsuccesors of {:?} are {:?}", node, out);
        out
    }
    fn descale_dist(&self, scaled_dist: i64, from_node: &Node<'a, T>, to_node: &Node<'a, T>, sol: &Solution<'a, T>) -> i64 {
        -sol.0[from_node] + scaled_dist + sol.0[to_node]
    }  // todo: not should be static or method of solution

    fn dist(self: &Self, from_node: &Node<'a, T>, to_node: &Node<'a, T>, sol: &Solution<'a, T>) -> Option<i64> {
        println!("calcing dist between {:?} and {:?}", from_node, to_node);

        let result = dijkstra(
            from_node,
            |node| {
                self.scaled_succesors(node, sol)
            },
            |node| {node == to_node},
        );
        match result {
            Some((_, cost)) => {
                println!("cost before augmentation: {cost}");
                Some(self.descale_dist(cost, from_node, to_node, sol))
            },
            None => {
                println!("no path found between {:?} and {:?}", from_node, to_node);
                None
            },
        }
    }
    fn all_dists_from(&self, from_node: &Node<'a, T>, sol: &Solution<'a, T>, include_source: bool) -> HashMap<Node<'a, T>, i64> {
        let result = dijkstra_all(
            from_node,
            |node| {
                self.scaled_succesors(node, sol)
            }
        );
        let mut r: HashMap<Node, i64> = result.into_iter()
            .map(
                |(node, (_, scaled_dist))| {
                    (
                        node,
                        self.descale_dist(scaled_dist, &Node::Source, &node, sol),
                    )
                }
            ).collect();
        if include_source {
            r.insert(from_node.clone(), 0);
        }
        r
    }
    fn get_implied_ub(&self, x: &Node<'a, T>, y: &Node<'a, T>, sol: &Solution<'a, T>) -> Option<i64> {
        // gives the constraint x - y <= a (with smallest possible a) that is implied by the system
       self.dist(
        x,
        y,
        sol,
       )
    }
    fn get_implied_lb(&self, x: &Node<'a, T>, y: &Node<'a, T>, sol: &Solution<'a, T>) -> Option<i64> {
        // gives the constraint x - y >= a (with larget possible a) that is implied by the system.
        // equivalent to y - x <= -a
        if let Some(val) = self.get_implied_ub(y, x, sol){
            Some(-val)
        } else {None}
    }
    fn check_new_constraint(&self, x: &Node<'a, T>, y: &Node<'a, T>, a: i64, sol: &Solution<'a, T>) -> bool {
        if !self.is_variable(x) || !self.is_variable(y) {
            return true
        };
        // todo: check if the constraint is satisfied by the current solution, and return early if so
         match self.get_implied_lb(x, y, sol) {
            Some(lb) => {lb <= a},
            None => true
         }
    }
    fn update_solution(&self, v: &Node<'a, T>, u: &Node<'a, T>, a: i64, sol: &mut Solution<'a, T>) {
        let d_from_v = self.all_dists_from(v, sol, true);
        let s2u = sol.0[u];
        for (x, s2x) in sol.0.iter_mut() {
            if let Some(v2x) = d_from_v.get(x) {
                let new_val = s2u + a + v2x;
                if new_val < *s2x {
                    *s2x = new_val
                }
            }
        };
    }
    fn add_constraint_helper(&mut self, x: &T, y: &T, a: i64, sol: &mut Solution<'a, T>) -> bool {
        self.add_constraint(&Node::Variable(x), &Node::Variable(y), a, sol)
    }
    fn add_constraint(&mut self, x: &Node<'a, T>, y: &Node<'a, T>, a: i64, sol: &mut Solution<'a, T>) -> bool{
        // x - y  <= a
        println!("adding constraint {:?} - {:?} <= {:?}", x, y, a);
        self.add_unconstrained_variable(x);
        self.add_unconstrained_variable(y);
        sol.add_var_if_missing(x);
        sol.add_var_if_missing(y);
        // new constraint would add the edge y -> x with weight a
        if self.check_new_constraint(&x, &y, a, sol) {
            self.update_solution(&x, &y, a, sol);
            self.succesors.entry(y.clone()).or_insert(vec![]).push((x.clone(), a));
            println!(
                "after inserting constraint {:?} - {:?} <= {}, succesrs of {:?} are: {:?}",
                x, y, a, y, self.succesors.get(&y),
            );
            // self.update_solution();
            // self.succesors.get_mut(&Node::Variable(y)).expect("error!").push((Node::Variable(x), a));
            true
        } else {
            println!("constraint {:?} - {:?} <= {} could not be added", x, y, a);
            false
        }
    }
}
fn main() {
    // let mut graph = DiGraphMap::<&str, i64>::new();
    // solution:
    // a = 0
    // b = 1
    // c = 3
    // constraints:
    // b >= a + 1 -> a - b <= -1
    // c >= b + 2 -> b - c <= -2
    // c <= a + 3 -> c - a <= 3
    let mut sys = DCS::new();
    let mut sol = Solution::new();
    println!("{}", sys.add_constraint_helper(&1, &2, -1, &mut sol));
    println!("{}", sys.add_constraint_helper(&2, &3, -2, &mut sol));
    println!("{}", sys.add_constraint_helper(&3, &1, 3, &mut sol));
    println!("{}", sys.add_constraint_helper(&2, &1, -3, &mut sol));
    println!("{:?}", sol);
}
