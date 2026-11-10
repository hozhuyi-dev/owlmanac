//! The solver lab for JavaScript. Each method converts its arguments, calls a library crate, and
//! converts the answer. Every run advances in slices, so a worker can yield between them.

use sat_core::random::random_k_sat;
use sat_core::{Budget, Solver, Status, dimacs, ladder};
use serde::Serialize;
use sudoku::{Encoding, Grid, LadderRun, Progress, Report, rung_names};
use wasm_bindgen::prelude::*;

const RUNNING: &str = "running";
const SOLVED: &str = "solved";
const NO_SOLUTION: &str = "noSolution";

/// The rungs of the ladder for Sudoku, brute force first.
#[wasm_bindgen(js_name = sudokuRungs)]
pub fn sudoku_rungs() -> Vec<String> {
    rung_names().into_iter().map(str::to_string).collect()
}

/// One rung of the ladder, working on one Sudoku.
#[wasm_bindgen]
pub struct SudokuRun {
    run: LadderRun,
}

#[wasm_bindgen]
impl SudokuRun {
    /// `clues` holds each cell's digit, row by row, with 0 for a blank.
    #[wasm_bindgen(constructor)]
    pub fn new(
        box_size: usize,
        clues: &[u8],
        rung: usize,
        extended: bool,
    ) -> Result<SudokuRun, JsError> {
        let puzzle = Grid::new(box_size, clues.to_vec())?;
        let encoding = if extended {
            Encoding::Extended
        } else {
            Encoding::Minimal
        };
        let run = LadderRun::new(&puzzle, rung, encoding)
            .ok_or_else(|| JsError::new(&format!("no rung {rung}")))?;
        Ok(SudokuRun { run })
    }

    /// Does one slice of work and says where it got to.
    pub fn advance(&mut self) -> String {
        match self.run.advance() {
            Progress::Running => RUNNING,
            Progress::Solved => SOLVED,
            Progress::NoSolution => NO_SOLUTION,
        }
        .to_string()
    }

    pub fn report(&self) -> Result<JsValue, JsError> {
        to_js(&self.run.report())
    }

    /// The grid as the search has it now, in the form of `clues`. SAT only has one once solved.
    #[wasm_bindgen(js_name = currentGrid)]
    pub fn current_grid(&self) -> Option<Vec<u8>> {
        self.run.current_grid().map(|grid| grid.cells().to_vec())
    }

    /// The solved grid in the form of `clues`, or `undefined` before there is one.
    pub fn solution(&self) -> Option<Vec<u8>> {
        self.run.solution().map(|grid| grid.cells().to_vec())
    }
}

/// One configuration of the solver, working on one formula.
#[wasm_bindgen]
pub struct FormulaRun {
    solver: Solver,
}

#[wasm_bindgen]
impl FormulaRun {
    /// `rung` counts from DPLL, the first rung of the SAT ladder.
    pub fn random(
        num_vars: u32,
        num_clauses: u32,
        clause_size: u32,
        seed: u64,
        rung: usize,
    ) -> Result<FormulaRun, JsError> {
        let cnf = random_k_sat(num_vars, num_clauses, clause_size, seed);
        Ok(FormulaRun {
            solver: Solver::from_cnf(&cnf, config_for(rung)?),
        })
    }

    #[wasm_bindgen(js_name = fromDimacs)]
    pub fn from_dimacs(text: &str, rung: usize) -> Result<FormulaRun, JsError> {
        let cnf = dimacs::parse(text)?;
        Ok(FormulaRun {
            solver: Solver::from_cnf(&cnf, config_for(rung)?),
        })
    }

    /// Searches for up to `conflicts` more conflicts and says where it got to.
    pub fn advance(&mut self, conflicts: u64) -> String {
        match self.solver.resume(Budget::conflicts(conflicts)) {
            Status::Unknown => RUNNING,
            Status::Sat => SOLVED,
            Status::Unsat => NO_SOLUTION,
        }
        .to_string()
    }

    pub fn report(&self) -> Result<JsValue, JsError> {
        to_js(&Report::from(self.solver.stats()))
    }

    /// Each variable's value from the last `solved`, as 0 or 1.
    pub fn model(&self) -> Vec<u8> {
        self.solver
            .model()
            .iter()
            .map(|&value| value as u8)
            .collect()
    }
}

fn config_for(rung: usize) -> Result<sat_core::Config, JsError> {
    ladder()
        .get(rung)
        .map(|&(_, config)| config)
        .ok_or_else(|| JsError::new(&format!("no rung {rung}")))
}

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
    serde_wasm_bindgen::to_value(value).map_err(|error| JsError::new(&error.to_string()))
}
