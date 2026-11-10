//! Every search configuration must give the right answer, checked against exhaustive enumeration
//! on small formulas and against each other on larger ones.

use super::support::{eager_cleanup, pigeonhole, satisfiable_by_enumeration, small_cnf};
use proptest::prelude::*;
use sat_core::random::random_k_sat;
use sat_core::{Budget, Cnf, Config, Solver, Status, Var};

fn every_config() -> [(&'static str, Config); 6] {
    [
        ("dpll", Config::dpll()),
        ("basic cdcl", Config::basic_cdcl()),
        (
            "vsids",
            Config {
                vsids: true,
                ..Config::basic_cdcl()
            },
        ),
        (
            "phase saving",
            Config {
                vsids: true,
                phase_saving: true,
                ..Config::basic_cdcl()
            },
        ),
        ("default", Config::default()),
        ("eager cleanup", eager_cleanup()),
    ]
}

fn assert_correct(cnf: &Cnf, solver: &Solver, status: Status, expected_sat: bool, name: &str) {
    let expected = if expected_sat {
        Status::Sat
    } else {
        Status::Unsat
    };
    assert_eq!(status, expected, "{name} gave the wrong answer on {cnf:?}");
    if expected_sat {
        assert!(
            cnf.is_satisfied_by(solver.model()),
            "{name} returned a bad model on {cnf:?}"
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]

    #[test]
    fn every_config_matches_enumeration(cnf in small_cnf()) {
        let expected = satisfiable_by_enumeration(&cnf);
        for (name, config) in every_config() {
            let mut solver = Solver::from_cnf(&cnf, config);
            let status = solver.solve(Budget::unlimited());
            assert_correct(&cnf, &solver, status, expected, name);
        }
    }

    #[test]
    fn solving_in_tiny_slices_gives_the_same_answer(cnf in small_cnf()) {
        let expected = satisfiable_by_enumeration(&cnf);
        for (name, config) in every_config() {
            let mut solver = Solver::from_cnf(&cnf, config);
            let mut status = Status::Unknown;
            while status == Status::Unknown {
                status = solver.resume(Budget::conflicts(1));
            }
            assert_correct(&cnf, &solver, status, expected, name);
        }
    }
}

#[test]
fn configs_agree_on_formulas_at_the_hardness_peak() {
    for seed in 0..40 {
        let cnf = random_k_sat(50, 213, 3, seed);
        let mut reference = Solver::from_cnf(&cnf, Config::dpll());
        let expected = reference.solve(Budget::unlimited()) == Status::Sat;

        for (name, config) in every_config() {
            let mut solver = Solver::from_cnf(&cnf, config);
            let status = solver.solve(Budget::unlimited());
            assert_correct(&cnf, &solver, status, expected, name);
        }
    }
}

#[test]
fn clause_deletion_really_runs() {
    let cnf = pigeonhole(7, 6);
    let mut solver = Solver::from_cnf(&cnf, eager_cleanup());

    assert_eq!(solver.solve(Budget::unlimited()), Status::Unsat);

    assert!(solver.stats().restarts > 0);
    assert!(solver.stats().reductions > 0);
}

#[test]
fn more_pigeons_than_holes_is_unsatisfiable() {
    for (name, config) in every_config() {
        let mut solver = Solver::from_cnf(&pigeonhole(6, 5), config);
        assert_eq!(solver.solve(Budget::unlimited()), Status::Unsat, "{name}");
    }
}

#[test]
fn one_pigeon_per_hole_is_satisfiable() {
    let cnf = pigeonhole(5, 5);
    for (name, config) in every_config() {
        let mut solver = Solver::from_cnf(&cnf, config);
        let status = solver.solve(Budget::unlimited());
        assert_correct(&cnf, &solver, status, true, name);
    }
}

#[test]
fn an_empty_clause_is_unsatisfiable() {
    let cnf = Cnf {
        num_vars: 1,
        clauses: vec![Vec::new()],
    };
    let mut solver = Solver::from_cnf(&cnf, Config::default());
    assert_eq!(solver.solve(Budget::unlimited()), Status::Unsat);
}

#[test]
fn a_clause_added_after_a_sat_answer_is_respected() {
    let a = Var::new(0);
    let mut solver = Solver::from_cnf(
        &Cnf {
            num_vars: 1,
            clauses: vec![],
        },
        Config::default(),
    );
    assert_eq!(solver.solve(Budget::unlimited()), Status::Sat);

    solver.add_clause(&[a.positive()]);
    assert_eq!(solver.solve(Budget::unlimited()), Status::Sat);
    assert_eq!(solver.model(), &[true]);

    solver.add_clause(&[a.negative()]);
    assert_eq!(solver.solve(Budget::unlimited()), Status::Unsat);
}
