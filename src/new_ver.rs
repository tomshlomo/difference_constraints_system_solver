use std::collections::HashMap;

struct Variable{
    name: String
}

impl Variable {
    fn new(name: String) -> Variable{
        Variable{name: name}
    }
}

enum Node<'a>{
    Variable(&'a Variable),
    Source,
}

struct Constraint<'a >{
    u: Node<'a>,
    v: Node<'a>,
    a: i64,
}

impl <'a> Constraint<'a> {
    fn new(u: &'a Variable, v: &'a Variable, a: i64) -> Constraint<'a>{
        Constraint{
            u: Node::Variable(u),
            v: Node::Variable(v),
            a: a,
        }
    }
    fn source(v: &Variable) -> Constraint {
        Constraint{
            u: Node::Source,
            v: Node::Variable(v),
            a: 0,
        }
    }
}

struct Solution<'a>{
    map: HashMap<Node<'a>, i64>
}

struct System<'a>{
    succesors: HashMap<Node<'a>, Vec<(Node<'a>, i64)>>,
}

fn main() {

}