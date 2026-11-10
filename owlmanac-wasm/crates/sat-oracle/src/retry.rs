//! Generate and test: draw a design at random, keep it if its puzzle has exactly one solution.
//! The simplest strategy, and it only needs `RandomDesign`. How well it does depends entirely on
//! how often a random design happens to be unique.

use crate::data::checker::Checker;
use crate::data::outcome::{Outcome, Stats};
use crate::problem::RandomDesign;
use sat_core::random::Rng;

/// Tries up to `tries` random designs. Never `Impossible`: retrying can't prove nothing works.
pub fn retry<P: RandomDesign>(
    puzzle: &P,
    rng: &mut Rng,
    tries: u32,
) -> (Outcome<P::Design>, Stats) {
    let mut stats = Stats::default();
    for _ in 0..tries {
        stats.rounds += 1;
        let design = puzzle.random_design(rng);
        let mut checker = Checker::new(puzzle, &design);
        let unique = checker.next_solution().is_some() && checker.next_solution().is_none();
        stats.checker_conflicts += checker.conflicts();
        if unique {
            return (Outcome::Unique(design), stats);
        }
    }
    (Outcome::OutOfRounds, stats)
}
