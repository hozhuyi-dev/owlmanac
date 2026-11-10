//! How many ways can this board be finished? A proper puzzle has exactly one.

use crate::board::{Board, Mark, Placed};
use crate::puzzle::Cell;
use crate::session::Session;
use sat_core::{Budget, Status};
use serde::Serialize;

#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct Solutions {
    /// The stars of each solution found.
    pub found: Vec<Vec<Cell>>,
    /// Whether `found` is all of them, or the search stopped at its limit.
    pub complete: bool,
}

impl Session {
    /// Finds up to `limit` solutions. A limit of 2 is enough to tell whether a puzzle is proper.
    pub fn solutions(&self, board: &Board, limit: usize) -> Solutions {
        // Blocking clauses are permanent, so this gets a solver of its own.
        let mut solver = self.encoding.plain_solver();
        let assumptions = self.literals(board);
        let mut found = Vec::new();

        while found.len() < limit {
            if solver.solve_assuming(&assumptions, Budget::unlimited()) != Status::Sat {
                return Solutions {
                    found,
                    complete: true,
                };
            }
            let stars: Vec<Cell> = self
                .puzzle
                .cells()
                .filter(|&cell| solver.model()[self.puzzle.index(cell)])
                .collect();

            // Every solution has the same number of stars, so a different one must drop at least
            // one of these.
            let blocking: Vec<_> = stars
                .iter()
                .map(|&cell| {
                    self.encoding.lit(Placed {
                        cell,
                        mark: Mark::Empty,
                    })
                })
                .collect();
            solver.add_clause(&blocking);
            found.push(stars);
        }

        let complete = solver.solve_assuming(&assumptions, Budget::unlimited()) != Status::Sat;
        Solutions { found, complete }
    }
}
