//! The checker: a solver for the puzzle one design makes, asked for one solution after another.
//! Each solution found is blocked, so the next answer is always a new one, and "no more" is a
//! proof that there are none.

use crate::problem::Puzzle;
use sat_core::{Budget, Config, Lit, Solver, Status, Var};

pub struct Checker<'a, P: Puzzle> {
    puzzle: &'a P,
    solver: Solver,
    solution_vars: Vec<Var>,
}

impl<'a, P: Puzzle> Checker<'a, P> {
    pub fn new(puzzle: &'a P, design: &P::Design) -> Self {
        Checker {
            puzzle,
            solver: Solver::from_cnf(&puzzle.clauses(design), Config::default()),
            solution_vars: puzzle.solution_vars(),
        }
    }

    /// Rules out every solution that makes all of `lits` true.
    pub fn exclude(&mut self, lits: &[Lit]) {
        let clause: Vec<Lit> = lits.iter().map(|&lit| !lit).collect();
        self.solver.add_clause(&clause);
    }

    /// A solution not found before, or `None` if every solution has been.
    pub fn next_solution(&mut self) -> Option<P::Solution> {
        if self.solver.solve(Budget::unlimited()) != Status::Sat {
            return None;
        }
        let model = self.solver.model().to_vec();
        let found: Vec<Lit> = self
            .solution_vars
            .iter()
            .map(|&var| {
                if model[var.index()] {
                    var.positive()
                } else {
                    var.negative()
                }
            })
            .collect();
        self.exclude(&found);
        Some(self.puzzle.read_solution(&model))
    }

    pub fn conflicts(&self) -> u64 {
        self.solver.stats().conflicts
    }
}

/// Whether the puzzle a design makes has exactly one solution.
pub fn is_unique<P: Puzzle>(puzzle: &P, design: &P::Design) -> bool {
    let mut checker = Checker::new(puzzle, design);
    checker.next_solution().is_some() && checker.next_solution().is_none()
}
