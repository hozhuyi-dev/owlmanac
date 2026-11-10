//! The solver from the start of the story: take the first blank, try each digit that doesn't
//! clash, move on, and step back when a cell runs out of digits.
//!
//! Usually written as a recursive function. This is the same search with the recursion replaced
//! by a cursor, visiting the same cells in the same order, so it can stop and resume between slices.

use crate::grid::{BLANK, Grid};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Progress {
    Running,
    Solved,
    NoSolution,
}

pub struct BruteForce {
    grid: Grid,
    blanks: Vec<(usize, usize)>,
    /// How many of `blanks` currently hold a digit.
    filled: usize,
    guesses: u64,
    dead_ends: u64,
    finished: Option<Progress>,
}

impl BruteForce {
    pub fn new(puzzle: &Grid) -> Self {
        let side = puzzle.side();
        let blanks = (0..side)
            .flat_map(|row| (0..side).map(move |col| (row, col)))
            .filter(|&(row, col)| puzzle.get(row, col) == BLANK)
            .collect();
        BruteForce {
            grid: puzzle.clone(),
            blanks,
            filled: 0,
            guesses: 0,
            dead_ends: 0,
            finished: None,
        }
    }

    /// Searches for roughly `slice` more guesses.
    pub fn advance(&mut self, slice: u64) -> Progress {
        if let Some(finished) = self.finished {
            return finished;
        }
        let stop_at = self.guesses + slice;
        while self.guesses < stop_at {
            if self.filled == self.blanks.len() {
                return self.finish(Progress::Solved);
            }
            let (row, col) = self.blanks[self.filled];
            match self.next_digit(row, col) {
                Some(digit) => {
                    self.grid.set(row, col, digit);
                    self.filled += 1;
                }
                None => {
                    self.grid.set(row, col, BLANK);
                    self.dead_ends += 1;
                    if self.filled == 0 {
                        return self.finish(Progress::NoSolution);
                    }
                    self.filled -= 1;
                }
            }
        }
        Progress::Running
    }

    fn finish(&mut self, outcome: Progress) -> Progress {
        self.finished = Some(outcome);
        outcome
    }

    /// The next digit above the cell's current one that doesn't clash, counting each one tried.
    fn next_digit(&mut self, row: usize, col: usize) -> Option<u8> {
        let side = self.grid.side() as u8;
        for digit in self.grid.get(row, col) + 1..=side {
            self.guesses += 1;
            if self.grid.can_place(row, col, digit) {
                return Some(digit);
            }
        }
        None
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn guesses(&self) -> u64 {
        self.guesses
    }

    pub fn dead_ends(&self) -> u64 {
        self.dead_ends
    }
}
