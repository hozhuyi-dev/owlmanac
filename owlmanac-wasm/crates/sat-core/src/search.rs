//! The CDCL loop. Start reading here: each step it names is one file in `steps/`.

use crate::observer::Observer;
use crate::solver::{Budget, Solver, Status};
use crate::steps::decide::Decision;

impl<O: Observer> Solver<O> {
    /// Runs until the formula is decided or `budget` is spent. Everything the search needs lives in
    /// `self`, so calling this again after `Unknown` carries on where it stopped.
    pub(crate) fn search(&mut self, budget: &mut Budget) -> Status {
        loop {
            if let Some(conflict) = self.propagate() {
                if self.trail.at_root() {
                    return self.mark_unsat();
                }
                let learnt = self.analyze(conflict);
                self.backjump(learnt.backjump_level);
                self.learn_and_assert(learnt);
                budget.spend_conflict();
                continue;
            }

            if budget.exhausted() {
                return Status::Unknown;
            }
            if self.restart_due() {
                self.restart();
                self.reduce_clauses_if_due();
                continue;
            }

            match self.next_decision() {
                Decision::Branch(lit) => self.decide(lit),
                Decision::AllAssigned => return Status::Sat,
                Decision::AssumptionFailed(lit) => return self.fail_assumption(lit),
            }
        }
    }
}
