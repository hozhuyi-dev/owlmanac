//! The synthesis loop. Start reading here: each step it names is one file in `steps/`.
//!
//! One solver proposes a design. Another checks the puzzle it makes for a second solution, and
//! each one it finds becomes a rule for the proposer, so the proposer never makes that mistake
//! again. This is counterexample-guided synthesis: clause learning, one level up.

use crate::data::outcome::Outcome;
use crate::problem::Synthesis;
use crate::synthesizer::Synthesizer;

impl<P: Synthesis> Synthesizer<P> {
    pub(crate) fn synthesize(&mut self, rounds: u32) -> Outcome<P::Design> {
        for _ in 0..rounds {
            self.stats.rounds += 1;
            let Some(design) = self.propose() else {
                return Outcome::Impossible;
            };
            if self.refine(&design) {
                continue;
            }
            match self.check(&design) {
                None => return Outcome::Unique(design),
                Some(other) => self.learn(&other),
            }
        }
        Outcome::OutOfRounds
    }
}
