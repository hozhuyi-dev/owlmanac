//! Picks the next guess once propagation has nothing left to force.

use crate::data::lit::Lit;
use crate::observer::Observer;
use crate::solver::Solver;

pub(crate) enum Decision {
    Branch(Lit),
    AllAssigned,
    /// This assumption is already false, so the assumptions can't all hold.
    AssumptionFailed(Lit),
}

impl<O: Observer> Solver<O> {
    /// Places the assumptions first, one per decision level, then branches freely.
    pub(crate) fn next_decision(&mut self) -> Decision {
        if let Some(decision) = self.next_assumption() {
            return decision;
        }
        while let Some(var) = self.order.pop() {
            if self.trail.is_unassigned(var) {
                return Decision::Branch(self.order.preferred_literal(var));
            }
        }
        Decision::AllAssigned
    }

    fn next_assumption(&mut self) -> Option<Decision> {
        while (self.trail.decision_level() as usize) < self.assumptions.len() {
            let assumption = self.assumptions[self.trail.decision_level() as usize];
            if self.trail.is_false(assumption) {
                return Some(Decision::AssumptionFailed(assumption));
            }
            if !self.trail.is_true(assumption) {
                return Some(Decision::Branch(assumption));
            }
            // Already true: an empty level keeps each assumption at its own level.
            self.trail.open_level();
        }
        None
    }

    /// Opens a new decision level with `lit` as its guess.
    pub(crate) fn decide(&mut self, lit: Lit) {
        self.stats.decisions += 1;
        self.observer.on_decide(lit);
        self.trail.open_level();
        self.trail.assign(lit, None);
    }
}
