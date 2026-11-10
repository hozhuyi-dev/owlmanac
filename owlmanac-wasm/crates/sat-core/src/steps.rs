//! One stage of the search per file, each an `impl Solver` block called from `search.rs`.

pub mod analyze;
pub mod decide;
pub mod learn;
pub mod propagate;
pub mod reduce;
pub mod restart;
