//! What the solver works out for itself: each learnt clause is a new rule of this puzzle, and
//! because every literal is a cell it can be drawn on the grid.

use crate::board::Placed;
use crate::session::Session;
use sat_core::{Budget, Config, Lit, Observer, Solver};
use serde::Serialize;

#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearnedRule {
    /// These can never all be true in a solution.
    pub cannot_all_hold: Vec<Placed>,
    /// How many decision levels it spanned when learnt: lower is a more general rule.
    pub lbd: u32,
}

struct LearntLog {
    clauses: Vec<(Vec<Lit>, u32)>,
    limit: usize,
}

impl Observer for LearntLog {
    fn on_learn(&mut self, clause: &[Lit], lbd: u32) {
        if self.clauses.len() < self.limit {
            self.clauses.push((clause.to_vec(), lbd));
        }
    }
}

impl Session {
    /// Solves the puzzle from scratch and reports the first `limit` clauses it learnt.
    pub fn learned_rules(&self, limit: usize) -> Vec<LearnedRule> {
        let log = LearntLog {
            clauses: Vec::new(),
            limit,
        };
        let mut solver = Solver::with_observer(Config::default(), log);
        self.encoding.load(&mut solver);
        solver.solve(Budget::unlimited());

        solver
            .observer()
            .clauses
            .iter()
            .map(|(clause, lbd)| LearnedRule {
                cannot_all_hold: clause
                    .iter()
                    .map(|&lit| self.encoding.placed(!lit))
                    .collect(),
                lbd: *lbd,
            })
            .collect()
    }
}
