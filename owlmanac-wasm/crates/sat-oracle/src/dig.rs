//! Remove clues while the answer stays unique: how Sudoku makers work.
//!
//! Start with every clue shown, so the intended solution is the only one. Then try taking clues
//! away in random order, putting each back if a second solution appears. A clue can only rule
//! solutions out, so once a clue has to stay, it always will: the result is a puzzle where every
//! clue is needed.
//!
//! One solver serves every check. Each clue hangs off a guard variable, assumed true while the
//! clue is shown, and one permanent clause rules out the intended solution, so "unique" is simply
//! "no solution with these guards", and what the solver learns carries from one check to the next.

use crate::data::outcome::Stats;
use crate::problem::Clues;
use sat_core::random::Rng;
use sat_core::{Budget, Config, Lit, Solver, Status};

/// Which clues to show, by index, after digging, until no more can go or only `keep_at_least`
/// remain.
pub fn dig<P: Clues>(puzzle: &P, rng: &mut Rng, keep_at_least: usize) -> (Vec<bool>, Stats) {
    let mut stats = Stats::default();
    let mut solver = Solver::from_cnf(&puzzle.rules(), Config::default());
    let guards: Vec<Lit> = (0..puzzle.clue_count())
        .map(|index| {
            let guard = solver.new_var().positive();
            for mut clause in puzzle.clue(index) {
                clause.push(!guard);
                solver.add_clause(&clause);
            }
            guard
        })
        .collect();
    let not_intended: Vec<Lit> = puzzle.intended().iter().map(|&lit| !lit).collect();
    solver.add_clause(&not_intended);

    let mut shown = vec![true; puzzle.clue_count()];
    let mut remaining = shown.len();
    for index in shuffled(shown.len(), rng) {
        if remaining <= keep_at_least {
            break;
        }
        stats.rounds += 1;
        shown[index] = false;
        let assumptions: Vec<Lit> = guards
            .iter()
            .zip(&shown)
            .filter(|&(_, &kept)| kept)
            .map(|(&guard, _)| guard)
            .collect();
        if solver.solve_assuming(&assumptions, Budget::unlimited()) == Status::Unsat {
            remaining -= 1;
        } else {
            stats.counterexamples += 1;
            shown[index] = true;
        }
    }
    stats.checker_conflicts = solver.stats().conflicts;
    (shown, stats)
}

fn shuffled(count: usize, rng: &mut Rng) -> Vec<usize> {
    let mut order: Vec<usize> = (0..count).collect();
    for last in (1..count).rev() {
        order.swap(last, rng.below(last as u32 + 1) as usize);
    }
    order
}
