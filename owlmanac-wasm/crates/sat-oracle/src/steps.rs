//! One stage of the loop per file, each an `impl Synthesizer` block called from `synthesis.rs`.

pub mod check;
pub mod learn;
pub mod propose;
pub mod refine;
