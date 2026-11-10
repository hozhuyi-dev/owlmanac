//! One puzzle being played, with the solvers that answer questions about it.

use crate::board::{Board, Placed};
use crate::encode::{Encoding, Guarded};
use crate::puzzle::Puzzle;
use sat_core::{Lit, Solver};

pub struct Session {
    pub(crate) puzzle: Puzzle,
    pub(crate) encoding: Encoding,
    /// Answers yes-or-no questions. It keeps what it learns between them.
    pub(crate) play: Solver,
    /// Finds which marks and rules are to blame.
    pub(crate) blame: Guarded,
    /// Only ever propagates. Having never searched, it has no learnt clauses, so every reason it
    /// gives is one rule of the puzzle and can be read out as a deduction.
    pub(crate) deduce: Guarded,
}

impl Session {
    pub fn new(puzzle: Puzzle) -> Self {
        let encoding = Encoding::new(&puzzle);
        Session {
            play: encoding.plain_solver(),
            blame: Guarded::new(&encoding),
            deduce: Guarded::new(&encoding),
            encoding,
            puzzle,
        }
    }

    pub fn puzzle(&self) -> &Puzzle {
        &self.puzzle
    }

    pub(crate) fn literals(&self, board: &Board) -> Vec<Lit> {
        board
            .placed()
            .map(|placed: Placed| self.encoding.lit(placed))
            .collect()
    }
}
