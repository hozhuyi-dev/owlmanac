//! Every analysis against brute force, on random regions and random boards.

use super::support::brute_force;
use proptest::collection::vec;
use proptest::prelude::*;
use star_battle::{Board, Cell, Mark, Placed, Puzzle, Session, WhyNot};

fn placed_strategy(size: usize) -> impl Strategy<Value = Placed> {
    (0..size, 0..size, any::<bool>()).prop_map(|(row, col, star)| Placed {
        cell: Cell { row, col },
        mark: if star { Mark::Star } else { Mark::Empty },
    })
}

/// A puzzle with random, possibly disconnected regions, plus some marks and a proposal.
fn scenario() -> impl Strategy<Value = (Puzzle, Board, Placed)> {
    (4usize..=6, 1usize..=2).prop_flat_map(|(size, stars)| {
        (
            vec(0..size, size * size),
            vec(placed_strategy(size), 0..=5),
            placed_strategy(size),
        )
            .prop_filter_map(
                "every region needs a cell",
                move |(regions, marks, proposal)| {
                    let puzzle = Puzzle::new(size, stars, regions).ok()?;
                    let mut board = Board::new();
                    marks.into_iter().for_each(|placed| board.place(placed));
                    Some((puzzle, board, proposal))
                },
            )
    })
}

fn allows(solution: &[Cell], placed: Placed) -> bool {
    solution.contains(&placed.cell) == (placed.mark == Mark::Star)
}

fn consistent(solutions: &[Vec<Cell>], board: &Board) -> Vec<Vec<Cell>> {
    solutions
        .iter()
        .filter(|solution| board.placed().all(|placed| allows(solution, placed)))
        .cloned()
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(150))]

    #[test]
    fn solutions_match_brute_force((puzzle, board, _) in scenario()) {
        let expected = consistent(&brute_force(&puzzle), &board);
        let session = Session::new(puzzle);

        let found = session.solutions(&board, usize::MAX);

        let mut sorted = found.found;
        sorted.sort();
        prop_assert_eq!(sorted, expected);
        prop_assert!(found.complete);
    }

    #[test]
    fn the_limit_stops_the_count_honestly((puzzle, board, _) in scenario()) {
        let total = consistent(&brute_force(&puzzle), &board).len();
        let session = Session::new(puzzle);

        let found = session.solutions(&board, 2);

        prop_assert_eq!(found.found.len(), total.min(2));
        prop_assert_eq!(found.complete, total <= 2);
    }

    #[test]
    fn a_board_is_solvable_exactly_when_it_has_a_solution((puzzle, board, _) in scenario()) {
        let expected = !consistent(&brute_force(&puzzle), &board).is_empty();
        let mut session = Session::new(puzzle);

        prop_assert_eq!(session.is_solvable(&board), expected);
        prop_assert_eq!(session.blame(&board).is_none(), expected);
    }

    #[test]
    fn forced_cells_are_what_every_solution_agrees_on((puzzle, board, _) in scenario()) {
        let solutions = consistent(&brute_force(&puzzle), &board);
        let cells: Vec<Cell> = puzzle.cells().collect();
        let mut session = Session::new(puzzle);

        let forced = session.forced(&board);

        prop_assert_eq!(forced.is_none(), solutions.is_empty());
        if let Some(forced) = forced {
            for (cell, mark) in cells.iter().zip(forced) {
                let stars = solutions.iter().filter(|s| s.contains(cell)).count();
                let expected = if stars == solutions.len() {
                    Some(Mark::Star)
                } else if stars == 0 {
                    Some(Mark::Empty)
                } else {
                    None
                };
                prop_assert_eq!(mark, expected);
            }
        }
    }

    #[test]
    fn why_not_says_possible_exactly_when_it_is((puzzle, board, proposal) in scenario()) {
        let mut without = board.clone();
        without.clear(proposal.cell);
        let expected = !consistent(&brute_force(&puzzle), &without.with(proposal)).is_empty();
        let mut session = Session::new(puzzle);

        let answer = session.why_not(&board, proposal);

        prop_assert_eq!(answer == WhyNot::Possible, expected);
    }

    #[test]
    fn blamed_marks_alone_are_enough_to_sink_the_board((puzzle, board, _) in scenario()) {
        let mut session = Session::new(puzzle);
        if let Some(blame) = session.blame(&board) {
            prop_assert!(blame.marks.iter().all(|placed| board.mark_at(placed.cell) == Some(placed.mark)));

            let mut only_blamed = Board::new();
            blame.marks.iter().for_each(|&placed| only_blamed.place(placed));
            prop_assert!(!session.is_solvable(&only_blamed));
        }
    }
}
