//! Star Battle on top of `sat-core`. The public API speaks of cells, marks and rules, never of
//! literals. Start reading at `session.rs`; each thing a reader can ask is one file in `analysis/`.

#![forbid(unsafe_code)]

mod analysis;
mod board;
mod encode;
mod puzzle;
mod regions;
mod rules;
mod session;

pub use analysis::blame::Blame;
pub use analysis::count::Solutions;
pub use analysis::difficulty::{Grade, Hint, MAX_DEPTH};
pub use analysis::learned::LearnedRule;
pub use analysis::why_not::{Chain, Clash, Step, WhyNot};
pub use board::{Board, Mark, Placed};
pub use puzzle::{Cell, Puzzle, PuzzleError};
pub use regions::{Attempt, Attempts, Generation, Strategy, generate};
pub use rules::Rule;
pub use session::Session;
