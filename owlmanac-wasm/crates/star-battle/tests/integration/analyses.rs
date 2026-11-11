//! Each analysis on puzzles whose single solution is known.

use super::support::{
    Fixture, eight_by_eight_two_stars, every_fixture, five_by_five, star, ten_by_ten_two_stars,
    touching,
};
use star_battle::{Board, Cell, Chain, Mark, Placed, Puzzle, Rule, Session, WhyNot};
use std::collections::BTreeSet;

fn wrong_cells(fixture: &Fixture) -> Vec<Cell> {
    fixture
        .puzzle
        .cells()
        .filter(|cell| !fixture.solution.contains(cell))
        .collect()
}

fn rule_covers(puzzle: &Puzzle, rule: Rule, cell: Cell) -> bool {
    match rule {
        Rule::Row(row) => cell.row == row,
        Rule::Column(col) => cell.col == col,
        Rule::Region(region) => puzzle.region_of(cell) == region,
        Rule::NoTouching(origin) => cell == origin || touching(cell, origin),
    }
}

/// A chain can be read out in order: each step rests only on the board, the proposal or an
/// earlier step, and only ever invokes a rule about the cells it mentions.
fn assert_chain_reads_in_order(puzzle: &Puzzle, board: &Board, proposal: Placed, chain: &Chain) {
    let mut known: BTreeSet<Placed> = board.placed().collect();
    known.insert(proposal);
    for step in &chain.steps {
        assert!(step.because.iter().all(|mark| known.contains(mark)));
        assert!(rule_covers(puzzle, step.rule, step.placed.cell));
        assert!(
            step.because
                .iter()
                .all(|mark| rule_covers(puzzle, step.rule, mark.cell))
        );
        known.insert(step.placed);
    }
    assert!(chain.clash.marks.iter().all(|mark| known.contains(mark)));
    assert!(
        chain
            .clash
            .marks
            .iter()
            .all(|mark| rule_covers(puzzle, chain.clash.rule, mark.cell))
    );
}

#[test]
fn each_fixture_has_exactly_its_known_solution() {
    for fixture in every_fixture() {
        let session = Session::new(fixture.puzzle);
        let found = session.solutions(&Board::new(), 2);
        assert_eq!(found.found, vec![fixture.solution]);
        assert!(found.complete);
    }
}

#[test]
fn a_single_wrong_star_is_caught_at_once() {
    for fixture in every_fixture() {
        let wrong = wrong_cells(&fixture);
        let right = fixture.solution[0];
        let mut session = Session::new(fixture.puzzle);

        assert!(session.is_solvable(&Board::new()));
        assert!(session.is_solvable(&Board::new().with(star(right))));
        for cell in wrong {
            assert!(!session.is_solvable(&Board::new().with(star(cell))));
        }
    }
}

#[test]
fn a_correct_star_far_from_a_wrong_one_is_not_blamed() {
    let fixture = ten_by_ten_two_stars();
    let wrong = wrong_cells(&fixture)[0];
    let right = fixture.solution[5];
    let mut session = Session::new(fixture.puzzle);
    let board = Board::new().with(star(right)).with(star(wrong));

    let blame = session.blame(&board).expect("the board is dead");

    assert_eq!(blame.marks, vec![star(wrong)]);
    assert!(!blame.rules.is_empty());
}

#[test]
fn a_finishable_board_has_nothing_to_blame() {
    let fixture = five_by_five();
    let right = fixture.solution[0];
    let mut session = Session::new(fixture.puzzle);

    assert_eq!(session.blame(&Board::new().with(star(right))), None);
}

#[test]
fn every_cell_of_a_proper_puzzle_is_forced() {
    for fixture in every_fixture() {
        let cells: Vec<Cell> = fixture.puzzle.cells().collect();
        let mut session = Session::new(fixture.puzzle);

        let forced = session.forced(&Board::new()).expect("it has a solution");

        for (cell, mark) in cells.iter().zip(forced) {
            let expected = if fixture.solution.contains(cell) {
                Mark::Star
            } else {
                Mark::Empty
            };
            assert_eq!(mark, Some(expected), "{cell:?}");
        }
    }
}

#[test]
fn touching_a_star_names_the_rule() {
    let fixture = five_by_five();
    let existing = fixture.solution[1];
    // Diagonal, in another region, so the touching rule is the only one it breaks.
    let neighbour = Cell {
        row: existing.row + 1,
        col: existing.col - 1,
    };
    let mut session = Session::new(fixture.puzzle);
    let board = Board::new().with(star(existing));

    let WhyNot::Deduced(chain) = session.why_not(&board, star(neighbour)) else {
        panic!("a star beside a star is an immediate clash");
    };

    assert_eq!(chain.clash.rule, Rule::NoTouching(existing));
    assert!(chain.clash.marks.contains(&star(existing)));
    assert!(chain.clash.marks.contains(&star(neighbour)));
}

#[test]
fn a_cell_in_the_solution_is_possible() {
    for fixture in every_fixture() {
        let cell = fixture.solution[0];
        let mut session = Session::new(fixture.puzzle);
        assert_eq!(session.why_not(&Board::new(), star(cell)), WhyNot::Possible);
    }
}

#[test]
fn asking_about_a_marked_cell_replaces_the_mark() {
    let fixture = five_by_five();
    let cell = fixture.solution[0];
    let mut session = Session::new(fixture.puzzle);
    let board = Board::new().with(Placed {
        cell,
        mark: Mark::Empty,
    });

    assert_eq!(session.why_not(&board, star(cell)), WhyNot::Possible);
}

