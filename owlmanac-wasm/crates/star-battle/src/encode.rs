//! Turns a puzzle into clauses. There is one variable per cell, true for a star, and no helper
//! variables, so every literal the solver ever works with, learnt clauses included, is a cell.

use crate::board::{Mark, Placed};
use crate::puzzle::{Cell, Puzzle};
use crate::rules::Rule;
use sat_core::cardinality::{at_most, exactly};
use sat_core::{Cnf, Config, Lit, NoObserver, Observer, Solver, Var};

struct RuleClauses {
    rule: Rule,
    clauses: Vec<Vec<Lit>>,
}

pub(crate) struct Encoding {
    size: usize,
    groups: Vec<RuleClauses>,
}

impl Encoding {
    pub fn new(puzzle: &Puzzle) -> Self {
        let size = puzzle.size();
        let var = |cell: Cell| Var::new(puzzle.index(cell) as u32).positive();
        let cells_where = |matches: &dyn Fn(Cell) -> bool| -> Vec<Lit> {
            puzzle.cells().filter(|&c| matches(c)).map(var).collect()
        };
        let exactly_stars = |lits: Vec<Lit>| exactly(&lits, puzzle.stars());

        let mut groups = Vec::new();
        for index in 0..size {
            groups.push(RuleClauses {
                rule: Rule::Row(index),
                clauses: exactly_stars(cells_where(&|cell| cell.row == index)),
            });
            groups.push(RuleClauses {
                rule: Rule::Column(index),
                clauses: exactly_stars(cells_where(&|cell| cell.col == index)),
            });
            groups.push(RuleClauses {
                rule: Rule::Region(index),
                clauses: exactly_stars(cells_where(&|cell| puzzle.region_of(cell) == index)),
            });
        }
        for cell in puzzle.cells() {
            let clauses: Vec<Vec<Lit>> = later_neighbours(cell, size)
                .into_iter()
                .flat_map(|neighbour| at_most(&[var(cell), var(neighbour)], 1))
                .collect();
            if !clauses.is_empty() {
                groups.push(RuleClauses {
                    rule: Rule::NoTouching(cell),
                    clauses,
                });
            }
        }
        Encoding { size, groups }
    }

    pub fn num_cells(&self) -> usize {
        self.size * self.size
    }

    pub fn lit(&self, placed: Placed) -> Lit {
        let var = Var::new((placed.cell.row * self.size + placed.cell.col) as u32);
        match placed.mark {
            Mark::Star => var.positive(),
            Mark::Empty => var.negative(),
        }
    }

    /// The placement a cell literal stands for.
    pub fn placed(&self, lit: Lit) -> Placed {
        let index = lit.var().index();
        Placed {
            cell: Cell {
                row: index / self.size,
                col: index % self.size,
            },
            mark: if lit.is_positive() {
                Mark::Star
            } else {
                Mark::Empty
            },
        }
    }

    /// Every rule's clauses, over the cell variables.
    pub fn cnf(&self) -> Cnf {
        Cnf {
            num_vars: self.num_cells() as u32,
            clauses: self
                .groups
                .iter()
                .flat_map(|group| group.clauses.clone())
                .collect(),
        }
    }

    pub fn plain_solver(&self) -> Solver {
        let mut solver = Solver::new(Config::default());
        self.load(&mut solver);
        solver
    }

    /// Adds the cell variables and every rule's clauses.
    pub fn load<O: Observer>(&self, solver: &mut Solver<O>) {
        for _ in 0..self.num_cells() {
            solver.new_var();
        }
        for clause in self.groups.iter().flat_map(|group| &group.clauses) {
            solver.add_clause(clause);
        }
    }
}

/// The neighbours of `cell` that come after it row by row, so each touching pair appears once.
fn later_neighbours(cell: Cell, size: usize) -> Vec<Cell> {
    let mut neighbours = Vec::new();
    if cell.col + 1 < size {
        neighbours.push(Cell {
            row: cell.row,
            col: cell.col + 1,
        });
    }
    if cell.row + 1 < size {
        let first = cell.col.saturating_sub(1);
        let last = (cell.col + 1).min(size - 1);
        neighbours.extend((first..=last).map(|col| Cell {
            row: cell.row + 1,
            col,
        }));
    }
    neighbours
}

/// A solver where each rule hangs off a guard variable, assumed true for every query. A proof of
/// unsatisfiability then has to name the guards it used, which says which rules were involved.
pub(crate) struct Guarded {
    pub solver: Solver<NoObserver>,
    guards: Vec<Lit>,
    rules: Vec<Rule>,
    num_cells: usize,
}

impl Guarded {
    pub fn new(encoding: &Encoding) -> Self {
        let mut solver = Solver::new(Config::default());
        for _ in 0..encoding.num_cells() {
            solver.new_var();
        }

        let mut guards = Vec::new();
        let mut rules = Vec::new();
        for group in &encoding.groups {
            let guard = solver.new_var().positive();
            guards.push(guard);
            rules.push(group.rule);
            for clause in &group.clauses {
                let mut guarded = clause.clone();
                guarded.push(!guard);
                solver.add_clause(&guarded);
            }
        }
        Guarded {
            solver,
            guards,
            rules,
            num_cells: encoding.num_cells(),
        }
    }

    pub fn guards(&self) -> &[Lit] {
        &self.guards
    }

    /// The rule behind a guard literal, or `None` for a cell literal.
    pub fn rule_of(&self, lit: Lit) -> Option<Rule> {
        let index = lit.var().index().checked_sub(self.num_cells)?;
        Some(self.rules[index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbours_are_listed_once_and_stay_on_the_grid() {
        let at = |row, col| Cell { row, col };
        assert_eq!(
            later_neighbours(at(0, 0), 2),
            vec![at(0, 1), at(1, 0), at(1, 1)]
        );
        assert_eq!(later_neighbours(at(0, 1), 2), vec![at(1, 0), at(1, 1)]);
        assert_eq!(later_neighbours(at(1, 1), 2), Vec::<Cell>::new());
    }
}
