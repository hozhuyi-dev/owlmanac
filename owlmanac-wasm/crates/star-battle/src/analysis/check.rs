//! Can this board still be finished? Catches a mistake long before anything visibly breaks.

use crate::board::Board;
use crate::session::Session;
use sat_core::{Budget, Status};

impl Session {
    pub fn is_solvable(&mut self, board: &Board) -> bool {
        let assumptions = self.literals(board);
        self.play.solve_assuming(&assumptions, Budget::unlimited()) == Status::Sat
    }
}
