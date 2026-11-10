//! "At most k of these" and "at least k of these" as plain clauses, with no helper variables, so
//! every literal in every clause still means something a reader can see.
//!
//! The clause count grows combinatorially with `k`, which is fine for the small `k` of puzzles.

use crate::data::lit::Lit;

/// Every group of `k + 1` literals must have a false one.
pub fn at_most(lits: &[Lit], k: usize) -> Vec<Vec<Lit>> {
    if k >= lits.len() {
        return Vec::new();
    }
    let negated: Vec<Lit> = lits.iter().map(|&lit| !lit).collect();
    subsets(&negated, k + 1)
}

/// Every group of `n - k + 1` literals must have a true one.
pub fn at_least(lits: &[Lit], k: usize) -> Vec<Vec<Lit>> {
    if k == 0 {
        return Vec::new();
    }
    if k > lits.len() {
        return vec![Vec::new()];
    }
    subsets(lits, lits.len() - k + 1)
}

pub fn exactly(lits: &[Lit], k: usize) -> Vec<Vec<Lit>> {
    let mut clauses = at_most(lits, k);
    clauses.extend(at_least(lits, k));
    clauses
}

/// Every subset of `items` with `size` elements. `size` is at least 1.
fn subsets(items: &[Lit], size: usize) -> Vec<Vec<Lit>> {
    let mut found = Vec::new();
    let mut chosen = Vec::with_capacity(size);
    extend_subset(items, size, &mut chosen, &mut found);
    found
}

fn extend_subset(items: &[Lit], size: usize, chosen: &mut Vec<Lit>, found: &mut Vec<Vec<Lit>>) {
    if chosen.len() == size {
        found.push(chosen.clone());
        return;
    }
    let still_needed = size - chosen.len();
    for first in 0..items.len().saturating_sub(still_needed - 1) {
        chosen.push(items[first]);
        extend_subset(&items[first + 1..], size, chosen, found);
        chosen.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cnf::Cnf;
    use crate::data::lit::Var;

    fn accepts(clauses: Vec<Vec<Lit>>, num_vars: u32, bits: u32) -> bool {
        let model: Vec<bool> = (0..num_vars).map(|var| bits >> var & 1 == 1).collect();
        Cnf { num_vars, clauses }.is_satisfied_by(&model)
    }

    fn check(encode: fn(&[Lit], usize) -> Vec<Vec<Lit>>, allowed: fn(u32, u32) -> bool) {
        for n in 0..=7u32 {
            let lits: Vec<Lit> = (0..n).map(|index| Var::new(index).positive()).collect();
            for k in 0..=n + 1 {
                for bits in 0u32..1 << n {
                    assert_eq!(
                        accepts(encode(&lits, k as usize), n, bits),
                        allowed(bits.count_ones(), k),
                        "n={n} k={k} bits={bits:b}"
                    );
                }
            }
        }
    }

    #[test]
    fn at_most_accepts_exactly_the_right_counts() {
        check(at_most, |count, k| count <= k);
    }

    #[test]
    fn at_least_accepts_exactly_the_right_counts() {
        check(at_least, |count, k| count >= k);
    }

    #[test]
    fn exactly_accepts_exactly_the_right_counts() {
        check(exactly, |count, k| count == k);
    }

    #[test]
    fn two_stars_in_ten_cells_needs_130_clauses() {
        let lits: Vec<Lit> = (0..10).map(|index| Var::new(index).positive()).collect();
        assert_eq!(exactly(&lits, 2).len(), 120 + 10);
    }
}
