//! Rejects a design that breaks a rule checked outside the solver, teaching the proposer a cut.
//!
//! Some rules are much easier to check than to write as clauses, like a region being in one piece.
//! Rather than encode them in full, the proposer is told only about the ways it has broken them.

use crate::problem::Synthesis;
use crate::synthesizer::Synthesizer;

impl<P: Synthesis> Synthesizer<P> {
    /// Returns whether the design was rejected.
    pub(crate) fn refine(&mut self, design: &P::Design) -> bool {
        let cuts = self.problem.cuts(design);
        if cuts.is_empty() {
            return false;
        }
        self.stats.cuts += 1;
        self.proposer.add_clauses(cuts);
        true
    }
}
