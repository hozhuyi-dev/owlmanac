//! How hard is a puzzle to reason through? Measured as the deepest "what if" it needs, never by
//! how long a solver took, which depends as much on the solver's habits as on the puzzle.
//!
//! Depth 0 is forced moves alone. Depth 1 also allows a trial: "if I put a star here, the forced
//! moves run into a contradiction, so it can't go here". Depth 2 allows a trial within a trial.

use crate::board::{Board, Placed};
use crate::session::Session;
use sat_core::{Lit, Solver, Var};
use serde::Serialize;

/// The deepest trial the grader will try. Past it, a puzzle needs search.
pub const MAX_DEPTH: u32 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Grade {
    /// The deepest trial needed, or `None` if some cell needs more than `MAX_DEPTH`, or the puzzle
    /// has more than one solution.
    pub depth: Option<u32>,
    /// How many cells needed a trial that deep. More means a longer hard stretch. 0 at depth 0.
    pub deepest_steps: u32,
}

/// Works out what follows from some marks, with a solver that only ever propagates. Having never
/// searched, it has no learnt clauses, so what it finds is only what the rules force.
struct Prober {
    solver: Solver,
    num_cells: usize,
    /// The depth of the outermost call, whose successful trials are counted.
    top_depth: u32,
    deepest_steps: u32,
}

impl Prober {
    /// Everything the forced moves give from `assumed`, or `None` if they reach a contradiction.
    fn propagate(&mut self, assumed: &[Lit]) -> Option<Vec<Lit>> {
        let deductions = self.solver.propagate_assuming(assumed);
        if deductions.contradiction.is_some() {
            return None;
        }
        Some(deductions.steps.into_iter().map(|step| step.lit).collect())
    }

    /// Everything that follows from `assumed` using trials up to `depth` deep, or `None` if that
    /// reaches a contradiction. Whenever a trial fails, its opposite is known, and the search for
    /// failing trials starts over, since one new fact can make another trial fail.
    fn closure(&mut self, assumed: &[Lit], depth: u32) -> Option<Vec<Lit>> {
        let mut known = self.propagate(assumed)?;
        if depth == 0 {
            return Some(known);
        }
        'progress: loop {
            let mut decided = vec![false; self.num_cells];
            known
                .iter()
                .for_each(|lit| decided[lit.var().index()] = true);

            for index in (0..self.num_cells).filter(|&index| !decided[index]) {
                let var = Var::new(index as u32);
                for trial in [var.positive(), var.negative()] {
                    let mut with_trial = known.clone();
                    with_trial.push(trial);
                    if self.closure(&with_trial, depth - 1).is_none() {
                        if depth == self.top_depth {
                            self.deepest_steps += 1;
                        }
                        let mut with_opposite = known.clone();
                        with_opposite.push(!trial);
                        known = self.propagate(&with_opposite)?;
                        continue 'progress;
                    }
                }
            }
            return Some(known);
        }
    }
}

impl Session {
    /// How deep the reasoning has to go to finish this board, or `None` if it can't be finished.
    /// Meant for a puzzle with exactly one solution: with more, no amount of reasoning settles
    /// every cell.
    pub fn grade(&self, board: &Board) -> Option<Grade> {
        let mut prober = Prober {
            solver: self.encoding.plain_solver(),
            num_cells: self.encoding.num_cells(),
            top_depth: 0,
            deepest_steps: 0,
        };
        let marks = self.literals(board);
        for depth in 0..=MAX_DEPTH {
            prober.top_depth = depth;
            prober.deepest_steps = 0;
            let known = prober.closure(&marks, depth)?;
            if known.len() == prober.num_cells {
                return Some(Grade {
                    depth: Some(depth),
                    deepest_steps: prober.deepest_steps,
                });
            }
        }
        Some(Grade {
            depth: None,
            deepest_steps: 0,
        })
    }

    /// The easiest cell to work out next, and how deep the reasoning goes: forced moves alone
    /// first, then a single "what if", then a "what if" inside one. This is one step of `grade`.
    /// `None` if the board is finished, can't be finished, or needs deeper reasoning than that.
    pub fn hint(&self, board: &Board) -> Option<Hint> {
        let mut prober = Prober {
            solver: self.encoding.plain_solver(),
            num_cells: self.encoding.num_cells(),
            top_depth: 0,
            deepest_steps: 0,
        };
        let marks = self.literals(board);
        let known = prober.propagate(&marks)?;

        let mut marked = vec![false; prober.num_cells];
        marks
            .iter()
            .for_each(|lit| marked[lit.var().index()] = true);
        if let Some(&forced) = known.iter().find(|lit| !marked[lit.var().index()]) {
            return Some(Hint {
                placed: self.encoding.placed(forced),
                depth: 0,
            });
        }

        let mut decided = vec![false; prober.num_cells];
        known
            .iter()
            .for_each(|lit| decided[lit.var().index()] = true);
        for depth in 1..=MAX_DEPTH {
            for index in (0..prober.num_cells).filter(|&index| !decided[index]) {
                let var = Var::new(index as u32);
                for trial in [var.positive(), var.negative()] {
                    let mut with_trial = known.clone();
                    with_trial.push(trial);
                    if prober.closure(&with_trial, depth - 1).is_none() {
                        return Some(Hint {
                            placed: self.encoding.placed(!trial),
                            depth,
                        });
                    }
                }
            }
        }
        None
    }
}

/// A cell the player can work out next, and how many "what if"s deep the reasoning goes.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
pub struct Hint {
    pub placed: Placed,
    pub depth: u32,
}
