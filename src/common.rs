use std::fmt::{Debug, Display};
use std::hash::Hash;

pub trait VarId: Eq + Hash + Debug + Clone + Display {}
impl<T> VarId for T where T: Eq + Hash + Debug + Clone + Display {}

pub trait ConstraintTag: Eq + Hash + Debug + Clone {}
impl<C> ConstraintTag for C where C: Eq + Hash + Debug + Clone {}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Constraint<T: VarId, C: ConstraintTag> {
    // v - u <= c
    pub v: T,
    pub u: T,
    pub c: i64,
    pub tag: C,
}

impl<T: VarId, C: ConstraintTag> Display for Constraint<T, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {} <= {}", self.v, self.u, self.c)
    }
}
