//! What each idea adds, on random 3-SAT at the hardness peak. Run with:
//! `cargo run --release -p sat-core --example ladder`

use sat_core::random::random_k_sat;
use sat_core::{Budget, Solver, Status, ladder};
use std::time::Instant;

const SEEDS: u64 = 20;
const CONFLICT_LIMIT: u64 = 2_000_000;

fn main() {
    println!(
        "{:>5}  {:<15} {:>7} {:>12} {:>14} {:>10}",
        "vars", "config", "solved", "conflicts", "propagations", "ms"
    );
    for num_vars in [50, 100, 150, 200] {
        let num_clauses = (num_vars as f64 * 4.26) as u32;
        for (name, config) in ladder() {
            // Without VSIDS, larger formulas take minutes each and mostly hit the limit.
            if num_vars > 150 && !config.vsids {
                continue;
            }
            let (mut solved, mut conflicts, mut propagations) = (0, 0, 0);
            let start = Instant::now();
            for seed in 0..SEEDS {
                let cnf = random_k_sat(num_vars, num_clauses, 3, seed);
                let mut solver = Solver::from_cnf(&cnf, config);
                if solver.solve(Budget::conflicts(CONFLICT_LIMIT)) != Status::Unknown {
                    solved += 1;
                }
                conflicts += solver.stats().conflicts;
                propagations += solver.stats().propagations;
            }
            let ms = start.elapsed().as_millis();
            println!(
                "{num_vars:>5}  {name:<15} {:>7} {conflicts:>12} {propagations:>14} {ms:>10}",
                format!("{solved}/{SEEDS}")
            );
        }
    }
}
