use std::fmt::{Debug, Display};
use std::hash::Hash;

pub trait VarId: Eq + Hash + Debug + Clone + Display {}
impl<T> VarId for T where T: Eq + Hash + Debug + Clone + Display {}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Constraint<T: VarId> {
    // v - u <= c
    pub v: T,
    pub u: T,
    pub c: i64,
}

impl<T: VarId> Display for Constraint<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {} <= {}", self.v, self.u, self.c)
    }
}
