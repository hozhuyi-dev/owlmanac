//! Sudoku as clauses. One variable per cell and digit, true if the cell holds that digit.

use crate::grid::{BLANK, Grid};
use sat_core::cardinality::{at_least, at_most};
use sat_core::{Cnf, Lit, Var};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Encoding {
    /// Just enough to be correct: every cell has a digit, and no unit repeats one.
    Minimal,
    /// Adds what is logically redundant: a cell has at most one digit, and every unit has every
    /// digit. Propagation then finds hidden singles without being told what they are.
    Extended,
}

pub fn literal(side: usize, row: usize, col: usize, digit: u8) -> Lit {
    Var::new((row * side * side + col * side + digit as usize - 1) as u32).positive()
}

/// The rules of an empty grid, with no clues.
pub fn rules(box_size: usize, encoding: Encoding) -> Cnf {
    let side = box_size * box_size;
    let digits = 1..=side as u8;
    let mut clauses = Vec::new();

    for row in 0..side {
        for col in 0..side {
            let lits: Vec<Lit> = digits.clone().map(|d| literal(side, row, col, d)).collect();
            clauses.extend(at_least(&lits, 1));
            if encoding == Encoding::Extended {
                clauses.extend(at_most(&lits, 1));
            }
        }
    }

    for unit in units(box_size) {
        for digit in digits.clone() {
            let lits: Vec<Lit> = unit
                .iter()
                .map(|&(row, col)| literal(side, row, col, digit))
                .collect();
            clauses.extend(at_most(&lits, 1));
            if encoding == Encoding::Extended {
                clauses.extend(at_least(&lits, 1));
            }
        }
    }

    Cnf {
        num_vars: (side * side * side) as u32,
        clauses,
    }
}

/// The rules plus one unit clause per clue.
pub fn encode(puzzle: &Grid, encoding: Encoding) -> Cnf {
    let side = puzzle.side();
    let mut cnf = rules(puzzle.box_size(), encoding);
    for row in 0..side {
        for col in 0..side {
            let digit = puzzle.get(row, col);
            if digit != BLANK {
                cnf.clauses.push(vec![literal(side, row, col, digit)]);
            }
        }
    }
    cnf
}

/// Reads a grid out of a model. The minimal encoding allows a cell several true digits, and any
/// one of them gives a valid grid, so this takes the first.
pub fn decode(puzzle: &Grid, model: &[bool]) -> Grid {
    let side = puzzle.side();
    let mut solved = puzzle.clone();
    for row in 0..side {
        for col in 0..side {
            let digit = (1..=side as u8)
                .find(|&digit| model[literal(side, row, col, digit).var().index()])
                .expect("every cell has at least one digit");
            solved.set(row, col, digit);
        }
    }
    solved
}

/// Every row, column and box, as lists of cells.
fn units(box_size: usize) -> Vec<Vec<(usize, usize)>> {
    let side = box_size * box_size;
    let mut units = Vec::with_capacity(3 * side);
    for line in 0..side {
        units.push((0..side).map(|col| (line, col)).collect());
        units.push((0..side).map(|row| (row, line)).collect());
        let (top, left) = (line / box_size * box_size, line % box_size * box_size);
        units.push(
            (0..side)
                .map(|index| (top + index / box_size, left + index % box_size))
                .collect(),
        );
    }
    units
}

#[cfg(test)]
mod tests {
    use super::*;
    use sat_core::{Config, Solver};

    #[test]
    fn nine_by_nine_clause_counts_match_the_literature() {
        assert_eq!(rules(3, Encoding::Minimal).clauses.len(), 8829);
        assert_eq!(rules(3, Encoding::Extended).clauses.len(), 11988);
    }

    #[test]
    fn only_the_extended_encoding_propagates_a_hidden_single() {
        // Digit 1 is shut out of the rest of the top row by the 1s in columns 1, 2 and 3, so the
        // top-left cell must be 1, though every digit is still open to it as far as it can see.
        let puzzle = Grid::parse(2, ".... ...1 .1.. ..1.").unwrap();
        let hidden_single = literal(4, 0, 0, 1);

        let propagates = |encoding| {
            let mut solver = Solver::from_cnf(&encode(&puzzle, encoding), Config::default());
            let deductions = solver.propagate_assuming(&[]);
            deductions
                .steps
                .iter()
                .any(|step| step.lit == hidden_single)
        };

        assert!(propagates(Encoding::Extended));
        assert!(!propagates(Encoding::Minimal));
    }
}
