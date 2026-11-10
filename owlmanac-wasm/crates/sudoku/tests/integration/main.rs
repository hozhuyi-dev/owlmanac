//! Brute force and SAT must agree, and each must find real solutions.

use proptest::collection::vec;
use proptest::prelude::*;
use sudoku::{BruteForce, Encoding, Grid, LadderRun, Progress, generate, rung_names, solved_grid};

/// Built to be slow for a solver that fills cells in order: the solution's first row is 987654321.
/// Posted by Rico Alan on Flickr in 2008, and used in Wikipedia's "Sudoku solving algorithms".
const AGAINST_BRUTE_FORCE: &str = "
    .........
    .....3.85
    ..1.2....
    ...5.7...
    ..4...1..
    .9.......
    5......73
    ..2.1....
    ....4...9";

fn finish(run: &mut LadderRun) -> Progress {
    loop {
        let progress = run.advance();
        if progress != Progress::Running {
            return progress;
        }
    }
}

/// The same search in its textbook recursive form.
struct Recursive {
    grid: Vec<u8>,
    box_size: usize,
    guesses: u64,
    dead_ends: u64,
}

impl Recursive {
    fn clashes(&self, at: usize, digit: u8) -> bool {
        let side = self.box_size * self.box_size;
        let (row, col) = (at / side, at % side);
        (0..side * side).any(|other| {
            let (r, c) = (other / side, other % side);
            let same_box = r / self.box_size == row / self.box_size
                && c / self.box_size == col / self.box_size;
            other != at && (r == row || c == col || same_box) && self.grid[other] == digit
        })
    }

    fn solve(&mut self) -> bool {
        let Some(at) = self.grid.iter().position(|&digit| digit == 0) else {
            return true;
        };
        let side = (self.box_size * self.box_size) as u8;
        for digit in 1..=side {
            self.guesses += 1;
            if !self.clashes(at, digit) {
                self.grid[at] = digit;
                if self.solve() {
                    return true;
                }
                self.grid[at] = 0;
            }
        }
        self.dead_ends += 1;
        false
    }
}

fn four_by_four() -> impl Strategy<Value = Grid> {
    vec(prop_oneof![3 => Just(0u8), 1 => 1u8..=4], 16)
        .prop_filter_map("clues must not repeat", |cells| Grid::new(2, cells).ok())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn every_rung_and_encoding_agrees(puzzle in four_by_four()) {
        let mut expected = None;
        for encoding in [Encoding::Minimal, Encoding::Extended] {
            for rung in 0..rung_names().len() {
                let mut run = LadderRun::new(&puzzle, rung, encoding).unwrap();
                let progress = finish(&mut run);
                prop_assert_eq!(progress, *expected.get_or_insert(progress));
                match run.solution() {
                    Some(solution) => prop_assert!(solution.is_solution_of(&puzzle)),
                    None => prop_assert_eq!(progress, Progress::NoSolution),
                }
            }
        }
    }

    #[test]
    fn the_cursor_search_is_the_recursive_search(puzzle in four_by_four()) {
        let mut recursive = Recursive {
            grid: puzzle.cells().to_vec(),
            box_size: 2,
            guesses: 0,
            dead_ends: 0,
        };
        let solved = recursive.solve();

        let mut search = BruteForce::new(&puzzle);
        let mut progress = Progress::Running;
        while progress == Progress::Running {
            progress = search.advance(1);
        }

        prop_assert_eq!(progress == Progress::Solved, solved);
        prop_assert_eq!(search.guesses(), recursive.guesses);
        prop_assert_eq!(search.dead_ends(), recursive.dead_ends);
        if solved {
            prop_assert_eq!(search.grid().cells(), recursive.grid.as_slice());
        }
    }

    #[test]
    fn shuffled_grids_are_always_valid(seed in any::<u64>(), box_size in 2usize..=5) {
        let grid = solved_grid(box_size, seed);
        prop_assert!(grid.is_solution_of(&grid));
    }
}

