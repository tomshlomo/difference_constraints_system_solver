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
struct DCS{
    // graph: DiGraphMap<Node, i64>,
    succesors: HashMap<Node, Vec<(Node, i64)>>,
    solution: HashMap<Node, i64>,
}

impl DCS{
    fn new() -> Self {
        let mut solution = HashMap::new();
        solution.insert(Node::Source, 0);
        let mut succesors = HashMap::new();
        succesors.insert(Node::Source, vec![]);
        DCS{
            succesors: succesors, 
            solution: solution,
        }
    }
    fn is_variable(&self, x: usize) -> bool {
        // self.graph.contains_node(Node::Variable(x))
        self.solution.contains_key(&Node::Variable(x))
    }
    fn add_unconstrained_variable(&mut self, x: usize) {
        if !self.is_variable(x) {
            // self.graph.add_edge(Node::Source, Node::Variable(x), 0);
            self.succesors.get_mut(&Node::Source).expect("error").push((Node::Variable(x), 0));
            self.solution.insert(Node::Variable(x), 0);
        }
    }
    fn scaled_succesors(&self, node: &Node) -> Vec<(Node, i64)> {
        println!("getting scaled succesors for {:?}", node);
        let def = vec![];
        let s = self.succesors.get(node).unwrap_or(&def);
        let out: Vec<(Node, i64)> = s.into_iter().map(
            |(y, w)| {
                (
                    y.clone(),
                    self.solution[node] + w - self.solution[y]
                )
            }
        ).collect();    
        println!("\tsuccesors of {:?} are {:?}", node, out);
        out
    }
    fn descale_dist(&self, scaled_dist: i64, from_node: &Node, to_node: &Node) -> i64 {
        -self.solution[from_node] + scaled_dist + self.solution[to_node]
    }
    fn dist(self: &Self, from_node: &Node, to_node: &Node) -> Option<i64> {
        println!("calcing dist between {:?} and {:?}", from_node, to_node);

        let result = dijkstra(
            from_node,
            |node| {
                self.scaled_succesors(node)
            },
            |node| {node == to_node},
        );
        match result {
            Some((_, cost)) => {
                println!("cost before augmentation: {cost}");
                Some(self.descale_dist(cost, from_node, to_node))
            },
            None => {
                println!("no path found between {:?} and {:?}", from_node, to_node);
                None
            },
        }
    }
    fn all_dists_from(&self, from_node: &Node, include_source: bool) -> HashMap<Node, i64> {
        let result = dijkstra_all(
            from_node,
            |node| {
                self.scaled_succesors(node)
            }
        );
        let mut r: HashMap<Node, i64> = result.into_iter()
            .map(
                |(node, (_, scaled_dist))| {
                    (
                        node,
                        self.descale_dist(scaled_dist, &Node::Source, &node),
                    )
                }
            ).collect();
        if include_source {
            r.insert(from_node.clone(), 0);
        }
        r
    }
    fn get_implied_ub(&self, x: usize, y: usize) -> Option<i64> {
        // gives the constraint x - y <= a (with smallest possible a) that is implied by the system
       self.dist(
        &Node::Variable(y),
        &Node::Variable(x),
       )
    }
    fn get_implied_lb(&self, x: usize, y: usize) -> Option<i64> {
        // gives the constraint x - y >= a (with larget possible a) that is implied by the system.
        // equivalent to y - x <= -a
        if let Some(val) = self.get_implied_ub(y, x){
            Some(-val)
        } else {None}
    }
    fn check_new_constraint(&self, x: usize, y: usize, a: i64) -> bool {
        if !self.is_variable(x) || !self.is_variable(y) {
            return true
        };
         match self.get_implied_lb(x, y) {
            Some(lb) => {lb <= a},
            None => true
         }
    }
    // fn get_new_solution_for_node(&self, x: &Node) -> i64 {
    //    let val = self.dist(&Node::Source, x).expect("should be a path");
    //    println!("{:?}: {}", x,val);
    //    val
    // }
    // fn update_solution(&mut self, x: usize, y: usize, a: i64) {
    //     let new_map = HashMap::<Node, i64>::from_iter(
    //         self.solution.keys().into_iter()
    //         .map(|node| {
    //             (node.clone(), self.get_new_solution_for_node(node))
    //         })
    //     );
    //     self.solution = new_map;
    //     // for node in self.solution.keys().into_iter() {
    //     //     self.solution.insert(Node::Variable(1), self.dist(&Node::Source, node).expect("should be a path"));
    //     // }
    // }
    fn update_solution(&self, v: usize, u: usize, a: i64, sol: &mut HashMap<Node, i64>) {
        // let d_from_source = self.all_dists_from(&Node::Source, false);
        let d_from_v = self.all_dists_from(&Node::Variable(v), true);
        let s2u = self.solution[&Node::Variable(u)];
        // let d_from_x = self.all_dists(&Node::Variable(x));
        // let mut new_sol = HashMap::new();
        for (x, s2x) in self.solution.iter() {
            let mut new_val = s2x;
            if let Some(v2x) = d_from_v.get(x) {
                new_val = min(new_val, s2u + a + v2x);
            }
            new_sol.insert(x.clone(), new_val.clone());
        };
        self.solution = new_sol;
    }
    fn add_constraint(&mut self, x: usize, y: usize, a: i64) -> bool{
        // x - y  <= a
        println!("adding constraint {} - {} <= {}", x, y, a);
        self.add_unconstrained_variable(x);
        self.add_unconstrained_variable(y);
        // new constraint would add the edge y -> x with weight a
        if self.check_new_constraint(x, y, a) {
            self.update_solution(x, y, a);
            self.succesors.entry(Node::Variable(y)).or_insert(vec![]).push((Node::Variable(x), a));
            println!(
                "after inserting constraint {} - {} <= {}, succesrs of {} are: {:?}",
                x, y, a, y, self.succesors.get(&Node::Variable(y)),
            );
            // self.update_solution();
            // self.succesors.get_mut(&Node::Variable(y)).expect("error!").push((Node::Variable(x), a));
            true
        } else {
            println!("constraint {} - {} <= {} could not be added", x, y, a);
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
    // graph.add_edge("b", "a", -1);
    // graph.add_edge("c", "b", -2);
    // graph.add_edge("a", "c", 3);
    // for node in ["a", "b", "c"].into_iter(){
    //     graph.add_edge("source", node, 0);
    // };
    let mut sys = DCS::new();
    println!("{}", sys.add_constraint(1, 2, -1));
    println!("{}", sys.add_constraint(2, 3, -2));
    println!("{}", sys.add_constraint(3, 1, 3));
    println!("{}", sys.add_constraint(2, 1, -3));
    println!("{:?}", sys.solution);

    println!("Hello, world!");
}
