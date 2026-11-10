//! Conflict analysis: turns a failed clause into a new clause that rules out the cause.
//!
//! Starting from the conflict, it resolves against the reason of each current-level literal, latest
//! first, until only one current-level literal is left: the first unique implication point (UIP).
//! That literal's negation will be forced as soon as the search jumps back.

use crate::data::clause_db::ClauseRef;
use crate::data::lit::{Lit, Var};
use crate::data::maps::VarMap;
use crate::observer::Observer;
use crate::solver::{Solver, Status};

pub(crate) struct Learnt {
    /// The asserting literal is first, and a literal from `backjump_level` second.
    pub lits: Vec<Lit>,
    pub backjump_level: u32,
    pub lbd: u32,
}

/// Which variables the current analysis has met.
pub(crate) struct Marks {
    seen: VarMap<bool>,
    touched: Vec<Var>,
}

impl Marks {
    pub fn new() -> Self {
        Marks {
            seen: VarMap::new(),
            touched: Vec::new(),
        }
    }

    pub fn add_var(&mut self, var: Var) {
        self.seen.grow_to_fit(var, false);
    }

    fn mark(&mut self, var: Var) {
        self.seen[var] = true;
        self.touched.push(var);
    }

    fn clear(&mut self) {
        for var in self.touched.drain(..) {
            self.seen[var] = false;
        }
    }
}

impl<O: Observer> Solver<O> {
    pub(crate) fn analyze(&mut self, conflict: ClauseRef) -> Learnt {
        self.stats.conflicts += 1;
        self.observer.on_conflict(self.clauses.lits(conflict));
        let mut lits = self.resolve_to_first_uip(conflict);
        self.minimize(&mut lits);
        let backjump_level = self.move_backjump_literal_to_second(&mut lits);
        let lbd = self.count_levels(&lits);
        self.marks.clear();
        self.stats.learnt_literals += lits.len() as u64;
        Learnt {
            lits,
            backjump_level,
            lbd,
        }
    }

    fn resolve_to_first_uip(&mut self, conflict: ClauseRef) -> Vec<Lit> {
        let mut earlier_levels = Vec::new();
        let mut open = self.mark_clause(conflict, None, &mut earlier_levels);
        let mut position = self.trail.len();
        loop {
            let pivot = self.latest_marked_literal(&mut position);
            self.marks.seen[pivot.var()] = false;
            open -= 1;
            if open == 0 {
                let mut lits = vec![!pivot];
                lits.append(&mut earlier_levels);
                return lits;
            }
            let reason = self.trail.reason_of(pivot.var()).expect(
                "only the level's decision has no reason, and it is always the last open literal",
            );
            self.observer.on_resolve(self.clauses.lits(reason));
            open += self.mark_clause(reason, Some(pivot), &mut earlier_levels);
        }
    }

    /// Marks the clause's unseen literals, collecting those from earlier levels into the clause
    /// being learnt. Root-level literals are always false, so they're left out. Returns how many
    /// current-level literals still have to be resolved away.
    fn mark_clause(
        &mut self,
        clause: ClauseRef,
        implied: Option<Lit>,
        earlier_levels: &mut Vec<Lit>,
    ) -> usize {
        let current_level = self.trail.decision_level();
        let mut newly_open = 0;
        for &lit in self.clauses.lits(clause) {
            let var = lit.var();
            let level = self.trail.level_of(var);
            if Some(lit) == implied || self.marks.seen[var] || level == 0 {
                continue;
            }
            self.marks.mark(var);
            self.order.bump(var);
            if level == current_level {
                newly_open += 1;
            } else {
                earlier_levels.push(lit);
            }
        }
        newly_open
    }

    fn latest_marked_literal(&self, position: &mut usize) -> Lit {
        loop {
            *position -= 1;
            let lit = self.trail.literal_at(*position);
            if self.marks.seen[lit.var()] {
                return lit;
            }
        }
    }

    /// Drops every literal already implied by the rest of the clause. The asserting literal stays.
    fn minimize(&mut self, lits: &mut Vec<Lit>) {
        for lit in lits.split_off(1) {
            match self.reason_if_redundant(lit) {
                Some(reason) => self.observer.on_resolve(self.clauses.lits(reason)),
                None => lits.push(lit),
            }
        }
    }

    fn reason_if_redundant(&self, lit: Lit) -> Option<ClauseRef> {
        let reason = self.trail.reason_of(lit.var())?;
        let redundant = self.clauses.lits(reason).iter().all(|other| {
            other.var() == lit.var()
                || self.marks.seen[other.var()]
                || self.trail.level_of(other.var()) == 0
        });
        redundant.then_some(reason)
    }

    /// Finds where to jump back to: the highest level among the non-asserting literals. Its literal
    /// goes second, so the clause watches the asserting literal and the last one to be undone.
    fn move_backjump_literal_to_second(&self, lits: &mut [Lit]) -> u32 {
        if lits.len() == 1 {
            return 0;
        }
        let highest = (1..lits.len())
            .max_by_key(|&index| self.trail.level_of(lits[index].var()))
            .expect("the range is not empty");
        lits.swap(1, highest);
        self.trail.level_of(lits[1].var())
    }

    /// Records which assumptions forced `assumption` false: the search can't get past them, so
    /// the answer is unsatisfiable under these assumptions, though maybe not without them.
    pub(crate) fn fail_assumption(&mut self, assumption: Lit) -> Status {
        self.failed = vec![assumption];
        if self.trail.level_of(assumption.var()) > 0 {
            self.marks.mark(assumption.var());
        }
        for position in (0..self.trail.len()).rev() {
            let lit = self.trail.literal_at(position);
            if !self.marks.seen[lit.var()] {
                continue;
            }
            match self.trail.reason_of(lit.var()) {
                None => self.failed.push(lit),
                Some(reason) => {
                    for other in self.clauses.lits(reason) {
                        if other.var() != lit.var() && self.trail.level_of(other.var()) > 0 {
                            self.marks.mark(other.var());
                        }
                    }
                }
            }
        }
        self.marks.clear();
        Status::Unsat
    }

    fn count_levels(&self, lits: &[Lit]) -> u32 {
        let mut levels: Vec<u32> = lits
            .iter()
            .map(|lit| self.trail.level_of(lit.var()))
            .collect();
        levels.sort_unstable();
        levels.dedup();
        levels.len() as u32
    }
}
