//! Acts on what conflict analysis found: jumps back, keeps the new clause, and asserts its literal.

use crate::observer::Observer;
use crate::solver::Solver;
use crate::steps::analyze::Learnt;

impl<O: Observer> Solver<O> {
    pub(crate) fn backjump(&mut self, level: u32) {
        if level < self.trail.decision_level() {
            self.observer.on_backjump(level);
        }
        let order = &mut self.order;
        self.trail
            .backjump_to(level, |lit| order.on_unassigned(lit));
    }

    /// Stores the clause and assigns its asserting literal, the one option the jump left open.
    pub(crate) fn learn_and_assert(&mut self, learnt: Learnt) {
        self.observer.on_learn(&learnt.lits, learnt.lbd);
        let asserting = learnt.lits[0];
        let reason = match learnt.lits.as_slice() {
            [_] => None,
            [first, second, ..] => {
                let clause = self.clauses.add_learnt(&learnt.lits, learnt.lbd);
                self.watches.watch_clause(clause, *first, *second);
                Some(clause)
            }
            [] => unreachable!("a learnt clause always has its asserting literal"),
        };
        self.trail.assign(asserting, reason);
        self.order.decay();
    }
}
