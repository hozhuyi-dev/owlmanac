//! DPLL: unit propagation plus plain backtracking. It learns nothing from a conflict, so it has
//! to flip decisions one by one, latest first, to be sure no branch is skipped.

use crate::observer::Observer;
use crate::solver::{Budget, Solver, Status};
use crate::steps::decide::Decision;

impl<O: Observer> Solver<O> {
    pub(crate) fn search_dpll(&mut self, budget: &mut Budget) -> Status {
        loop {
            if self.propagate().is_some() {
                self.stats.conflicts += 1;
                budget.spend_conflict();
                if !self.flip_latest_decision() {
                    return self.mark_unsat();
                }
                continue;
            }

            if budget.exhausted() {
                return Status::Unknown;
            }

            match self.next_decision() {
                Decision::Branch(lit) => {
                    self.decide(lit);
                    self.flipped.push(false);
                }
                Decision::AllAssigned => return Status::Sat,
                Decision::AssumptionFailed(_) => unreachable!("DPLL takes no assumptions"),
            }
        }
    }

    /// Undoes decisions until one hasn't tried its other value, then tries it. Returns false when
    /// every decision has been tried both ways.
    fn flip_latest_decision(&mut self) -> bool {
        while let Some(already_flipped) = self.flipped.pop() {
            let level = self.flipped.len() as u32;
            let decision = self.trail.decision_at(level + 1);
            self.backjump(level);
            if !already_flipped {
                self.trail.open_level();
                self.trail.assign(!decision, None);
                self.flipped.push(true);
                return true;
            }
        }
        false
    }
}
