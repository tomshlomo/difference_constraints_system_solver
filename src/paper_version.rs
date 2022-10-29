
use std::{collections::HashMap};
use std::hash::Hash;
use std::fmt::Debug;
use priority_queue::PriorityQueue; 

pub trait VarId: Eq + Hash + Debug + Clone {}
impl<T> VarId for T where T: Eq + Hash + Debug + Clone {}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Node<T> where T: VarId {
    Source,
    Variable(T),
}

#[derive(Debug, Clone)]
pub struct Solution<T: VarId>(HashMap<Node<T>, i64>);

impl <T: VarId> Solution<T> {
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
        if !self.0.contains_key(&node){
            self.update(&node, 0);
        };
    }
    pub fn get(&self, node: &Node<T>) -> i64 {
        self.0.get(node).unwrap().clone()
    }
}

pub struct DCS<T> where T: VarId{
    // graph: DiGraphMap<Node, i64>,
    succesors: HashMap<Node<T>, Vec<(Node<T>, i64)>>,
}

impl  <T: VarId> DCS<T>{
    fn new() -> Self {
        let mut succesors = HashMap::new();
        succesors.insert(Node::Source, vec![]);
        DCS{
            succesors: succesors, 
        }
    }
    fn is_variable(&self, x: &Node<T>) -> bool {
        self.succesors.contains_key(x)
    }
    fn add_unconstrained_variable(&mut self, x: &Node<T>) {
        if !self.is_variable(&x) {
            self.add_constraint(&Node::Source, x, 0);
        }
    }
    fn add_constraint(&mut self, v: &Node<T>, u: &Node<T>, c: i64) {
        self.succesors.get_mut(u).expect("variable should be added").push((v.clone(), c));
    }
    pub fn add_to_feasible(&mut self, v: Node<T>, u: Node<T>, c: i64, sol: &Solution<T>) -> bool {
        self.add_unconstrained_variable(&v);
        self.add_unconstrained_variable(&u);
        self.add_constraint(&v, &u, c);
        let mut q = PriorityQueue::new();
        let mut new_sol = sol.clone();
        q.push(v, 0);
        loop {
            if let Some((x, dist_x)) = q.pop() {
                let new_val = sol.get(&u) + c + dist_x;
                let d_x = sol.get(&x);
                if new_val < d_x {
                    if x == u {
                        // todo: remove new constraint
                        return false
                    } else {
                        new_sol.update(&x, new_val);
                        for (y, x2y) in self.succesors.get(&x).unwrap() {
                            let scaled_path_len = dist_x + d_x + x2y - sol.get(y);
                            q.push_decrease(y.clone(), scaled_path_len);
                        }
                    }
                }
            } else {
                return true
            }
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
    let mut sys: DCS<usize> = DCS::new();
    let mut sol: Solution<usize> = Solution::new();
    // println!("{}", sys.add_constraint_helper(&1, &2, -1, &mut sol));
    // println!("{}", sys.add_constraint_helper(&2, &3, -2, &mut sol));
    // println!("{}", sys.add_constraint_helper(&3, &1, 3, &mut sol));
    // println!("{}", sys.add_constraint_helper(&2, &1, -3, &mut sol));
    // println!("{:?}", sol);
}
