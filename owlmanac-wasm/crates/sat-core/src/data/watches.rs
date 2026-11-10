//! For each literal, the clauses watching it. A list is visited when its literal becomes false.

use super::clause_db::{ClauseDb, ClauseRef};
use super::lit::Lit;
use super::maps::LitMap;

/// `blocker` is some other literal of the clause: if it's true the clause is satisfied, and
/// propagation skips reading the clause at all.
#[derive(Clone, Copy, Debug)]
pub struct Watcher {
    pub clause: ClauseRef,
    pub blocker: Lit,
}

pub struct Watches {
    lists: LitMap<Vec<Watcher>>,
}

impl Watches {
    pub fn new() -> Self {
        Watches {
            lists: LitMap::new(),
        }
    }

    pub fn add_var(&mut self, highest: Lit) {
        self.lists.grow_to_fit(highest, Vec::new());
    }

    /// Starts watching a clause on its first two literals.
    pub fn watch_clause(&mut self, clause: ClauseRef, first: Lit, second: Lit) {
        self.lists[first].push(Watcher {
            clause,
            blocker: second,
        });
        self.lists[second].push(Watcher {
            clause,
            blocker: first,
        });
    }

    pub fn push(&mut self, lit: Lit, watcher: Watcher) {
        self.lists[lit].push(watcher);
    }

    /// Lends out a list so it can be edited while other lists are appended to. Give it back with `restore`.
    pub fn take(&mut self, lit: Lit) -> Vec<Watcher> {
        std::mem::take(&mut self.lists[lit])
    }

    pub fn restore(&mut self, lit: Lit, watchers: Vec<Watcher>) {
        debug_assert!(
            self.lists[lit].is_empty(),
            "a clause moved its watch to a false literal"
        );
        self.lists[lit] = watchers;
    }

    pub fn forget_deleted(&mut self, clauses: &ClauseDb) {
        for list in self.lists.values_mut() {
            list.retain(|watcher| !clauses.is_deleted(watcher.clause));
        }
    }
}
