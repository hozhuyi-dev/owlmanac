//! The proposer's solver, as a puzzle sees it: it can add variables and rules, and read the design
//! found. Everything added stays, so the proposer keeps every lesson it's been taught.

use sat_core::{Budget, Config, Lit, Solver, Status, Var};

pub struct Proposer {
    solver: Solver,
}

impl Proposer {
    pub(crate) fn new() -> Self {
        Proposer {
            solver: Solver::new(Config::default()),
        }
    }

    pub fn new_var(&mut self) -> Var {
        self.solver.new_var()
    }

    pub fn add_clause(&mut self, clause: &[Lit]) {
        self.solver.add_clause(clause);
    }

    pub fn add_clauses(&mut self, clauses: impl IntoIterator<Item = Vec<Lit>>) {
        for clause in clauses {
            self.solver.add_clause(&clause);
        }
    }

    /// A new variable that, when true, makes all of `clauses` hold. Lets a rule say "one of these
    /// sets of clauses holds", by putting the guards in one clause.
    pub fn guard(&mut self, clauses: Vec<Vec<Lit>>) -> Lit {
        let guard = self.solver.new_var().positive();
        for mut clause in clauses {
            clause.push(!guard);
            self.solver.add_clause(&clause);
        }
        guard
    }

    /// Suggests a value to try first, so the proposer starts from a likely design. Only a hint.
    pub fn suggest(&mut self, lit: Lit) {
        self.solver.suggest(lit);
    }

    /// Whether `lit` holds in the design just found.
    pub fn value(&self, lit: Lit) -> bool {
        self.solver.model()[lit.var().index()] == lit.is_positive()
    }

    pub(crate) fn conflicts(&self) -> u64 {
        self.solver.stats().conflicts
    }

    /// Finds a design that follows every rule so far, returning false if none exists.
    pub(crate) fn solve(&mut self) -> bool {
        self.solver.solve(Budget::unlimited()) == Status::Sat
    }
}
