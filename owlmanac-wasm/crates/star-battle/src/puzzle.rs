//! A puzzle: a square grid split into as many regions as it has rows, and how many stars each
//! row, column and region must hold.

use serde::Serialize;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize)]
pub struct Cell {
    pub row: usize,
    pub col: usize,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PuzzleError {
    EmptyGrid,
    NoStars,
    WrongCellCount { expected: usize, found: usize },
    MissingRegion(usize),
    RegionOutOfRange(usize),
    UnevenRows,
}

impl fmt::Display for PuzzleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PuzzleError::EmptyGrid => write!(f, "the grid has no cells"),
            PuzzleError::NoStars => write!(f, "each unit must hold at least one star"),
            PuzzleError::WrongCellCount { expected, found } => {
                write!(f, "expected {expected} cells, found {found}")
            }
            PuzzleError::MissingRegion(region) => write!(f, "region {region} has no cells"),
            PuzzleError::RegionOutOfRange(region) => write!(f, "region {region} is out of range"),
            PuzzleError::UnevenRows => write!(f, "the grid is not square"),
        }
    }
}

impl std::error::Error for PuzzleError {}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Puzzle {
    size: usize,
    stars: usize,
    /// Each cell's region, row by row.
    regions: Vec<usize>,
}

impl Puzzle {
    pub fn new(size: usize, stars: usize, regions: Vec<usize>) -> Result<Self, PuzzleError> {
        if size == 0 {
            return Err(PuzzleError::EmptyGrid);
        }
        if stars == 0 {
            return Err(PuzzleError::NoStars);
        }
        if regions.len() != size * size {
            return Err(PuzzleError::WrongCellCount {
                expected: size * size,
                found: regions.len(),
            });
        }
        if let Some(&region) = regions.iter().find(|&&region| region >= size) {
            return Err(PuzzleError::RegionOutOfRange(region));
        }
        if let Some(region) = (0..size).find(|region| !regions.contains(region)) {
            return Err(PuzzleError::MissingRegion(region));
        }
        Ok(Puzzle {
            size,
            stars,
            regions,
        })
    }

    /// Reads a grid drawn as lines of characters, one character per region.
    pub fn parse(stars: usize, text: &str) -> Result<Self, PuzzleError> {
        let rows: Vec<&str> = text
            .lines()
            .map(str::trim)
            .filter(|row| !row.is_empty())
            .collect();
        if rows.iter().any(|row| row.chars().count() != rows.len()) {
            return Err(PuzzleError::UnevenRows);
        }

        let mut names: Vec<char> = Vec::new();
        let mut regions = Vec::new();
        for name in rows.iter().flat_map(|row| row.chars()) {
            let region = names.iter().position(|&known| known == name);
            regions.push(region.unwrap_or_else(|| {
                names.push(name);
                names.len() - 1
            }));
        }
        Puzzle::new(rows.len(), stars, regions)
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn stars(&self) -> usize {
        self.stars
    }

    pub fn region_of(&self, cell: Cell) -> usize {
        self.regions[self.index(cell)]
    }

    pub fn cells(&self) -> impl Iterator<Item = Cell> + use<> {
        let size = self.size;
        (0..size).flat_map(move |row| (0..size).map(move |col| Cell { row, col }))
    }

    pub(crate) fn index(&self, cell: Cell) -> usize {
        cell.row * self.size + cell.col
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_numbers_regions_by_first_appearance() {
        let puzzle = Puzzle::parse(1, "ba\naa").unwrap();
        assert_eq!(puzzle, Puzzle::new(2, 1, vec![0, 1, 1, 1]).unwrap());
    }

    #[test]
    fn a_grid_needs_one_region_per_row() {
        assert_eq!(
            Puzzle::new(2, 1, vec![0, 0, 0, 0]),
            Err(PuzzleError::MissingRegion(1))
        );
        assert_eq!(
            Puzzle::new(2, 1, vec![0, 2, 1, 1]),
            Err(PuzzleError::RegionOutOfRange(2))
        );
    }

    #[test]
    fn a_grid_must_be_square() {
        assert_eq!(Puzzle::parse(1, "ab\na"), Err(PuzzleError::UnevenRows));
        assert_eq!(
            Puzzle::new(2, 1, vec![0, 1, 1]),
            Err(PuzzleError::WrongCellCount {
                expected: 4,
                found: 3
            })
        );
    }

    #[test]
    fn a_cell_knows_its_region() {
        let puzzle = Puzzle::parse(1, "ba\nab").unwrap();
        assert_eq!(puzzle.region_of(Cell { row: 0, col: 0 }), 0);
        assert_eq!(puzzle.region_of(Cell { row: 1, col: 0 }), 1);
    }
}
