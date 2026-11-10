//! The current partial assignment, in the order it was made.
//!
//! The trail is also the propagation queue: literals from `head` onwards have been assigned but
//! their consequences haven't been worked out yet.

use super::clause_db::ClauseRef;
use super::lit::{Lit, Var};
use super::maps::{LitMap, VarMap};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Value {
    Unassigned,
    True,
    False,
}

#[derive(Clone, Copy)]
struct Assignment {
    level: u32,
    reason: Option<ClauseRef>,
}

pub struct Trail {
    assigned: Vec<Lit>,
    /// Where each decision level begins in `assigned`. Level `n` starts at `level_starts[n - 1]`.
    level_starts: Vec<usize>,
    head: usize,
    values: LitMap<Value>,
    assignments: VarMap<Assignment>,
}

impl Trail {
    pub fn new() -> Self {
        Trail {
            assigned: Vec::new(),
            level_starts: Vec::new(),
            head: 0,
            values: LitMap::new(),
            assignments: VarMap::new(),
        }
    }

    pub fn add_var(&mut self, var: Var) {
        self.values.grow_to_fit(var.negative(), Value::Unassigned);
        self.assignments.grow_to_fit(
            var,
            Assignment {
                level: 0,
                reason: None,
            },
        );
    }

    pub fn is_true(&self, lit: Lit) -> bool {
        self.values[lit] == Value::True
    }

    pub fn is_false(&self, lit: Lit) -> bool {
        self.values[lit] == Value::False
    }

    pub fn is_unassigned(&self, var: Var) -> bool {
        self.values[var.positive()] == Value::Unassigned
    }

    pub fn level_of(&self, var: Var) -> u32 {
        self.assignments[var].level
    }

    /// The clause that forced this variable, or `None` for a decision or a fact known at the root.
    pub fn reason_of(&self, var: Var) -> Option<ClauseRef> {
        self.assignments[var].reason
    }

    pub fn len(&self) -> usize {
        self.assigned.len()
    }

    pub fn literal_at(&self, position: usize) -> Lit {
        self.assigned[position]
    }

    pub fn decision_level(&self) -> u32 {
        self.level_starts.len() as u32
    }

    pub fn at_root(&self) -> bool {
        self.level_starts.is_empty()
    }

    pub fn decision_at(&self, level: u32) -> Lit {
        self.assigned[self.level_starts[level as usize - 1]]
    }

    pub fn open_level(&mut self) {
        self.level_starts.push(self.assigned.len());
    }

    pub fn assign(&mut self, lit: Lit, reason: Option<ClauseRef>) {
        debug_assert!(self.is_unassigned(lit.var()));
        self.values[lit] = Value::True;
        self.values[!lit] = Value::False;
        self.assignments[lit.var()] = Assignment {
            level: self.decision_level(),
            reason,
        };
        self.assigned.push(lit);
    }

    pub fn next_to_propagate(&mut self) -> Option<Lit> {
        let lit = *self.assigned.get(self.head)?;
        self.head += 1;
        Some(lit)
    }

    /// Undoes every level above `level`, reporting each literal it unassigns, latest first.
    pub fn backjump_to(&mut self, level: u32, mut on_unassigned: impl FnMut(Lit)) {
        if level >= self.decision_level() {
            return;
        }
        let start = self.level_starts[level as usize];
        for lit in self.assigned.drain(start..).rev() {
            self.values[lit] = Value::Unassigned;
            self.values[!lit] = Value::Unassigned;
            on_unassigned(lit);
        }
        self.level_starts.truncate(level as usize);
        self.head = self.assigned.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trail_with_vars(count: u32) -> Trail {
        let mut trail = Trail::new();
        (0..count).for_each(|index| trail.add_var(Var::new(index)));
        trail
    }

    #[test]
    fn assigning_a_literal_makes_its_negation_false() {
        let mut trail = trail_with_vars(1);
        let lit = Var::new(0).positive();

        trail.assign(lit, None);

        assert!(trail.is_true(lit));
        assert!(trail.is_false(!lit));
    }

    #[test]
    fn the_trail_is_consumed_in_assignment_order() {
        let mut trail = trail_with_vars(2);
        let (a, b) = (Var::new(0).positive(), Var::new(1).negative());
        trail.assign(a, None);
        trail.assign(b, None);

        assert_eq!(trail.next_to_propagate(), Some(a));
        assert_eq!(trail.next_to_propagate(), Some(b));
        assert_eq!(trail.next_to_propagate(), None);
    }

    #[test]
    fn backjumping_undoes_only_the_higher_levels() {
        let mut trail = trail_with_vars(3);
        let lits: Vec<Lit> = (0..3).map(|index| Var::new(index).positive()).collect();
        trail.assign(lits[0], None);
        trail.open_level();
        trail.assign(lits[1], None);
        trail.open_level();
        trail.assign(lits[2], None);

        let mut undone = Vec::new();
        trail.backjump_to(1, |lit| undone.push(lit));

        assert_eq!(undone, vec![lits[2]]);
        assert_eq!(trail.decision_level(), 1);
        assert!(trail.is_true(lits[1]));
        assert!(trail.is_unassigned(lits[2].var()));
    }
}
