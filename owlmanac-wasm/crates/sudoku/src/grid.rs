//! A Sudoku grid of any box size: 3 is the usual 9×9, 4 is 16×16.

use std::fmt;

pub const BLANK: u8 = 0;
const SMALLEST_BOX: usize = 2;
const LARGEST_BOX: usize = 5;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum GridError {
    BoxSizeOutOfRange(usize),
    WrongCellCount { expected: usize, found: usize },
    DigitOutOfRange(u8),
    ConflictingClues { row: usize, col: usize },
    BadCharacter(char),
}

impl fmt::Display for GridError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GridError::BoxSizeOutOfRange(size) => {
                write!(
                    f,
                    "box size {size} is not between {SMALLEST_BOX} and {LARGEST_BOX}"
                )
            }
            GridError::WrongCellCount { expected, found } => {
                write!(f, "expected {expected} cells, found {found}")
            }
            GridError::DigitOutOfRange(digit) => write!(f, "digit {digit} is out of range"),
            GridError::ConflictingClues { row, col } => {
                write!(f, "the clue at row {row}, column {col} repeats a digit")
            }
            GridError::BadCharacter(character) => write!(f, "bad character: {character}"),
        }
    }
}

impl std::error::Error for GridError {}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Grid {
    box_size: usize,
    /// Row by row, with `BLANK` for an empty cell.
    cells: Vec<u8>,
}

impl Grid {
    pub fn new(box_size: usize, cells: Vec<u8>) -> Result<Self, GridError> {
        if !(SMALLEST_BOX..=LARGEST_BOX).contains(&box_size) {
            return Err(GridError::BoxSizeOutOfRange(box_size));
        }
        let side = box_size * box_size;
        if cells.len() != side * side {
            return Err(GridError::WrongCellCount {
                expected: side * side,
                found: cells.len(),
            });
        }
        if let Some(&digit) = cells.iter().find(|&&digit| digit as usize > side) {
            return Err(GridError::DigitOutOfRange(digit));
        }

        let grid = Grid { box_size, cells };
        for row in 0..side {
            for col in 0..side {
                let digit = grid.get(row, col);
                if digit != BLANK && !grid.can_place(row, col, digit) {
                    return Err(GridError::ConflictingClues { row, col });
                }
            }
        }
        Ok(grid)
    }

    /// Reads a grid written with `.` or `0` for blanks and digits in base 36, so 16×16 uses 1-9
    /// then A-G. Whitespace is ignored.
    pub fn parse(box_size: usize, text: &str) -> Result<Self, GridError> {
        let cells = text
            .chars()
            .filter(|character| !character.is_whitespace())
            .map(|character| match character {
                '.' => Ok(BLANK),
                _ => character
                    .to_digit(36)
                    .map(|digit| digit as u8)
                    .ok_or(GridError::BadCharacter(character)),
            })
            .collect::<Result<Vec<u8>, GridError>>()?;
        Grid::new(box_size, cells)
    }

    pub fn box_size(&self) -> usize {
        self.box_size
    }

    pub fn side(&self) -> usize {
        self.box_size * self.box_size
    }

    pub fn cells(&self) -> &[u8] {
        &self.cells
    }

    pub fn get(&self, row: usize, col: usize) -> u8 {
        self.cells[row * self.side() + col]
    }

    pub(crate) fn set(&mut self, row: usize, col: usize, digit: u8) {
        let side = self.side();
        self.cells[row * side + col] = digit;
    }

    /// Whether `digit` is absent from this cell's row, column and box, not counting the cell itself.
    pub fn can_place(&self, row: usize, col: usize, digit: u8) -> bool {
        let side = self.side();
        let (box_row, box_col) = (row - row % self.box_size, col - col % self.box_size);
        (0..side).all(|index| {
            let in_box = (
                box_row + index / self.box_size,
                box_col + index % self.box_size,
            );
            (index == col || self.get(row, index) != digit)
                && (index == row || self.get(index, col) != digit)
                && (in_box == (row, col) || self.get(in_box.0, in_box.1) != digit)
        })
    }

    /// Whether this grid is full, breaks no rule, and keeps every clue of `puzzle`.
    pub fn is_solution_of(&self, puzzle: &Grid) -> bool {
        let side = self.side();
        let keeps_clues = self
            .cells
            .iter()
            .zip(&puzzle.cells)
            .all(|(&digit, &clue)| digit != BLANK && (clue == BLANK || clue == digit));
        let breaks_no_rule =
            (0..side).all(|row| (0..side).all(|col| self.can_place(row, col, self.get(row, col))));
        keeps_clues && breaks_no_rule
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOLVED: &str = "1234 3412 2143 4321";

    #[test]
    fn a_full_valid_grid_is_a_solution_of_its_own_clues() {
        let grid = Grid::parse(2, SOLVED).unwrap();
        assert!(grid.is_solution_of(&grid));
    }

    #[test]
    fn a_solution_must_keep_the_clues() {
        let grid = Grid::parse(2, SOLVED).unwrap();
        let puzzle = Grid::parse(2, "2... .... .... ....").unwrap();
        assert!(!grid.is_solution_of(&puzzle));
    }

    #[test]
    fn repeated_clues_are_refused_in_each_kind_of_unit() {
        let conflict = |text| {
            matches!(
                Grid::parse(2, text),
                Err(GridError::ConflictingClues { .. })
            )
        };
        assert!(conflict("1..1 .... .... ...."));
        assert!(conflict("1... .... 1... ...."));
        assert!(conflict("1... .1.. .... ...."));
    }

    #[test]
    fn malformed_grids_are_refused() {
        assert_eq!(Grid::new(1, vec![0]), Err(GridError::BoxSizeOutOfRange(1)));
        assert_eq!(
            Grid::new(2, vec![0; 15]),
            Err(GridError::WrongCellCount {
                expected: 16,
                found: 15
            })
        );
        assert_eq!(
            Grid::new(2, vec![5; 16]),
            Err(GridError::DigitOutOfRange(5))
        );
        assert_eq!(
            Grid::parse(2, "?... .... .... ...."),
            Err(GridError::BadCharacter('?'))
        );
    }

    #[test]
    fn large_grids_read_digits_past_nine() {
        let mut text = vec!['.'; 256];
        text[0] = 'G';
        let grid = Grid::parse(4, &text.into_iter().collect::<String>()).unwrap();
        assert_eq!(grid.get(0, 0), 16);
    }
}
