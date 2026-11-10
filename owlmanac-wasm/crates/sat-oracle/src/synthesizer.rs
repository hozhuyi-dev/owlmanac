//! The synthesizer's state and public interface. The algorithm itself starts in `synthesis.rs`.

use crate::data::outcome::{Outcome, Stats};
use crate::data::proposer::Proposer;
use crate::problem::Synthesis;

pub struct Synthesizer<P: Synthesis> {
    pub(crate) problem: P,
    /// Finds designs, and learns from every one that fails.
    pub(crate) proposer: Proposer,
    pub(crate) stats: Stats,
}

impl<P: Synthesis> Synthesizer<P> {
    pub fn new(problem: P) -> Self {
        let mut proposer = Proposer::new();
        problem.define(&mut proposer);
        Synthesizer {
            problem,
            proposer,
            stats: Stats::default(),
        }
    }

    /// Runs for at most `rounds` proposals. Everything learnt stays, so calling it again after
    /// `OutOfRounds` carries on.
    pub fn run(&mut self, rounds: u32) -> Outcome<P::Design> {
        self.synthesize(rounds)
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }

    pub fn problem(&self) -> &P {
        &self.problem
    }
}
