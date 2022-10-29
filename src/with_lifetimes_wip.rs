use std::{collections::HashMap, cmp::min};

use pathfinding::{prelude::{astar, bfs, dijkstra_all}, directed::dijkstra::dijkstra};


///  x - y <= c
// struct Constraint{
//     x: String,
//     y: String,
//     c: i32,
// }
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum Node{
    Source,
    Variable(usize),
}

struct Constraint<'a>{
    u: &'a Node,
    v: &'a Node,
    a: i64,
}
struct DCS<'a>{
    // graph: DiGraphMap<Node, i64>,
    constraints: HashMap<&'a Node, Vec<Constraint<'a>>>
    // succesors: HashMap<Node, Vec<(Node, i64)>>,
}

#[derive(Debug)]
struct Solution<'a>(HashMap<&'a Node, i64>);

impl <'a>Solution<'a> {
    fn new() -> Solution<'a> {
        let mut map = HashMap::new();
        map.insert(&Node::Source, 0);
        Solution(map)
    }
    fn update(&mut self, node: &'a Node, val: i64) {
        self.0.insert(node, val);
    }

    fn add_var_if_missing(&mut self, node: &'a Node) {
        if !self.0.contains_key(&node){
            self.update(node, 0);
        };
    }
}
impl <'a> DCS<'a>{
    fn new() -> Self {
        let mut constraints = HashMap::new();
        constraints.insert(Node::Source, vec![]);
        DCS{
            constraints: constraints, 
        }
    }
    fn is_variable(&self, x: &Node) -> bool {
        // self.graph.contains_node(Node::Variable(x))
        self.succesors.contains_key(x)
    }
    fn add_unconstrained_variable(&mut self, x: &Node) {
        if !self.is_variable(&x) {
            // self.graph.add_edge(Node::Source, Node::Variable(x), 0);
            self.succesors.get_mut(&Node::Source).expect("error").push((x.clone(), 0));
        }
    }
    fn scaled_succesors(&self, node: &Node, sol: &Solution) -> Vec<(Node, i64)> {
        println!("getting scaled succesors for {:?}", node);
        let def = vec![];
        let s = self.succesors.get(node).unwrap_or(&def);
        let out: Vec<(Node, i64)> = s.into_iter().map(
            |(y, w)| {
                (
                    y.clone(),
                    sol.0[node] + w - sol.0[y]
                )
            }
        ).collect();    
        println!("\tsuccesors of {:?} are {:?}", node, out);
        out
    }
    fn descale_dist(&self, scaled_dist: i64, from_node: &Node, to_node: &Node, sol: &Solution) -> i64 {
        -sol.0[from_node] + scaled_dist + sol.0[to_node]
    }
    fn dist(self: &Self, from_node: &Node, to_node: &Node, sol: &Solution) -> Option<i64> {
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
    fn all_dists_from(&self, from_node: &Node, sol: &Solution, include_source: bool) -> HashMap<Node, i64> {
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
    fn get_implied_ub(&self, x: &Node, y: &Node, sol: &Solution) -> Option<i64> {
        // gives the constraint x - y <= a (with smallest possible a) that is implied by the system
       self.dist(
        x,
        y,
        sol,
       )
    }
    fn get_implied_lb(&self, x: &Node, y: &Node, sol: &Solution) -> Option<i64> {
        // gives the constraint x - y >= a (with larget possible a) that is implied by the system.
        // equivalent to y - x <= -a
        if let Some(val) = self.get_implied_ub(y, x, sol){
            Some(-val)
        } else {None}
    }
    fn check_new_constraint(&self, x: &Node, y: &Node, a: i64, sol: &Solution) -> bool {
        if !self.is_variable(x) || !self.is_variable(y) {
            return true
        };
        // todo: check if the constraint is satisfied by the current solution, and return early if so
         match self.get_implied_lb(x, y, sol) {
            Some(lb) => {lb <= a},
            None => true
         }
    }
    fn update_solution(&self, v: &Node, u: &Node, a: i64, sol: &mut Solution) {
        // let d_from_source = self.all_dists_from(&Node::Source, false);
        let d_from_v = self.all_dists_from(v, sol, true);
        let s2u = sol.0[u];
        // let d_from_x = self.all_dists(&Node::Variable(x));
        // let d: HashMap<_, _> = sol.0.iter()
        //     .map(|(x, s2x)| {
        //         let mut new_val = s2x.clone();
        //         if let Some(v2x) = d_from_v.get(x) {
        //             new_val = min(new_val, s2u + a + v2x);
        //         }
        //         (x.clone(), new_val)
        //     }).collect();
        // Solution(d)
        // let mut new_sol = Solution::new();
        for (x, s2x) in sol.0.iter_mut() {
            // let mut new_val = s2x.clone();
            if let Some(v2x) = d_from_v.get(x) {
                let new_val = s2u + a + v2x;
                if new_val < *s2x {
                    *s2x = new_val
                }
                // let val = min(*s2x, s2u + a + v2x);
            }
            // new_sol.update(x.clone(), new_val);
        };
        // new_sol
    }
    fn add_constraint_helper(&mut self, x: usize, y: usize, a: i64, sol: &'a mut Solution<'a>) -> bool {
        self.add_constraint(&Node::Variable(x), &Node::Variable(y), a, sol)
    }
    fn add_constraint(&mut self, x: &'a Node, y: &'a Node, a: i64, sol: &'a mut Solution<'a>) -> bool{
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
    println!("{}", sys.add_constraint_helper(1, 2, -1, &mut sol));
    println!("{}", sys.add_constraint_helper(2, 3, -2, &mut sol));
    println!("{}", sys.add_constraint_helper(3, 1, 3, &mut sol));
    println!("{}", sys.add_constraint_helper(2, 1, -3, &mut sol));
    println!("{:?}", sol);
}
