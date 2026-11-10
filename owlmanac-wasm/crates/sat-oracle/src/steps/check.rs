//! Does the puzzle this design makes have a solution other than the intended one?
//!
//! "No" is the answer that matters, and it's a proof: the checker has shown the puzzle, plus "not
//! the intended solution", has no solution at all.

use crate::data::checker::Checker;
use crate::problem::Synthesis;
use crate::synthesizer::Synthesizer;

impl<P: Synthesis> Synthesizer<P> {
    /// Returns the other solution, or `None` if the intended one is the only one.
    pub(crate) fn check(&mut self, design: &P::Design) -> Option<P::Solution> {
        let mut checker = Checker::new(&self.problem, design);
        checker.exclude(&self.problem.intended());
        let other = checker.next_solution();
        self.stats.checker_conflicts += checker.conflicts();
        other
    }
}
