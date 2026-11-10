//! Propagation without search: what follows immediately from some assumptions, and why. No guess
//! is ever made, so every step can be shown to a reader as a plain deduction.

use crate::data::clause_db::ClauseRef;
use crate::data::lit::Lit;
use crate::observer::Observer;
use crate::solver::Solver;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Deduction {
    pub lit: Lit,
    /// 0 for facts that hold without any assumption.
    pub level: u32,
    /// The clause that forced `lit`: its other literals are all false by this step. `None` for an
    /// assumption or a fact the formula states outright.
    pub reason: Option<Vec<Lit>>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Contradiction {
    /// Every literal of this clause is false.
    Clause(Vec<Lit>),
    /// This assumption was already false when its turn came.
    Assumption(Lit),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Deductions {
    /// Everything assigned, in the order it followed.
    pub steps: Vec<Deduction>,
    pub contradiction: Option<Contradiction>,
}

impl<O: Observer> Solver<O> {
    /// Places `assumptions` and propagates, stopping at the first contradiction. Abandons any
    /// search in progress, and leaves nothing behind.
    pub fn propagate_assuming(&mut self, assumptions: &[Lit]) -> Deductions {
        self.backjump(0);
        let contradiction = self.place_and_propagate(assumptions);
        let steps = (0..self.trail.len())
            .map(|position| self.deduction_at(position))
            .collect();
        self.backjump(0);
        Deductions {
            steps,
            contradiction,
        }
    }

    fn place_and_propagate(&mut self, assumptions: &[Lit]) -> Option<Contradiction> {
        if self.proven_unsat {
            return Some(Contradiction::Clause(Vec::new()));
        }
        if let Some(conflict) = self.propagate() {
            self.mark_unsat();
            return Some(self.contradiction_from(conflict));
        }
        for &assumption in assumptions {
            if self.trail.is_false(assumption) {
                return Some(Contradiction::Assumption(assumption));
            }
            if self.trail.is_true(assumption) {
                continue;
            }
            self.decide(assumption);
            if let Some(conflict) = self.propagate() {
                return Some(self.contradiction_from(conflict));
            }
        }
        None
    }

    fn contradiction_from(&self, conflict: ClauseRef) -> Contradiction {
        Contradiction::Clause(self.clauses.lits(conflict).to_vec())
    }

    fn deduction_at(&self, position: usize) -> Deduction {
        let lit = self.trail.literal_at(position);
        Deduction {
            lit,
            level: self.trail.level_of(lit.var()),
            reason: self
                .trail
                .reason_of(lit.var())
                .map(|reason| self.clauses.lits(reason).to_vec()),
        }
    }
}
