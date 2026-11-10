//! The solver's state and public interface. The algorithm itself starts in `search.rs`.

use crate::cnf::Cnf;
use crate::data::clause_db::ClauseDb;
use crate::data::lit::{Lit, Var};
use crate::data::trail::Trail;
use crate::data::var_order::VarOrder;
use crate::data::watches::Watches;
use crate::observer::{NoObserver, Observer};
use crate::steps::analyze::Marks;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Sat,
    Unsat,
    /// The budget ran out. Call `solve` again to carry on.
    Unknown,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Strategy {
    /// Learns a clause from each conflict and jumps back to where it applies.
    Cdcl,
    /// Learns nothing: undoes the latest decision and tries its other value.
    Dpll,
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub strategy: Strategy,
    /// Branch on recently conflicting variables, rather than in index order.
    pub vsids: bool,
    /// Give a variable its last value when branching on it again, rather than false.
    pub phase_saving: bool,
    /// Restart after this many conflicts times the Luby sequence. `None` never restarts.
    pub restart_base: Option<u64>,
    /// Conflicts between deleting learnt clauses, growing each time. Deletion happens straight
    /// after a restart, so it needs `restart_base`. `None` keeps every clause.
    pub reduce_interval: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            strategy: Strategy::Cdcl,
            vsids: true,
            phase_saving: true,
            restart_base: Some(100),
            reduce_interval: Some(2000),
        }
    }
}

impl Config {
    /// Backtracking with unit propagation, in a fixed variable order.
    pub fn dpll() -> Self {
        Config {
            strategy: Strategy::Dpll,
            ..Config::basic_cdcl()
        }
    }

    /// Clause learning alone: a fixed variable order, no restarts, nothing deleted.
    pub fn basic_cdcl() -> Self {
        Config {
            strategy: Strategy::Cdcl,
            vsids: false,
            phase_saving: false,
            restart_base: None,
            reduce_interval: None,
        }
    }
}

/// How long one `solve` call may run.
#[derive(Clone, Copy, Debug)]
pub struct Budget {
    conflicts_left: Option<u64>,
}

impl Budget {
    pub fn unlimited() -> Self {
        Budget {
            conflicts_left: None,
        }
    }

    pub fn conflicts(count: u64) -> Self {
        Budget {
            conflicts_left: Some(count),
        }
    }

    pub(crate) fn spend_conflict(&mut self) {
        if let Some(left) = &mut self.conflicts_left {
            *left = left.saturating_sub(1);
        }
    }

