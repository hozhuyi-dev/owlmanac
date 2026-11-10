//! A CDCL SAT solver. Start reading at `search.rs`, the main loop.

#![forbid(unsafe_code)]

mod baseline;
pub mod cardinality;
mod cnf;
mod data;
pub mod dimacs;
mod ladder;
mod observer;
mod probe;
mod proof;
pub mod random;
mod search;
mod solver;
mod steps;

pub use cnf::Cnf;
pub use data::lit::{Lit, Var};
pub use ladder::ladder;
pub use observer::{NoObserver, Observer};
pub use probe::{Contradiction, Deduction, Deductions};
pub use proof::DratProof;
pub use solver::{Budget, Config, Solver, Stats, Status, Strategy};
