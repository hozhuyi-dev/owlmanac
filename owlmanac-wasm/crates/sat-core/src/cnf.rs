//! A formula as plain data, before any solver has seen it.

use crate::data::lit::Lit;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cnf {
    pub num_vars: u32,
    pub clauses: Vec<Vec<Lit>>,
}

impl Cnf {
    /// `model` holds each variable's value, by variable index.
    pub fn is_satisfied_by(&self, model: &[bool]) -> bool {
        self.clauses.iter().all(|clause| {
            clause
                .iter()
                .any(|lit| model[lit.var().index()] == lit.is_positive())
        })
    }
}
