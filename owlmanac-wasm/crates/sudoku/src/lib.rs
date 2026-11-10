//! Sudoku of any box size, solved by brute force and by SAT, so the two can be compared. Start
//! reading at `ladder.rs`.

#![forbid(unsafe_code)]

mod brute_force;
mod encode;
mod generate;
mod grid;
mod ladder;

pub use brute_force::{BruteForce, Progress};
pub use encode::{Encoding, decode, encode, literal, rules};
pub use generate::{generate, solved_grid};
pub use grid::{Grid, GridError};
pub use ladder::{LadderRun, Report, rung_names};
