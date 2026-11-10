//! Star Battle for JavaScript. Each method converts its arguments, calls `star-battle`, and
//! converts the answer; anything more belongs in that crate.
//!
//! A board crosses as an `Int8Array`, one entry per cell row by row: 1 a star, -1 ruled out, 0 unmarked.

use serde::Serialize;
use star_battle::{Attempts, Board, Cell, Mark, Placed, Puzzle, Session};
use wasm_bindgen::prelude::*;

const STAR: i8 = 1;
const RULED_OUT: i8 = -1;
const UNMARKED: i8 = 0;

#[wasm_bindgen]
pub struct StarBattle {
    session: Session,
}

#[wasm_bindgen]
impl StarBattle {
    /// `regions` holds each cell's region, row by row.
    #[wasm_bindgen(constructor)]
    pub fn new(size: usize, stars: usize, regions: &[u8]) -> Result<StarBattle, JsError> {
        let regions = regions.iter().map(|&region| region as usize).collect();
        let puzzle = Puzzle::new(size, stars, regions)?;
        Ok(StarBattle {
            session: Session::new(puzzle),
        })
    }

    #[wasm_bindgen(js_name = isSolvable)]
    pub fn is_solvable(&mut self, marks: &[i8]) -> bool {
        let board = self.board(marks);
        self.session.is_solvable(&board)
    }

    /// A minimal set of marks and rules that can't stand together, or `undefined`.
    pub fn blame(&mut self, marks: &[i8]) -> Result<JsValue, JsError> {
        let board = self.board(marks);
        to_js(&self.session.blame(&board))
    }

    #[wasm_bindgen(js_name = whyNot)]
    pub fn why_not(
        &mut self,
        marks: &[i8],
        row: usize,
        col: usize,
        star: bool,
    ) -> Result<JsValue, JsError> {
        let board = self.board(marks);
        let proposal = Placed {
            cell: Cell { row, col },
            mark: if star { Mark::Star } else { Mark::Empty },
        };
        to_js(&self.session.why_not(&board, proposal))
    }

    /// Per cell, what every way of finishing agrees on, encoded like a board. `undefined` if none exists.
    pub fn forced(&mut self, marks: &[i8]) -> Option<Vec<i8>> {
        let board = self.board(marks);
        let forced = self.session.forced(&board)?;
        Some(forced.into_iter().map(encode_mark).collect())
    }

    pub fn solutions(&self, marks: &[i8], limit: usize) -> Result<JsValue, JsError> {
        to_js(&self.session.solutions(&self.board(marks), limit))
    }

    #[wasm_bindgen(js_name = learnedRules)]
    pub fn learned_rules(&self, limit: usize) -> Result<JsValue, JsError> {
        to_js(&self.session.learned_rules(limit))
    }

    /// The easiest cell to work out next and how deep it takes, or `undefined` if there's none.
    pub fn hint(&self, marks: &[i8]) -> Result<JsValue, JsError> {
        to_js(&self.session.hint(&self.board(marks)))
    }

    /// How deep the reasoning has to go to finish this board, or `undefined` if it can't be.
    pub fn grade(&self, marks: &[i8]) -> Result<JsValue, JsError> {
        to_js(&self.session.grade(&self.board(marks)))
    }

    fn board(&self, marks: &[i8]) -> Board {
        decode_board(self.session.puzzle().size(), marks)
    }
}

/// Puzzle generation by choosing the stars first, one attempt at a time, for watching it work.
#[wasm_bindgen]
pub struct Generator {
    attempts: Attempts,
}

#[wasm_bindgen]
impl Generator {
    #[wasm_bindgen(constructor)]
    pub fn new(size: usize, stars: usize, seed: u32) -> Generator {
        Generator {
            attempts: Attempts::new(size, stars, seed as u64),
        }
    }

    /// The next attempt: the stars chosen, the regions grown, and another solution if there is one.
    pub fn attempt(&mut self) -> Result<JsValue, JsError> {
        to_js(&self.attempts.next_attempt())
    }
}

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
    serde_wasm_bindgen::to_value(value).map_err(|error| JsError::new(&error.to_string()))
}

fn encode_mark(mark: Option<Mark>) -> i8 {
    match mark {
        Some(Mark::Star) => STAR,
        Some(Mark::Empty) => RULED_OUT,
        None => UNMARKED,
    }
}

fn decode_board(size: usize, marks: &[i8]) -> Board {
    let mut board = Board::new();
    for (index, &mark) in marks.iter().enumerate() {
        let mark = match mark {
            STAR => Mark::Star,
            RULED_OUT => Mark::Empty,
            _ => continue,
        };
        board.place(Placed {
            cell: Cell {
                row: index / size,
                col: index % size,
            },
            mark,
        });
    }
    board
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_board_decodes_row_by_row() {
        let board = decode_board(2, &[0, STAR, RULED_OUT, 0]);
        let at = |row, col| board.mark_at(Cell { row, col });

        assert_eq!(at(0, 0), None);
        assert_eq!(at(0, 1), Some(Mark::Star));
        assert_eq!(at(1, 0), Some(Mark::Empty));
    }
}
