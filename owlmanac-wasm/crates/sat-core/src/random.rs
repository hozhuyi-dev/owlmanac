//! Reproducible random formulas: the same seed gives the same formula, in Rust and in the browser.

use crate::cnf::Cnf;
use crate::data::lit::{Lit, Var};

/// Generates a uniform random k-SAT formula: each clause has `k` distinct variables and random signs.
pub fn random_k_sat(num_vars: u32, num_clauses: u32, k: u32, seed: u64) -> Cnf {
    assert!(k <= num_vars, "a clause needs {k} distinct variables");
    let mut rng = Rng::new(seed);
    let clauses = (0..num_clauses)
        .map(|_| random_clause(num_vars, k, &mut rng))
        .collect();
    Cnf { num_vars, clauses }
}

fn random_clause(num_vars: u32, k: u32, rng: &mut Rng) -> Vec<Lit> {
    let mut clause: Vec<Lit> = Vec::with_capacity(k as usize);
    while clause.len() < k as usize {
        let var = Var::new(rng.below(num_vars));
        if clause.iter().all(|lit| lit.var() != var) {
            clause.push(if rng.below(2) == 0 {
                var.positive()
            } else {
                var.negative()
            });
        }
    }
    clause
}

/// xorshift64*: small, fast, and good enough to pick clauses.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        // SplitMix64 scrambles the seed, so nearby seeds start far apart. (Just setting the low
        // bit, to avoid xorshift's all-zero state, made seeds 2k and 2k + 1 identical.)
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        Rng(if z == 0 { 1 } else { z })
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn below(&mut self, bound: u32) -> u32 {
        (self.next() >> 32) as u32 % bound
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_gives_the_same_formula() {
        assert_eq!(random_k_sat(20, 80, 3, 7), random_k_sat(20, 80, 3, 7));
        assert_ne!(random_k_sat(20, 80, 3, 7), random_k_sat(20, 80, 3, 8));
    }

    #[test]
    fn neighbouring_seeds_give_different_formulas() {
        let formulas: Vec<Cnf> = (0..64).map(|seed| random_k_sat(20, 80, 3, seed)).collect();
        for (index, formula) in formulas.iter().enumerate() {
            assert!(
                formulas[index + 1..].iter().all(|other| other != formula),
                "seed {index}"
            );
        }
    }

    #[test]
    fn clauses_have_k_distinct_variables() {
        for clause in random_k_sat(5, 200, 3, 1).clauses {
            let mut vars: Vec<_> = clause.iter().map(|lit| lit.var()).collect();
            vars.sort();
            vars.dedup();
            assert_eq!(vars.len(), 3);
        }
    }
}
