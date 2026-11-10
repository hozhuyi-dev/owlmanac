//! The solver's data structures. Each one guards its own invariants and knows nothing about search.

pub mod clause_db;
pub mod lit;
pub mod maps;
pub mod trail;
pub mod var_order;
pub mod watches;
