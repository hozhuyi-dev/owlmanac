//! Chooses what to branch on: the variable with the highest VSIDS score, set to its last value.
//!
//! The scores sit in an indexed binary heap, so raising a score and taking the best are both
//! `O(log n)`. With VSIDS off no score ever changes, and ties go to the lowest variable, which is
//! a fixed order.

use super::lit::{Lit, Var};
use super::maps::VarMap;

const DECAY: f64 = 0.95;
const RESCALE_ABOVE: f64 = 1e100;

pub struct VarOrder {
    vsids: bool,
    phase_saving: bool,
    activity: VarMap<f64>,
    /// How much a bump adds. Growing it is the same as decaying every score, without touching them.
    increment: f64,
    heap: Vec<Var>,
    /// Where each variable is in `heap`, or `None` if it's been taken out.
    position: VarMap<Option<u32>>,
    saved_phase: VarMap<bool>,
}

impl VarOrder {
    pub fn new(vsids: bool, phase_saving: bool) -> Self {
        VarOrder {
            vsids,
            phase_saving,
            activity: VarMap::new(),
            increment: 1.0,
            heap: Vec::new(),
            position: VarMap::new(),
            saved_phase: VarMap::new(),
        }
    }

    pub fn add_var(&mut self, var: Var) {
        self.activity.grow_to_fit(var, 0.0);
        self.position.grow_to_fit(var, None);
        self.saved_phase.grow_to_fit(var, false);
        self.insert(var);
    }

    /// Takes out the best variable. It may already be assigned, so the caller checks.
    pub fn pop(&mut self) -> Option<Var> {
        let best = *self.heap.first()?;
        let last = self.heap.pop().expect("the heap has a first element");
        self.position[best] = None;
        if best != last {
            self.heap[0] = last;
            self.position[last] = Some(0);
            self.sift_down(0);
        }
        Some(best)
    }

    /// Sets the value a variable is first tried with. Phase saving updates it as the search goes.
    pub fn suggest(&mut self, lit: Lit) {
        self.saved_phase[lit.var()] = lit.is_positive();
    }

    pub fn preferred_literal(&self, var: Var) -> Lit {
        if self.saved_phase[var] {
            var.positive()
        } else {
            var.negative()
        }
    }

    pub fn on_unassigned(&mut self, lit: Lit) {
        if self.phase_saving {
            self.saved_phase[lit.var()] = lit.is_positive();
        }
        self.insert(lit.var());
    }

    pub fn bump(&mut self, var: Var) {
        if !self.vsids {
            return;
        }
        self.activity[var] += self.increment;
        if self.activity[var] > RESCALE_ABOVE {
            self.rescale();
        }
        if let Some(position) = self.position[var] {
            self.sift_up(position as usize);
        }
    }

    pub fn decay(&mut self) {
        self.increment /= DECAY;
    }

    fn rescale(&mut self) {
        for activity in self.activity.values_mut() {
            *activity /= RESCALE_ABOVE;
        }
        self.increment /= RESCALE_ABOVE;
    }

    fn insert(&mut self, var: Var) {
        if self.position[var].is_none() {
            self.heap.push(var);
            self.position[var] = Some(self.heap.len() as u32 - 1);
            self.sift_up(self.heap.len() - 1);
        }
    }

    fn ranks_before(&self, a: Var, b: Var) -> bool {
        self.activity[a] > self.activity[b] || (self.activity[a] == self.activity[b] && a < b)
    }

    fn swap(&mut self, i: usize, j: usize) {
        self.heap.swap(i, j);
        self.position[self.heap[i]] = Some(i as u32);
        self.position[self.heap[j]] = Some(j as u32);
    }

    fn sift_up(&mut self, mut child: usize) {
        while child > 0 {
            let parent = (child - 1) / 2;
            if !self.ranks_before(self.heap[child], self.heap[parent]) {
                break;
            }
            self.swap(child, parent);
            child = parent;
        }
    }

    fn sift_down(&mut self, mut parent: usize) {
        loop {
            let (left, right) = (2 * parent + 1, 2 * parent + 2);
            let mut best = parent;
            for child in [left, right] {
                if child < self.heap.len() && self.ranks_before(self.heap[child], self.heap[best]) {
                    best = child;
                }
            }
            if best == parent {
                break;
            }
            self.swap(parent, best);
            parent = best;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order_with_vars(vsids: bool, phase_saving: bool, count: u32) -> VarOrder {
        let mut order = VarOrder::new(vsids, phase_saving);
        (0..count).for_each(|index| order.add_var(Var::new(index)));
        order
    }

    #[test]
    fn without_vsids_variables_come_out_in_index_order() {
        let mut order = order_with_vars(false, false, 4);
        order.bump(Var::new(3));

        let popped: Vec<_> = std::iter::from_fn(|| order.pop()).collect();

        assert_eq!(popped, (0..4).map(Var::new).collect::<Vec<_>>());
    }

    #[test]
    fn the_most_bumped_variable_comes_out_first() {
        let mut order = order_with_vars(true, false, 4);
        order.bump(Var::new(2));
        order.bump(Var::new(3));
        order.bump(Var::new(3));

        assert_eq!(order.pop(), Some(Var::new(3)));
        assert_eq!(order.pop(), Some(Var::new(2)));
    }

    #[test]
    fn recent_bumps_outweigh_old_ones() {
        let mut order = order_with_vars(true, false, 2);
        order.bump(Var::new(0));
        order.decay();
        order.bump(Var::new(1));

        assert_eq!(order.pop(), Some(Var::new(1)));
    }

    #[test]
    fn rescaling_keeps_the_ranking() {
        let mut order = order_with_vars(true, false, 2);
        order.increment = RESCALE_ABOVE;
        order.bump(Var::new(1));
        order.bump(Var::new(1));

        assert!(order.increment < RESCALE_ABOVE);
        assert_eq!(order.pop(), Some(Var::new(1)));
    }

    #[test]
    fn a_variable_returns_with_its_last_value_when_saving_phases() {
        let mut order = order_with_vars(false, true, 1);
        let var = Var::new(0);
        assert_eq!(order.preferred_literal(var), var.negative());
        order.pop();

        order.on_unassigned(var.positive());

        assert_eq!(order.preferred_literal(var), var.positive());
        assert_eq!(order.pop(), Some(var));
    }
}