    pub(crate) fn exhausted(&self) -> bool {
        self.conflicts_left == Some(0)
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Stats {
    pub decisions: u64,
    pub propagations: u64,
    pub conflicts: u64,
    pub learnt_literals: u64,
    pub restarts: u64,
    pub reductions: u64,
}

pub struct Solver<O: Observer = NoObserver> {
    pub(crate) config: Config,
    /// Every clause, original and learnt.
    pub(crate) clauses: ClauseDb,
    /// Which clauses to visit when a literal becomes false.
    pub(crate) watches: Watches,
    /// The assignments so far, in order, each with its level and reason.
    pub(crate) trail: Trail,
    /// What to branch on next.
    pub(crate) order: VarOrder,
    /// Scratch space for conflict analysis.
    pub(crate) marks: Marks,
    /// What the current query assumes, placed first, one per decision level.
    pub(crate) assumptions: Vec<Lit>,
    /// The assumptions behind the last `Unsat`. Empty if it holds without any of them.
    pub(crate) failed: Vec<Lit>,
    /// For DPLL, whether each open decision has already been flipped.
    pub(crate) flipped: Vec<bool>,
    pub(crate) next_restart: Option<u64>,
    pub(crate) next_reduction: Option<u64>,
    pub(crate) proven_unsat: bool,
    pub(crate) stats: Stats,
    pub(crate) observer: O,
    num_vars: u32,
    model: Vec<bool>,
}

impl Solver<NoObserver> {
    pub fn new(config: Config) -> Self {
        Solver::with_observer(config, NoObserver)
    }

    pub fn from_cnf(cnf: &Cnf, config: Config) -> Self {
        let mut solver = Solver::new(config);
        solver.add_cnf(cnf);
        solver
    }
}

impl<O: Observer> Solver<O> {
    pub fn with_observer(config: Config, observer: O) -> Self {
        Solver {
            config,
            clauses: ClauseDb::default(),
            watches: Watches::new(),
            trail: Trail::new(),
            order: VarOrder::new(config.vsids, config.phase_saving),
            marks: Marks::new(),
            assumptions: Vec::new(),
            failed: Vec::new(),
            flipped: Vec::new(),
            next_restart: config.restart_base,
            next_reduction: config.reduce_interval,
            proven_unsat: false,
            stats: Stats::default(),
            observer,
            num_vars: 0,
            model: Vec::new(),
        }
    }

    /// Loads a formula into a solver that has no variables yet.
    pub fn add_cnf(&mut self, cnf: &Cnf) {
        assert_eq!(self.num_vars, 0, "a formula numbers its variables from 0");
        for _ in 0..cnf.num_vars {
            self.new_var();
        }
        for clause in &cnf.clauses {
            self.add_clause(clause);
        }
    }

    pub fn new_var(&mut self) -> Var {
        let var = Var::new(self.num_vars);
        self.num_vars += 1;
        self.watches.add_var(var.negative());
        self.trail.add_var(var);
        self.order.add_var(var);
        self.marks.add_var(var);
        var
    }

    /// Adds a clause, abandoning any search in progress. Literals already decided at the root are
    /// dropped, so watched literals always start out unassigned.
    pub fn add_clause(&mut self, lits: &[Lit]) {
        self.backjump(0);
        let mut clause = lits.to_vec();
        clause.sort_unstable();
        clause.dedup();

        // Sorting puts a literal right beside its negation.
        let tautology = clause.windows(2).any(|pair| pair[0] == !pair[1]);
        if tautology || clause.iter().any(|&lit| self.trail.is_true(lit)) {
            return;
        }
        clause.retain(|&lit| !self.trail.is_false(lit));

        match clause.as_slice() {
            [] => {
                self.mark_unsat();
            }
            [unit] => self.trail.assign(*unit, None),
            [first, second, ..] => {
                let added = self.clauses.add_original(&clause);
                self.watches.watch_clause(added, *first, *second);
            }
        }
    }

    pub fn solve(&mut self, budget: Budget) -> Status {
        self.solve_assuming(&[], budget)
    }

    /// Starts a new query, in which `assumptions` hold for this call only. What is learnt stays
    /// valid for later queries, so asking many related questions is cheap.
    pub fn solve_assuming(&mut self, assumptions: &[Lit], budget: Budget) -> Status {
        assert!(
            assumptions.is_empty() || self.config.strategy == Strategy::Cdcl,
            "DPLL takes no assumptions"
        );
        self.backjump(0);
        self.flipped.clear();
        self.assumptions = assumptions.to_vec();
        self.failed.clear();
        self.resume(budget)
    }

    /// Carries on with the current query after `Unknown`.
    pub fn resume(&mut self, mut budget: Budget) -> Status {
        if self.proven_unsat {
            return Status::Unsat;
        }
        let status = match self.config.strategy {
            Strategy::Cdcl => self.search(&mut budget),
            Strategy::Dpll => self.search_dpll(&mut budget),
        };
        if status == Status::Sat {
            self.save_model();
        }
        if status != Status::Unknown {
            self.backjump(0);
            self.flipped.clear();
        }
        status
    }

    /// The satisfying assignment from the last `Sat`, by variable index.
    pub fn model(&self) -> &[bool] {
        &self.model
    }

    /// Which assumptions the last `Unsat` depended on. Empty if the formula has no solution at all.
    pub fn failed_assumptions(&self) -> &[Lit] {
        &self.failed
    }

    pub fn observer(&self) -> &O {
        &self.observer
    }

    /// Suggests a value to try `lit`'s variable with first. Only a hint: it changes how fast an
    /// answer is found, never which answers exist. A good starting guess, such as a nearly
    /// right design, can save a great deal of search.
    pub fn suggest(&mut self, lit: Lit) {
        self.order.suggest(lit);
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }

    pub(crate) fn mark_unsat(&mut self) -> Status {
        self.proven_unsat = true;
        self.observer.on_unsat();
        Status::Unsat
    }

    fn save_model(&mut self) {
        self.model = (0..self.num_vars)
            .map(|index| self.trail.is_true(Var::new(index).positive()))
            .collect();
    }
}
