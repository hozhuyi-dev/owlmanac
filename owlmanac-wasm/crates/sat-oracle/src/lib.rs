//! Making puzzles with exactly one solution, using SAT solvers as the oracle. Generic over the
//! puzzle: a puzzle describes itself through the traits in `problem.rs`. Three strategies:
//!
//! - `retry`: draw designs at random, keep one whose puzzle is unique.
//! - `dig`: show every clue, then take clues away while the answer stays unique.
//! - `Synthesizer`: one solver proposes designs, another finds second solutions, and the proposer
//!   learns from each. Start reading at `synthesis.rs`.

#![forbid(unsafe_code)]

mod data;
mod dig;
mod problem;
mod retry;
mod steps;
mod synthesis;
mod synthesizer;

pub use data::checker::{Checker, is_unique};
pub use data::outcome::{Outcome, Stats};
pub use data::proposer::Proposer;
pub use dig::dig;
pub use problem::{Clues, Puzzle, RandomDesign, Synthesis};
pub use retry::retry;
pub use synthesizer::Synthesizer;
