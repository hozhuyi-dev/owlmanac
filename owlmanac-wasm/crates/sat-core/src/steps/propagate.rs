//! Unit propagation: works out everything the current assignment forces.
//!
//! Each clause watches two literals, and only needs a look when one of them becomes false. It then
//! either finds another literal to watch, or it has one option left, which is forced, or none,
//! which is a conflict. Backjumping needs no work here: unassigning can't break a watch.

use crate::data::clause_db::ClauseRef;
use crate::data::lit::Lit;
use crate::data::watches::Watcher;
use crate::observer::Observer;
use crate::solver::Solver;

enum Visit {
    /// The clause keeps watching this literal.
    Stay,
    /// The clause moved its watch to another literal.
    Moved,
    /// Every literal of the clause is false.
    Conflict,
}

impl<O: Observer> Solver<O> {
    /// Empties the propagation queue. Returns the clause that failed, if one did.
    pub(crate) fn propagate(&mut self) -> Option<ClauseRef> {
        while let Some(lit) = self.trail.next_to_propagate() {
            self.stats.propagations += 1;
            if let Some(conflict) = self.visit_watchers_of(!lit) {
                return Some(conflict);
            }
        }
        None
    }

    fn visit_watchers_of(&mut self, false_lit: Lit) -> Option<ClauseRef> {
        let mut watchers = self.watches.take(false_lit);
        let mut conflict = None;
        watchers.retain_mut(|watcher| {
            if conflict.is_some() {
                return true;
            }
            match self.visit(watcher, false_lit) {
                Visit::Stay => true,
                Visit::Moved => false,
                Visit::Conflict => {
                    conflict = Some(watcher.clause);
                    true
                }
            }
        });
        self.watches.restore(false_lit, watchers);
        conflict
    }

    fn visit(&mut self, watcher: &mut Watcher, false_lit: Lit) -> Visit {
        if self.trail.is_true(watcher.blocker) {
            return Visit::Stay;
        }

        let lits = self.clauses.lits_mut(watcher.clause);
        if lits[0] == false_lit {
            lits.swap(0, 1);
        }
        let other_watch = lits[0];
        watcher.blocker = other_watch;
        if self.trail.is_true(other_watch) {
            return Visit::Stay;
        }

        if let Some(offset) = lits[2..].iter().position(|&lit| !self.trail.is_false(lit)) {
            lits.swap(1, offset + 2);
            let new_watch = lits[1];
            self.watches.push(
                new_watch,
                Watcher {
                    clause: watcher.clause,
                    blocker: other_watch,
                },
            );
            return Visit::Moved;
        }

        if self.trail.is_false(other_watch) {
            return Visit::Conflict;
        }
        self.trail.assign(other_watch, Some(watcher.clause));
        self.observer
            .on_propagate(other_watch, self.clauses.lits(watcher.clause));
        Visit::Stay
    }
}
