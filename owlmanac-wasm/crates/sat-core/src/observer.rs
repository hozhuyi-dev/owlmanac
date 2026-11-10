//! Hooks the search calls as it runs, for traces and proofs. Every hook does nothing by default,
//! and `Solver` is generic over its observer, so the default `NoObserver` has no dispatch to pay for.

use crate::data::lit::Lit;

pub trait Observer {
    fn on_decide(&mut self, _lit: Lit) {}

    /// `reason` is the clause that forced `lit`.
    fn on_propagate(&mut self, _lit: Lit, _reason: &[Lit]) {}

    /// Every literal of `clause` is false.
    fn on_conflict(&mut self, _clause: &[Lit]) {}

    /// Conflict analysis used `clause`. These, after `on_conflict`, are what the next `on_learn` is
    /// derived from.
    fn on_resolve(&mut self, _clause: &[Lit]) {}

    /// `clause[0]` is assigned straight after the backjump this clause called for.
    fn on_learn(&mut self, _clause: &[Lit], _lbd: u32) {}

    fn on_backjump(&mut self, _level: u32) {}

    fn on_restart(&mut self) {}

    fn on_delete(&mut self, _clause: &[Lit]) {}

    /// The formula has no solution, whatever is assumed.
    fn on_unsat(&mut self) {}
}

#[derive(Clone, Copy, Default, Debug)]
pub struct NoObserver;

impl Observer for NoObserver {}

/// Two observers side by side, for instance a proof and a trace.
impl<A: Observer, B: Observer> Observer for (A, B) {
    fn on_decide(&mut self, lit: Lit) {
        self.0.on_decide(lit);
        self.1.on_decide(lit);
    }

    fn on_propagate(&mut self, lit: Lit, reason: &[Lit]) {
        self.0.on_propagate(lit, reason);
        self.1.on_propagate(lit, reason);
    }

    fn on_conflict(&mut self, clause: &[Lit]) {
        self.0.on_conflict(clause);
        self.1.on_conflict(clause);
    }

    fn on_resolve(&mut self, clause: &[Lit]) {
        self.0.on_resolve(clause);
        self.1.on_resolve(clause);
    }

    fn on_learn(&mut self, clause: &[Lit], lbd: u32) {
        self.0.on_learn(clause, lbd);
        self.1.on_learn(clause, lbd);
    }

    fn on_backjump(&mut self, level: u32) {
        self.0.on_backjump(level);
        self.1.on_backjump(level);
    }

    fn on_restart(&mut self) {
        self.0.on_restart();
        self.1.on_restart();
    }

    fn on_delete(&mut self, clause: &[Lit]) {
        self.0.on_delete(clause);
        self.1.on_delete(clause);
    }

    fn on_unsat(&mut self) {
        self.0.on_unsat();
        self.1.on_unsat();
    }
}
