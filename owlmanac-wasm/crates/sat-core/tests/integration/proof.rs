//! Checks `Unsat` proofs with a small independent checker: each learnt clause must follow by unit
//! propagation from the clauses before it, and the proof must reach the empty clause.

use super::support::{eager_cleanup, pigeonhole, satisfiable_by_enumeration, small_cnf};
use proptest::prelude::*;
use sat_core::random::random_k_sat;
use sat_core::{Budget, Cnf, Config, DratProof, Lit, Solver, Status};

fn proof_of(cnf: &Cnf, config: Config) -> (Status, String) {
    let mut solver = Solver::with_observer(config, DratProof::default());
    solver.add_cnf(cnf);
    let status = solver.solve(Budget::unlimited());
    (status, solver.observer().as_str().to_string())
}

/// Sorted, with repeated literals collapsed, as DRAT checkers treat clauses.
fn sorted(clause: &[Lit]) -> Vec<Lit> {
    let mut sorted = clause.to_vec();
    sorted.sort();
    sorted.dedup();
    sorted
}

/// Whether assuming every literal of `clause` false leads, by unit propagation, to a conflict.
fn follows_by_propagation(active: &[Vec<Lit>], clause: &[Lit]) -> bool {
    let mut truths: Vec<Lit> = clause.iter().map(|&lit| !lit).collect();
    loop {
        let mut changed = false;
        for candidate in active {
            if candidate.iter().any(|lit| truths.contains(lit)) {
                continue;
            }
            let open: Vec<Lit> = candidate
                .iter()
                .copied()
                .filter(|lit| !truths.contains(&!*lit))
                .collect();
            match open.as_slice() {
                [] => return true,
                [unit] => {
                    truths.push(*unit);
                    changed = true;
                }
                _ => {}
            }
        }
        if !changed {
            return false;
        }
    }
}

fn check(cnf: &Cnf, proof: &str) -> Result<(), String> {
    let mut active: Vec<Vec<Lit>> = cnf.clauses.iter().map(|clause| sorted(clause)).collect();
    for line in proof.lines() {
        let (deleting, body) = match line.strip_prefix("d ") {
            Some(body) => (true, body),
            None => (false, line),
        };
        let clause: Vec<Lit> = body
            .split_whitespace()
            .map(|token| token.parse::<i32>().expect("a proof holds only numbers"))
            .take_while(|&number| number != 0)
            .map(Lit::from_dimacs)
            .collect();
        let clause = sorted(&clause);

        if deleting {
            let position = active
                .iter()
                .position(|candidate| *candidate == clause)
                .ok_or(format!("deleted a clause that isn't there: {line}"))?;
            active.swap_remove(position);
            continue;
        }
        if !follows_by_propagation(&active, &clause) {
            return Err(format!("doesn't follow by propagation: {line}"));
        }
        if clause.is_empty() {
            return Ok(());
        }
        active.push(clause);
    }
    Err("the proof never reaches the empty clause".to_string())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn every_unsat_answer_has_a_valid_proof(cnf in small_cnf()) {
        prop_assume!(!satisfiable_by_enumeration(&cnf));
        for config in [Config::basic_cdcl(), Config::default(), eager_cleanup()] {
            let (status, proof) = proof_of(&cnf, config);
            prop_assert_eq!(status, Status::Unsat);
            prop_assert_eq!(check(&cnf, &proof), Ok(()));
        }
    }
}

#[test]
fn pigeonhole_proofs_check_out_even_with_deletion() {
    let cnf = pigeonhole(6, 5);
    let (status, proof) = proof_of(&cnf, eager_cleanup());

    assert_eq!(status, Status::Unsat);
    assert!(proof.lines().any(|line| line.starts_with("d ")));
    assert_eq!(check(&cnf, &proof), Ok(()));
}

#[test]
fn proofs_of_random_formulas_check_out() {
    let mut unsat = 0;
    for seed in 0..30 {
        let cnf = random_k_sat(40, 200, 3, seed);
        let (status, proof) = proof_of(&cnf, eager_cleanup());
        if status == Status::Unsat {
            unsat += 1;
            assert_eq!(check(&cnf, &proof), Ok(()), "seed {seed}");
        }
    }
    assert!(unsat > 0);
}

#[test]
fn a_checker_rejects_a_clause_that_does_not_follow() {
    let cnf = pigeonhole(2, 2);
    assert!(check(&cnf, "1 0\n0\n").is_err());
}
