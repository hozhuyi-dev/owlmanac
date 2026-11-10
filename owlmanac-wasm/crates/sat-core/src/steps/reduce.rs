//! Clause deletion: forgets the less useful half of what was learnt, so propagation stays fast.
//!
//! It runs straight after a restart, when the only reasons in use are for facts known at the root.

use crate::data::clause_db::ClauseRef;
use crate::observer::Observer;
use crate::solver::Solver;
use std::cmp::Reverse;

/// Clauses spanning this few decision levels are kept for good.
const ALWAYS_KEEP_LBD: u32 = 2;

impl<O: Observer> Solver<O> {
    pub(crate) fn reduce_clauses_if_due(&mut self) {
        let (Some(at), Some(interval)) = (self.next_reduction, self.config.reduce_interval) else {
            return;
        };
        if self.stats.conflicts < at {
            return;
        }
        self.stats.reductions += 1;
        self.next_reduction =
            Some(self.stats.conflicts + interval + self.stats.reductions * interval / 8);

        let mut candidates: Vec<ClauseRef> = self
            .clauses
            .learnt()
            .iter()
            .copied()
            .filter(|&clause| self.clauses.lbd(clause) > ALWAYS_KEEP_LBD && !self.is_reason(clause))
            .collect();
        candidates.sort_by_key(|&clause| {
            Reverse((self.clauses.lbd(clause), self.clauses.lits(clause).len()))
        });
        candidates.truncate(candidates.len() / 2);

        for clause in candidates {
            self.observer.on_delete(self.clauses.lits(clause));
            self.clauses.delete(clause);
        }
        self.watches.forget_deleted(&self.clauses);
        self.clauses.compact();
    }

    fn is_reason(&self, clause: ClauseRef) -> bool {
        let implied = self.clauses.lits(clause)[0];
        self.trail.is_true(implied) && self.trail.reason_of(implied.var()) == Some(clause)
    }
}
