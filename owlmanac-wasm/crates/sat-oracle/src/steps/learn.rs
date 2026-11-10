//! Turns a second solution into a rule for the proposer, so no later design allows it.

use crate::problem::Synthesis;
use crate::synthesizer::Synthesizer;

impl<P: Synthesis> Synthesizer<P> {
    pub(crate) fn learn(&mut self, other: &P::Solution) {
        self.stats.counterexamples += 1;
        self.problem.rule_out(other, &mut self.proposer);
    }
}
