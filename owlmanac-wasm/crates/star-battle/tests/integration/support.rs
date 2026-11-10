//! Puzzles with one known solution, and a brute-force solver that shares no code with the encoder.

use star_battle::{Cell, Mark, Placed, Puzzle};

pub struct Fixture {
    pub puzzle: Puzzle,
    pub solution: Vec<Cell>,
}

fn fixture(stars: usize, grid: &str, solution: &[(usize, usize)]) -> Fixture {
    Fixture {
        puzzle: Puzzle::parse(stars, grid).unwrap(),
        solution: solution
            .iter()
            .map(|&(row, col)| Cell { row, col })
            .collect(),
    }
}

/// Solved by propagation alone: the solver learns nothing.
pub fn five_by_five() -> Fixture {
    fixture(
        1,
        "deeaa
         ddeee
         dddee
         cddbd
         ccddd",
        &[(0, 4), (1, 2), (2, 0), (3, 3), (4, 1)],
    )
}

pub fn eight_by_eight_two_stars() -> Fixture {
    fixture(
        2,
        "cccgheee
         cccgheee
         ccgghaae
         bbbhhaaa
         bbhhdaaa
         bhhhdfff
         bhhhdfff
         hhhhdfff",
        &[
            (0, 1),
            (0, 3),
            (1, 5),
            (1, 7),
            (2, 1),
            (2, 3),
            (3, 5),
            (3, 7),
            (4, 0),
            (4, 2),
            (5, 4),
            (5, 6),
            (6, 0),
            (6, 2),
            (7, 4),
            (7, 6),
        ],
    )
}

pub fn ten_by_ten_two_stars() -> Fixture {
    fixture(
        2,
        "fffidddddd
         fffiiddddd
         hhhhiddddd
         hhhhiideed
         hhhgiiieee
         aagggiiejj
         aaaaaaajjj
         ccccaaabjj
         ccccaabbjb
         cccccbbbbb",
        &[
            (0, 0),
            (0, 2),
            (1, 4),
            (1, 6),
            (2, 1),
            (2, 8),
            (3, 3),
            (3, 5),
            (4, 7),
            (4, 9),
            (5, 2),
            (5, 4),
            (6, 0),
            (6, 7),
            (7, 5),
            (7, 9),
            (8, 1),
            (8, 3),
            (9, 6),
            (9, 8),
        ],
    )
}

pub fn every_fixture() -> [Fixture; 3] {
    [
        five_by_five(),
        eight_by_eight_two_stars(),
        ten_by_ten_two_stars(),
    ]
}

pub fn star(cell: Cell) -> Placed {
    Placed {
        cell,
        mark: Mark::Star,
    }
}

pub fn touching(a: Cell, b: Cell) -> bool {
    a != b && a.row.abs_diff(b.row) <= 1 && a.col.abs_diff(b.col) <= 1
}

/// Every solution, each as its stars in row order. Works row by row over bitmasks of columns.
pub fn brute_force(puzzle: &Puzzle) -> Vec<Vec<Cell>> {
    let size = puzzle.size();
    let rows: Vec<u32> = (0u32..1 << size)
        .filter(|mask| mask.count_ones() as usize == puzzle.stars() && mask & (mask << 1) == 0)
        .collect();
    let mut chosen = Vec::new();
    let mut solutions = Vec::new();
    extend(puzzle, &rows, &mut chosen, &mut solutions);
    solutions
}

fn extend(puzzle: &Puzzle, rows: &[u32], chosen: &mut Vec<u32>, solutions: &mut Vec<Vec<Cell>>) {
    if chosen.len() == puzzle.size() {
        let stars = stars_of(chosen, puzzle.size());
        if counts_are_right(puzzle, &stars) {
            solutions.push(stars);
        }
        return;
    }
    for &mask in rows {
        let touches_previous = chosen
            .last()
            .is_some_and(|&above| mask & (above | above << 1 | above >> 1) != 0);
        if !touches_previous {
            chosen.push(mask);
            extend(puzzle, rows, chosen, solutions);
            chosen.pop();
        }
    }
}

fn stars_of(masks: &[u32], size: usize) -> Vec<Cell> {
    let mut stars = Vec::new();
    for (row, mask) in masks.iter().enumerate() {
        stars.extend(
            (0..size)
                .filter(|col| mask >> col & 1 == 1)
                .map(|col| Cell { row, col }),
        );
    }
    stars
}

fn counts_are_right(puzzle: &Puzzle, stars: &[Cell]) -> bool {
    (0..puzzle.size()).all(|index| {
        stars.iter().filter(|cell| cell.col == index).count() == puzzle.stars()
            && stars
                .iter()
                .filter(|&&cell| puzzle.region_of(cell) == index)
                .count()
                == puzzle.stars()
    })
}
