mod lib;
use lib::{Solution, System, VarId, Constraint};
use pathfinding::{prelude::{dijkstra_all}, directed::dijkstra::dijkstra};


#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Node<'a, T> {
    Source,
    Var(&'a T),
}
impl <'a, T: VarId> Clone for Node<'a, T> {
    fn clone(&self) -> Self {
        match self {
            Node::Source => Node::Source,
            Node::Var(var) => Node::Var(var.clone())
        }
    }
}

pub struct SolvedSystem<'a, T: VarId> {
    sys: System<'a, T>,
    sol: Solution<'a, T>,
    unprocessed: Vec<Constraint<'a, T>>,
}

impl<'a, T: VarId> SolvedSystem<'a, T> {
    pub fn new(sys: System<'a, T>, sol: Solution<'a, T>) -> Self{
        SolvedSystem { sys: sys, sol: sol, unprocessed: vec![] }
    }
    pub fn check_new_constraint(
        &self,
        c: Constraint<'a, T>,
    ) -> bool {
        if !self.sys.is_variable(c.u) | !self.sys.is_variable(c.v) | c.check(&self.sol) {
            return true
        }
        if let Some(implied_lb) = self.get_implied_lb(c.u, c.v) {
            implied_lb <= c.a
        } else {
            true
        }
    }
    pub fn get_implied_lb(&self, u: &T, v: &T) -> Option<i64> {
        if let Some(implied_ub) = self.get_implied_ub(v, u) {
            Some(-implied_ub)
        } else {
            None
        }
    }
    pub fn get_implied_ub(&self, u: &T, v: &T) -> Option<i64> {
        self.dist(Node::Var(u), Node::Var(v))
    }
    fn scaled_succesors(&self, node: Node<'a, T>) -> Vec<(Node<'a, T>, i64)> {
        match node {
            Node::Source => {
                self.sys.variables().into_iter().map(
                    |v| {(Node::Var(v), 0)}
                ).collect()
            },
            Node::Var(u) => {
                vec![]
            },
        }
        // let s = self.sys.0.get(node).unwrap_or(&def);
        // let out: Vec<(Node, i64)> = s.into_iter().map(
        //     |(y, w)| {
        //         (
        //             y.clone(),
        //             sol.0[node] + w - sol.0[y]
        //         )
        //     }
        // ).collect();    
        // println!("\tsuccesors of {:?} are {:?}", node, out);
        // out
    }
    fn descale_dist(&self, scaled_dist: i64, from_node: &Node, to_node: &Node, sol: &Solution) -> i64 {
        -sol.0[from_node] + scaled_dist + sol.0[to_node]
    }
    fn dist(self: &Self, from_node: Node<'a, T>, to_node: Node<'a, T>) -> Option<i64> {
        let result = dijkstra(
            &from_node,
            |node| {
                self.scaled_succesors(node.clone())
            },
            |node| {node == &to_node},
        );
        match result {
            Some((_, cost)) => {
                // Some(self.descale_dist(cost, from_node, to_node, sol))
                Some(cost)
            },
            None => {
                None
            },
        }
    }
}