#[test]
fn every_wrong_cell_gets_an_explanation_that_reads_in_order() {
    for fixture in every_fixture() {
        let wrong = wrong_cells(&fixture);
        let puzzle = fixture.puzzle.clone();
        let mut session = Session::new(fixture.puzzle);

        for cell in wrong {
            match session.why_not(&Board::new(), star(cell)) {
                WhyNot::Possible => panic!("{cell:?} is not in the only solution"),
                WhyNot::Deduced(chain) => {
                    assert_chain_reads_in_order(&puzzle, &Board::new(), star(cell), &chain)
                }
                WhyNot::NeedsCases(blame) => {
                    assert_eq!(blame.marks, vec![star(cell)]);
                    assert!(!blame.rules.is_empty());
                }
            }
        }
    }
}

#[test]
fn the_large_puzzle_has_chains_and_cases_alike() {
    let fixture = ten_by_ten_two_stars();
    let wrong = wrong_cells(&fixture);
    let mut session = Session::new(fixture.puzzle);

    let answers: Vec<WhyNot> = wrong
        .into_iter()
        .map(|cell| session.why_not(&Board::new(), star(cell)))
        .collect();

    assert!(
        answers
            .iter()
            .any(|answer| matches!(answer, WhyNot::Deduced(chain) if !chain.steps.is_empty()))
    );
    assert!(
        answers
            .iter()
            .any(|answer| matches!(answer, WhyNot::NeedsCases(_)))
    );
}

#[test]
fn propagation_alone_settles_the_easy_puzzle_but_not_the_hard_one() {
    assert!(
        Session::new(five_by_five().puzzle)
            .learned_rules(1000)
            .is_empty()
    );
    assert!(
        !Session::new(ten_by_ten_two_stars().puzzle)
            .learned_rules(1000)
            .is_empty()
    );
}

#[test]
fn no_solution_breaks_a_learned_rule() {
    let fixture = ten_by_ten_two_stars();
    let session = Session::new(fixture.puzzle);

    let learned = session.learned_rules(1000);

    for rule in learned {
        let all_hold = rule
            .cannot_all_hold
            .iter()
            .all(|placed| fixture.solution.contains(&placed.cell) == (placed.mark == Mark::Star));
        assert!(!all_hold, "{rule:?}");
    }
}

#[test]
fn the_learned_limit_is_respected() {
    let session = Session::new(ten_by_ten_two_stars().puzzle);
    assert_eq!(session.learned_rules(3).len(), 3);
}

#[test]
fn an_impossible_puzzle_has_no_solutions_and_says_so() {
    // Two rows can't each hold a star without touching.
    let mut session = Session::new(Puzzle::parse(1, "ab\nab").unwrap());

    assert!(!session.is_solvable(&Board::new()));
    assert_eq!(session.forced(&Board::new()), None);
    let blame = session.blame(&Board::new()).expect("it can't be solved");
    assert!(blame.marks.is_empty());
    assert!(!blame.rules.is_empty());
}

#[test]
fn the_fixtures_grade_as_measured() {
    let grade = |fixture: Fixture| {
        let grade = Session::new(fixture.puzzle).grade(&Board::new()).unwrap();
        (grade.depth, grade.length)
    };
    assert_eq!(grade(five_by_five()), (Some(0), 0));
    assert_eq!(grade(eight_by_eight_two_stars()), (Some(1), 1));
    assert_eq!(grade(ten_by_ten_two_stars()), (Some(1), 2));
}

#[test]
fn following_hints_solves_each_puzzle() {
    for fixture in every_fixture() {
        let session = Session::new(fixture.puzzle.clone());
        let mut board = Board::new();
        let mut deepest = 0;
        while let Some(hint) = session.hint(&board) {
            let in_solution = fixture.solution.contains(&hint.placed.cell);
            assert_eq!(in_solution, hint.placed.mark == Mark::Star, "{hint:?}");
            assert_eq!(
                board.mark_at(hint.placed.cell),
                None,
                "a hint is always a new cell"
            );
            deepest = deepest.max(hint.depth);
            board.place(hint.placed);
        }
        let cells = fixture.puzzle.cells().count();
        assert_eq!(board.placed().count(), cells, "hints fill the whole grid");
        let grade = session.grade(&Board::new()).unwrap().depth.unwrap();
        assert!(
            deepest <= grade,
            "no hint is deeper than the puzzle's grade"
        );
    }
}

#[test]
fn an_easy_puzzle_starts_with_a_forced_move() {
    let session = Session::new(five_by_five().puzzle);
    assert_eq!(session.hint(&Board::new()).unwrap().depth, 0);
}

#[test]
fn a_dead_or_finished_board_has_no_hint() {
    let fixture = five_by_five();
    let wrong = wrong_cells(&fixture)[0];
    let session = Session::new(fixture.puzzle.clone());
    assert_eq!(session.hint(&Board::new().with(star(wrong))), None);

    let mut finished = Board::new();
    for cell in fixture.puzzle.cells() {
        let mark = if fixture.solution.contains(&cell) {
            Mark::Star
        } else {
            Mark::Empty
        };
        finished.place(Placed { cell, mark });
    }
    assert_eq!(session.hint(&finished), None);
}
