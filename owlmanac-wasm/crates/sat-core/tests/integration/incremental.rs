//! A solver reused across queries must answer as a fresh one would, and say why.

use super::support::{
    any_lit, eager_cleanup, every_model, satisfiable_by_enumeration, small_cnf, with_units,
};
use proptest::collection::vec;
use proptest::prelude::*;
use sat_core::{Budget, Cnf, Config, Contradiction, Lit, Solver, Status, Var};

fn cdcl_configs() -> [Config; 3] {
    [Config::basic_cdcl(), Config::default(), eager_cleanup()]
}

fn cnf_and_queries() -> impl Strategy<Value = (Cnf, Vec<Vec<Lit>>)> {
    small_cnf().prop_flat_map(|cnf| {
        let queries = vec(vec(any_lit(cnf.num_vars), 0..=4), 1..=4);
        (Just(cnf), queries)
    })
}

fn holds(model: &[bool], lit: Lit) -> bool {
    model[lit.var().index()] == lit.is_positive()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn reused_solvers_answer_every_query_correctly((cnf, queries) in cnf_and_queries()) {
        for config in cdcl_configs() {
            let mut solver = Solver::from_cnf(&cnf, config);
            for assumptions in &queries {
                let status = solver.solve_assuming(assumptions, Budget::unlimited());
                let expected_sat = satisfiable_by_enumeration(&with_units(&cnf, assumptions));
                prop_assert_eq!(status == Status::Sat, expected_sat);
                prop_assert_ne!(status, Status::Unknown);

                if expected_sat {
                    prop_assert!(cnf.is_satisfied_by(solver.model()));
                    prop_assert!(assumptions.iter().all(|&lit| holds(solver.model(), lit)));
                } else {
                    let failed = solver.failed_assumptions();
                    prop_assert!(failed.iter().all(|lit| assumptions.contains(lit)));
                    prop_assert!(!satisfiable_by_enumeration(&with_units(&cnf, failed)));
                }
            }
        }
    }

    #[test]
    fn a_query_can_be_solved_in_slices((cnf, queries) in cnf_and_queries()) {
        let assumptions = &queries[0];
        let expected_sat = satisfiable_by_enumeration(&with_units(&cnf, assumptions));
        let mut solver = Solver::from_cnf(&cnf, eager_cleanup());

        let mut status = solver.solve_assuming(assumptions, Budget::conflicts(1));
        while status == Status::Unknown {
            status = solver.resume(Budget::conflicts(1));
        }

        prop_assert_eq!(status == Status::Sat, expected_sat);
    }

    #[test]
    fn clauses_added_between_queries_count(cnf in small_cnf()) {
        let half = cnf.clauses.len() / 2;
        let first_half = Cnf { num_vars: cnf.num_vars, clauses: cnf.clauses[..half].to_vec() };
        let mut solver = Solver::from_cnf(&first_half, eager_cleanup());
        solver.solve(Budget::unlimited());

        cnf.clauses[half..].iter().for_each(|clause| solver.add_clause(clause));

        let status = solver.solve(Budget::unlimited());
        prop_assert_eq!(status == Status::Sat, satisfiable_by_enumeration(&cnf));
    }

    #[test]
    fn deductions_follow_from_the_assumptions((cnf, queries) in cnf_and_queries()) {
        let assumptions = &queries[0];
        let mut solver = Solver::from_cnf(&cnf, Config::default());

        let deductions = solver.propagate_assuming(assumptions);

        let models: Vec<Vec<bool>> = every_model(cnf.num_vars)
            .filter(|model| with_units(&cnf, assumptions).is_satisfied_by(model))
            .collect();
        if deductions.contradiction.is_some() {
            prop_assert!(models.is_empty());
        } else {
            for step in &deductions.steps {
                prop_assert!(models.iter().all(|model| holds(model, step.lit)));
            }
        }
    }

    #[test]
    fn each_reason_rests_only_on_earlier_steps((cnf, queries) in cnf_and_queries()) {
        let mut solver = Solver::from_cnf(&cnf, Config::default());

        let deductions = solver.propagate_assuming(&queries[0]);

        let mut known: Vec<Lit> = Vec::new();
        for step in &deductions.steps {
            if let Some(reason) = &step.reason {
                prop_assert!(reason.contains(&step.lit));
                for &other in reason.iter().filter(|&&other| other != step.lit) {
                    prop_assert!(known.contains(&!other));
                }
            }
            known.push(step.lit);
        }
        if let Some(Contradiction::Clause(clause)) = &deductions.contradiction {
            prop_assert!(clause.iter().all(|lit| known.contains(&!*lit)));
        }
    }

    #[test]
    fn probing_leaves_the_answer_unchanged((cnf, queries) in cnf_and_queries()) {
        let mut solver = Solver::from_cnf(&cnf, Config::default());
        solver.propagate_assuming(&queries[0]);

        let status = solver.solve(Budget::unlimited());

        prop_assert_eq!(status == Status::Sat, satisfiable_by_enumeration(&cnf));
    }
}

#[test]
fn only_the_assumptions_responsible_are_blamed() {
    let (a, b, c) = (Var::new(0), Var::new(1), Var::new(2));
    let cnf = Cnf {
        num_vars: 3,
        clauses: vec![vec![a.negative(), b.negative()]],
    };
    let mut solver = Solver::from_cnf(&cnf, Config::default());

    let status = solver.solve_assuming(
        &[c.positive(), a.positive(), b.positive()],
        Budget::unlimited(),
    );

    assert_eq!(status, Status::Unsat);
    let mut failed = solver.failed_assumptions().to_vec();
    failed.sort();
    assert_eq!(failed, vec![a.positive(), b.positive()]);
}

#[test]
fn a_formula_with_no_solution_blames_no_assumption() {
    let a = Var::new(0);
    let cnf = Cnf {
        num_vars: 1,
        clauses: vec![vec![a.positive()], vec![a.negative()]],
    };
    let mut solver = Solver::from_cnf(&cnf, Config::default());

    assert_eq!(
        solver.solve_assuming(&[a.positive()], Budget::unlimited()),
        Status::Unsat
    );
    assert!(solver.failed_assumptions().is_empty());
}

#[test]
fn unsat_under_assumptions_does_not_stick() {
    let (a, b) = (Var::new(0), Var::new(1));
    let cnf = Cnf {
        num_vars: 2,
        clauses: vec![vec![a.negative(), b.negative()]],
    };
    let mut solver = Solver::from_cnf(&cnf, Config::default());

    let contradictory = [a.positive(), b.positive()];
    assert_eq!(
        solver.solve_assuming(&contradictory, Budget::unlimited()),
        Status::Unsat
    );

    assert_eq!(solver.solve(Budget::unlimited()), Status::Sat);
}

#[test]
fn propagation_reports_the_chain_in_order() {
    let (a, b, c) = (Var::new(0), Var::new(1), Var::new(2));
    let cnf = Cnf {
        num_vars: 3,
        clauses: vec![
            vec![a.negative(), b.positive()],
            vec![b.negative(), c.positive()],
        ],
    };
    let mut solver = Solver::from_cnf(&cnf, Config::default());

    let deductions = solver.propagate_assuming(&[a.positive()]);

    let lits: Vec<Lit> = deductions.steps.iter().map(|step| step.lit).collect();
    assert_eq!(lits, vec![a.positive(), b.positive(), c.positive()]);
    assert_eq!(deductions.steps[0].reason, None);
    assert_eq!(deductions.contradiction, None);
}
