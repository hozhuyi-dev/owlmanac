//! Which marks can't stand together, and under which rules. The set is minimal, with nothing to
//! spare, but not necessarily the smallest: two touching stars can be blamed together even if
//! one of them is impossible on its own. Either way, at least one blamed mark is wrong.

use crate::board::{Board, Placed};
use crate::rules::Rule;
use crate::session::Session;
use sat_core::{Budget, Lit, Status};
use serde::Serialize;

/// Together these make the board impossible, and dropping any one of them would not.
#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct Blame {
    pub marks: Vec<Placed>,
    pub rules: Vec<Rule>,
}

impl Session {
    /// Returns `None` if the board can still be finished.
    pub fn blame(&mut self, board: &Board) -> Option<Blame> {
        // Marks go first so they are the first dropped, keeping the accusation against the player short.
        let mut assumptions = self.literals(board);
        assumptions.extend_from_slice(self.blame.guards());
        if self.solve_guarded(&assumptions) == Status::Sat {
            return None;
        }
        let core = self.shrink_to_minimal(&assumptions);
        Some(self.describe(&core))
    }

    fn solve_guarded(&mut self, assumptions: &[Lit]) -> Status {
        self.blame
            .solver
            .solve_assuming(assumptions, Budget::unlimited())
    }

    /// Tries removing each assumption in turn, keeping the removal whenever the rest still fail.
    /// `assumptions` must have just failed.
    fn shrink_to_minimal(&mut self, assumptions: &[Lit]) -> Vec<Lit> {
        let mut core = self.failed_among(assumptions);
        let mut index = 0;
        while index < core.len() {
            let mut trial = core.clone();
            trial.remove(index);
            if self.solve_guarded(&trial) == Status::Unsat {
                core = self.failed_among(&trial);
            } else {
                index += 1;
            }
        }
        core
    }

    /// The solver's failed assumptions, in the order they were given.
    fn failed_among(&self, assumptions: &[Lit]) -> Vec<Lit> {
        let failed = self.blame.solver.failed_assumptions();
        assumptions
            .iter()
            .copied()
            .filter(|lit| failed.contains(lit))
            .collect()
    }

    fn describe(&self, core: &[Lit]) -> Blame {
        let mut blame = Blame {
            marks: Vec::new(),
            rules: Vec::new(),
        };
        for &lit in core {
            match self.blame.rule_of(lit) {
                Some(rule) => blame.rules.push(rule),
                None => blame.marks.push(self.encoding.placed(lit)),
            }
        }
        blame
    }
}
