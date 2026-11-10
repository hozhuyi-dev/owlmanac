//! Reproducible puzzles with exactly one solution.

use crate::encode::{Encoding, literal, rules};
use crate::grid::{BLANK, Grid};
use sat_core::random::Rng;
use sat_core::{Cnf, Lit};
use sat_oracle::{Clues, dig};

/// A valid full grid: a fixed pattern with its digits, rows and columns shuffled.
pub fn solved_grid(box_size: usize, seed: u64) -> Grid {
    let side = box_size * box_size;
    let mut rng = Rng::new(seed);
    let digits = shuffled((1..=side as u8).collect(), &mut rng);
    let rows = shuffled_lines(box_size, &mut rng);
    let cols = shuffled_lines(box_size, &mut rng);

    let mut cells = Vec::with_capacity(side * side);
    for &row in &rows {
        for &col in &cols {
            let pattern = (box_size * (row % box_size) + row / box_size + col) % side;
            cells.push(digits[pattern]);
        }
    }
    Grid::new(box_size, cells).expect("the pattern is a valid grid under any shuffle")
}

/// Blanks cells in random order for as long as the solution stays unique, stopping once only
/// `target_clues` remain or nothing more can go. The digging is `sat_oracle::dig`: Sudoku only
/// says what its rules and clues are.
pub fn generate(box_size: usize, target_clues: usize, seed: u64) -> Grid {
    let solved = solved_grid(box_size, seed);
    let mut rng = Rng::new(seed.wrapping_add(1));
    let (shown, _) = dig(&SolvedGrid(&solved), &mut rng, target_clues);

    let puzzle = solved
        .cells()
        .iter()
        .zip(&shown)
        .map(|(&digit, &kept)| if kept { digit } else { BLANK })
        .collect();
    Grid::new(box_size, puzzle).expect("clues from a valid grid can't conflict")
}

/// A solved grid, every cell a clue that can be taken away.
struct SolvedGrid<'a>(&'a Grid);

impl Clues for SolvedGrid<'_> {
    fn clue_count(&self) -> usize {
        self.0.cells().len()
    }

    fn rules(&self) -> Cnf {
        rules(self.0.box_size(), Encoding::Extended)
    }

    fn clue(&self, index: usize) -> Vec<Vec<Lit>> {
        vec![vec![self.held(index)]]
    }

    fn intended(&self) -> Vec<Lit> {
        (0..self.clue_count())
            .map(|index| self.held(index))
            .collect()
    }
}

impl SolvedGrid<'_> {
    /// "This cell holds its digit", for the cell at `index`, row by row.
    fn held(&self, index: usize) -> Lit {
        let side = self.0.side();
        let (row, col) = (index / side, index % side);
        literal(side, row, col, self.0.get(row, col))
    }
}

fn shuffled<T>(mut items: Vec<T>, rng: &mut Rng) -> Vec<T> {
    for last in (1..items.len()).rev() {
        items.swap(last, rng.below(last as u32 + 1) as usize);
    }
    items
}

/// Row or column indices, shuffled without breaking a valid grid: lines only move within their
/// band, and whole bands move together.
fn shuffled_lines(box_size: usize, rng: &mut Rng) -> Vec<usize> {
    let bands = shuffled((0..box_size).collect::<Vec<_>>(), rng);
    let mut lines = Vec::with_capacity(box_size * box_size);
    for band in bands {
        let within = shuffled((0..box_size).collect::<Vec<_>>(), rng);
        lines.extend(within.into_iter().map(|line| band * box_size + line));
    }
    lines
}