#[test]
fn the_adversarial_puzzle_stumps_brute_force_but_not_cdcl() {
    let puzzle = Grid::parse(3, AGAINST_BRUTE_FORCE).unwrap();

    let mut brute = BruteForce::new(&puzzle);
    assert_eq!(brute.advance(2_000_000), Progress::Running);

    let last_rung = rung_names().len() - 1;
    let mut cdcl = LadderRun::new(&puzzle, last_rung, Encoding::Extended).unwrap();
    assert_eq!(finish(&mut cdcl), Progress::Solved);

    let solution = cdcl.solution().unwrap();
    assert!(solution.is_solution_of(&puzzle));
    assert_eq!(&solution.cells()[..9], &[9, 8, 7, 6, 5, 4, 3, 2, 1]);
}

#[test]
fn brute_force_shows_its_digits_as_it_goes() {
    let puzzle = Grid::parse(3, AGAINST_BRUTE_FORCE).unwrap();
    let mut run = LadderRun::new(&puzzle, 0, Encoding::Extended).unwrap();
    assert_eq!(run.current_grid(), Some(&puzzle));

    run.advance();

    let current = run.current_grid().unwrap();
    let filled = |grid: &Grid| grid.cells().iter().filter(|&&digit| digit != 0).count();
    assert!(filled(current) > filled(&puzzle));
    assert!(
        puzzle
            .cells()
            .iter()
            .zip(current.cells())
            .all(|(&clue, &now)| clue == 0 || clue == now)
    );
}

#[test]
fn an_impossible_puzzle_has_no_solution_on_any_rung() {
    // The top-left cell can be neither 1, 2, 3 nor 4.
    let puzzle = Grid::parse(2, ".12. 34.. .... ....").unwrap();
    for rung in 0..rung_names().len() {
        let mut run = LadderRun::new(&puzzle, rung, Encoding::Extended).unwrap();
        assert_eq!(
            finish(&mut run),
            Progress::NoSolution,
            "{}",
            rung_names()[rung]
        );
        assert_eq!(run.solution(), None);
    }
}

#[test]
fn a_rung_past_the_top_does_not_exist() {
    let puzzle = Grid::parse(2, "....  .... .... ....").unwrap();
    assert!(LadderRun::new(&puzzle, rung_names().len(), Encoding::Extended).is_none());
}

#[test]
fn generated_puzzles_have_exactly_one_solution() {
    for seed in 0..10 {
        let puzzle = generate(3, 30, seed);
        assert!(solved_grid(3, seed).is_solution_of(&puzzle));
        assert_eq!(count_solutions(&puzzle, 2), 1, "seed {seed}");
    }
}

#[test]
fn generation_is_reproducible() {
    assert_eq!(generate(3, 30, 7), generate(3, 30, 7));
    assert_ne!(generate(3, 30, 7), generate(3, 30, 8));
}

/// Counts solutions up to `limit` with plain backtracking, sharing nothing with the generator.
fn count_solutions(puzzle: &Grid, limit: u32) -> u32 {
    fn extend(grid: &mut Vec<u8>, box_size: usize, count: &mut u32, limit: u32) {
        if *count >= limit {
            return;
        }
        let side = box_size * box_size;
        let candidates = |grid: &[u8], at: usize| -> Vec<u8> {
            let (row, col) = (at / side, at % side);
            (1..=side as u8)
                .filter(|&digit| {
                    (0..side * side).all(|other| {
                        let (r, c) = (other / side, other % side);
                        let same_box =
                            r / box_size == row / box_size && c / box_size == col / box_size;
                        !(r == row || c == col || same_box) || grid[other] != digit
                    })
                })
                .collect()
        };
        // Taking the most constrained blank keeps this quick enough to run as a test.
        let blank = (0..side * side)
            .filter(|&at| grid[at] == 0)
            .min_by_key(|&at| candidates(grid, at).len());
        let Some(at) = blank else {
            *count += 1;
            return;
        };
        for digit in candidates(grid, at) {
            grid[at] = digit;
            extend(grid, box_size, count, limit);
            grid[at] = 0;
        }
    }

    let mut grid = puzzle.cells().to_vec();
    let mut count = 0;
    extend(&mut grid, puzzle.box_size(), &mut count, limit);
    count
}
