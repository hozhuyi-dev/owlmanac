//! Which cells are settled: the same in every way of finishing this board.

use crate::board::{Board, Mark, Placed};
use crate::session::Session;
use sat_core::{Budget, Status};

impl Session {
    /// For each cell, row by row, the mark every solution agrees on, or `None` if they differ.
    /// Returns `None` overall if the board can't be finished.
    ///
    /// Each model found rules out every cell it disagrees on, so most cells need no query.
    pub fn forced(&mut self, board: &Board) -> Option<Vec<Option<Mark>>> {
        let assumptions = self.literals(board);
        if self.play.solve_assuming(&assumptions, Budget::unlimited()) != Status::Sat {
            return None;
        }

        let cells = self.encoding.num_cells();
        let mut ever_star = vec![false; cells];
        let mut ever_empty = vec![false; cells];
        self.record_model(&mut ever_star, &mut ever_empty);

        for cell in self.puzzle.cells() {
            let index = self.puzzle.index(cell);
            if ever_star[index] && ever_empty[index] {
                continue;
            }
            let untried = if ever_star[index] {
                Mark::Empty
            } else {
                Mark::Star
            };
            let mut trial = assumptions.clone();
            trial.push(self.encoding.lit(Placed {
                cell,
                mark: untried,
            }));
            if self.play.solve_assuming(&trial, Budget::unlimited()) == Status::Sat {
                self.record_model(&mut ever_star, &mut ever_empty);
            }
        }

        Some(
            ever_star
                .into_iter()
                .zip(ever_empty)
                .map(|seen| match seen {
                    (true, false) => Some(Mark::Star),
                    (false, true) => Some(Mark::Empty),
                    _ => None,
                })
                .collect(),
        )
    }

    fn record_model(&self, ever_star: &mut [bool], ever_empty: &mut [bool]) {
        let model = &self.play.model()[..self.encoding.num_cells()];
        for (index, &star) in model.iter().enumerate() {
            if star {
                ever_star[index] = true;
            } else {
                ever_empty[index] = true;
            }
        }
    }
}
