use priority_queue::PriorityQueue;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub trait VarId: Eq + Hash + Debug + Clone {}
impl<T> VarId for T where T: Eq + Hash + Debug + Clone {}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum Node<T: VarId> {
    Source,
    Variable(T),
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Constraint<T: VarId>{
    // v - u <= c
    pub v: T,
    pub u: T,
    pub c: i64,
}
impl <T: VarId> Constraint<T> {
    pub fn new(v: T, u: T, c: i64) -> Self {
        Constraint{v: u, u: v, c: c}
    }
}

#[derive(Debug, Clone)]
pub struct Solution<T: VarId>(HashMap<Node<T>, i64>);

impl<T: VarId> Solution<T> {
    pub fn new() -> Solution<T> {
        let mut map = HashMap::new();
        map.insert(Node::Source, 0);
        Solution(map)
    }
    fn update(&mut self, node: &Node<T>, val: i64) {
        self.0.insert(node.clone(), val);
    }
    pub fn add_var_if_missing(&mut self, var: T) {
        let node = Node::Variable(var);
        if !self.0.contains_key(&node) {
            self.update(&node, 0);
        };
    }
    pub fn get(&self, var: &T) -> i64 {
        self.safe_get_node(&Node::Variable(var.clone())).unwrap_or(&0).clone()
    }
    fn get_node(&self, node: &Node<T>) -> i64 {
        self.safe_get_node(node).unwrap().clone()
    }
     fn safe_get_node(&self, node: &Node<T>) -> Option<&i64> {
        self.0.get(node)
    }
    pub fn check_constraint(&self, constraint: &Constraint<T>) -> bool {
        // v - u <= c
        if let (
            Some(u_val),
            Some(v_val),
            ) = (
            self.safe_get_node(&Node::Variable(constraint.u.clone())), 
            self.safe_get_node(&Node::Variable(constraint.v.clone())),
        ) {
            return v_val - u_val <= constraint.c
        }
        true
    }
}

pub struct DCS<T: VarId> {
    // graph: DiGraphMap<Node, i64>,
    succesors: HashMap<Node<T>, Vec<(T, i64)>>,
}

impl<T: VarId> DCS<T> {
    fn new() -> Self {
        let mut succesors = HashMap::new();
        succesors.insert(Node::Source, vec![]);
        DCS {
            succesors: succesors,
        }
    }
    pub fn is_variable(&self, x: &T) -> bool {
        self.succesors.contains_key(&Node::Variable(x.clone()))
    }
    pub fn add_unconstrained_variable(&mut self, x: &T) {
        if !self.is_variable(x) {
            let node = Node::Variable(x.clone());
            self.add_succesor(&Node::Source, x, 0);
            self.succesors.insert(node, vec![]);
        }
    }
    fn var_constraints(&self, u: &T) -> Vec<Constraint<T>> {
        self.succesors.get(&Node::Variable(u.clone())).unwrap_or(&vec![]).iter().map(
            |(v, c)| {
                Constraint::new(v.clone(), u.clone(), c.clone())
            }
        ).collect()
    }
    fn constraints(&self) -> Vec<Constraint<T>> {
        // not great
        self.succesors.keys().into_iter().map(|node| {
            match node {
                Node::Source => vec![],
                Node::Variable(var) => self.var_constraints(var)
            }
        }).flat_map(|s| s.clone()).collect()
    }
    pub fn check_solution(&self, sol: &Solution<T>) -> bool {
        for constraint in self.constraints() {
            if !sol.check_constraint(&constraint) {
                return false
            }
        }
        true
    }
    fn add_succesor(&mut self, from_node: &Node<T>, to_variable: &T, c: i64) {
        self.succesors
            .get_mut(from_node)
            .expect("variable should be added")
            .push((to_variable.clone(), c));
    }
    pub fn add_to_feasible(&mut self, constraint: &Constraint<T>, sol: &Solution<T>) -> Option<Solution<T>> {
        self.add_unconstrained_variable(&constraint.v);
        self.add_unconstrained_variable(&constraint.u);
        let v_node = Node::Variable(constraint.v.clone());
        let u_node = Node::Variable(constraint.u.clone());
        self.add_succesor(&u_node, &constraint.v, constraint.c);
        let mut q = PriorityQueue::new();
        let mut new_sol = sol.clone();
        q.push(v_node, 0);
        loop {
            if let Some((x, dist_x)) = q.pop() {
                let new_val = sol.safe_get_node(&u_node).unwrap_or(&0) + constraint.c + dist_x;
                let d_x = sol.safe_get_node(&x).unwrap_or(&0).clone();
                if new_val < d_x {
                    if x == u_node {
                        self.succesors.get_mut(&u_node).unwrap().pop(); // remove the new constraint
                        return None;
                    } 
                    new_sol.update(&x, new_val);
                    for (y, x2y) in self.succesors.get(&x).unwrap() {
                        let scaled_path_len = dist_x + d_x + x2y - sol.get(y);
                        q.push_decrease(Node::Variable(y.clone()), scaled_path_len);
                    }
                }
            } else {
                return Some(new_sol);
            }
        }
    }
}
#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_check() {
        let x = "x".to_owned();
        let y = "y".to_owned();
        let mut sys = DCS::new();
        let sol = Solution::new();
        let new_sol = sys.add_to_feasible(&Constraint::new(x, y, 0), &sol);
        assert!(sys.check_solution(&new_sol.unwrap()))
    }
}