//! One puzzle, one rung of the ladder: brute force first, then each step of `sat_core::ladder`.
//! Every run advances in slices, so the browser can show progress and stop it.

use crate::brute_force::{BruteForce, Progress};
use crate::encode::{Encoding, decode, encode};
use crate::grid::Grid;
use sat_core::{Budget, Cnf, Solver, Stats, Status, ladder};
use serde::Serialize;

const BRUTE_FORCE_NAME: &str = "Brute force";
const GUESSES_PER_SLICE: u64 = 200_000;
const CONFLICTS_PER_SLICE: u64 = 1_000;

/// How much work a run has done. A dead end is a brute-force cell out of digits or a SAT conflict.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub guesses: u64,
    pub dead_ends: u64,
    pub propagations: u64,
}

impl From<Stats> for Report {
    fn from(stats: Stats) -> Self {
        Report {
            guesses: stats.decisions,
            dead_ends: stats.conflicts,
            propagations: stats.propagations,
        }
    }
}

pub fn rung_names() -> Vec<&'static str> {
    std::iter::once(BRUTE_FORCE_NAME)
        .chain(ladder().map(|(name, _)| name))
        .collect()
}

enum Engine {
    BruteForce(BruteForce),
    Sat { solver: Box<Solver>, cnf: Cnf },
}

pub struct LadderRun {
    puzzle: Grid,
    engine: Engine,
    finished: Option<Progress>,
    solution: Option<Grid>,
}

impl LadderRun {
    /// Starts rung `rung`, counting from brute force. Returns `None` if there's no such rung.
    pub fn new(puzzle: &Grid, rung: usize, encoding: Encoding) -> Option<Self> {
        let engine = match rung.checked_sub(1) {
            None => Engine::BruteForce(BruteForce::new(puzzle)),
            Some(index) => {
                let (_, config) = *ladder().get(index)?;
                let cnf = encode(puzzle, encoding);
                Engine::Sat {
                    solver: Box::new(Solver::from_cnf(&cnf, config)),
                    cnf,
                }
            }
        };
        Some(LadderRun {
            puzzle: puzzle.clone(),
            engine,
            finished: None,
            solution: None,
        })
    }

    /// Does one slice of work.
    pub fn advance(&mut self) -> Progress {
        if let Some(finished) = self.finished {
            return finished;
        }
        let progress = match &mut self.engine {
            Engine::BruteForce(search) => {
                let progress = search.advance(GUESSES_PER_SLICE);
                if progress == Progress::Solved {
                    self.solution = Some(search.grid().clone());
                }
                progress
            }
            Engine::Sat { solver, cnf } => {
                match solver.resume(Budget::conflicts(CONFLICTS_PER_SLICE)) {
                    Status::Unknown => Progress::Running,
                    Status::Unsat => Progress::NoSolution,
                    Status::Sat => {
                        debug_assert!(cnf.is_satisfied_by(solver.model()));
                        self.solution = Some(decode(&self.puzzle, solver.model()));
                        Progress::Solved
                    }
                }
            }
        };
        if progress != Progress::Running {
            self.finished = Some(progress);
        }
        progress
    }

    pub fn report(&self) -> Report {
        match &self.engine {
            Engine::BruteForce(search) => Report {
                guesses: search.guesses(),
                dead_ends: search.dead_ends(),
                propagations: 0,
            },
            Engine::Sat { solver, .. } => solver.stats().into(),
        }
    }

    /// The grid as the search has it now: brute force's digits so far, or SAT's once solved.
    pub fn current_grid(&self) -> Option<&Grid> {
        match &self.engine {
            Engine::BruteForce(search) => Some(search.grid()),
            Engine::Sat { .. } => self.solution.as_ref(),
        }
    }

    pub fn solution(&self) -> Option<&Grid> {
        self.solution.as_ref()
    }
}
