//! What the integration tests share: formulas to try and exhaustive answers to compare against.

use proptest::collection::vec;
use proptest::prelude::*;
use sat_core::{Cnf, Config, Lit, Var};

/// Restarting and deleting far more often than the defaults, so small formulas exercise both.
pub fn eager_cleanup() -> Config {
    Config {
        restart_base: Some(2),
        reduce_interval: Some(4),
        ..Config::default()
    }
}

pub fn every_model(num_vars: u32) -> impl Iterator<Item = Vec<bool>> {
    (0u32..1 << num_vars).map(move |bits| (0..num_vars).map(|var| bits >> var & 1 == 1).collect())
}

pub fn satisfiable_by_enumeration(cnf: &Cnf) -> bool {
    every_model(cnf.num_vars).any(|model| cnf.is_satisfied_by(&model))
}

/// The formula with each of `units` added as a one-literal clause.
pub fn with_units(cnf: &Cnf, units: &[Lit]) -> Cnf {
    let mut extended = cnf.clone();
    extended
        .clauses
        .extend(units.iter().map(|&unit| vec![unit]));
    extended
}

pub fn any_lit(num_vars: u32) -> impl Strategy<Value = Lit> {
    (0..num_vars, any::<bool>()).prop_map(|(index, positive)| {
        let var = Var::new(index);
        if positive {
            var.positive()
        } else {
            var.negative()
        }
    })
}

pub fn small_cnf() -> impl Strategy<Value = Cnf> {
    (1u32..=10).prop_flat_map(|num_vars| {
        vec(vec(any_lit(num_vars), 1..=3), 0..=45)
            .prop_map(move |clauses| Cnf { num_vars, clauses })
    })
}

pub fn pigeonhole(pigeons: u32, holes: u32) -> Cnf {
    let var = |pigeon: u32, hole: u32| Var::new(pigeon * holes + hole);
    let mut clauses: Vec<Vec<Lit>> = (0..pigeons)
        .map(|pigeon| {
            (0..holes)
                .map(|hole| var(pigeon, hole).positive())
                .collect()
        })
        .collect();
    for hole in 0..holes {
        for first in 0..pigeons {
            for second in first + 1..pigeons {
                clauses.push(vec![
                    var(first, hole).negative(),
                    var(second, hole).negative(),
                ]);
            }
        }
    }
    Cnf {
        num_vars: pigeons * holes,
        clauses,
    }
}
