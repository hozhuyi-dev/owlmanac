//! Asks the proposer for a design that follows every rule it has been given so far.

use crate::problem::Synthesis;
use crate::synthesizer::Synthesizer;

impl<P: Synthesis> Synthesizer<P> {
    /// Returns `None` once the rules leave no design at all.
    pub(crate) fn propose(&mut self) -> Option<P::Design> {
        let found = self.proposer.solve();
        self.stats.proposer_conflicts = self.proposer.conflicts();
        if !found {
            return None;
        }
        Some(self.problem.read_design(&self.proposer))
    }
}
