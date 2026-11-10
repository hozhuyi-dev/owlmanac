//! The solver from the command line, in the SAT competition's conventions, so outside tools can
//! check its answers: prints `s` and `v` lines and exits with 10 (SAT) or 20 (UNSAT).

use sat_core::{Budget, Cnf, Config, DratProof, Solver, Status, dimacs, random};
use std::fs;
use std::process::ExitCode;

const USAGE: &str = "usage:
  sat-cli solve <formula.cnf> [--proof <out.drat>]
  sat-cli random <vars> <clauses> <k> <seed>";

const EXIT_SAT: u8 = 10;
const EXIT_UNSAT: u8 = 20;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args.iter().map(String::as_str).collect::<Vec<_>>()) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("{message}\n\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[&str]) -> Result<ExitCode, String> {
    match args {
        ["solve", formula] => solve(formula, None),
        ["solve", formula, "--proof", proof] => solve(formula, Some(proof)),
        ["random", vars, clauses, k, seed] => {
            let cnf =
                random::random_k_sat(number(vars)?, number(clauses)?, number(k)?, number(seed)?);
            print!("{}", dimacs::write(&cnf));
            Ok(ExitCode::SUCCESS)
        }
        _ => Err("unknown command".to_string()),
    }
}

fn number<T: std::str::FromStr>(text: &str) -> Result<T, String> {
    text.parse().map_err(|_| format!("not a number: {text}"))
}

fn solve(formula: &str, proof_path: Option<&str>) -> Result<ExitCode, String> {
    let cnf = read_formula(formula)?;
    let mut solver = Solver::with_observer(Config::default(), DratProof::default());
    solver.add_cnf(&cnf);
    let status = solver.solve(Budget::unlimited());

    if let Some(path) = proof_path {
        fs::write(path, solver.observer().as_str()).map_err(|error| format!("{path}: {error}"))?;
    }
    match status {
        Status::Sat => {
            println!("s SATISFIABLE");
            println!("v {} 0", model_line(solver.model()));
            Ok(ExitCode::from(EXIT_SAT))
        }
        Status::Unsat => {
            println!("s UNSATISFIABLE");
            Ok(ExitCode::from(EXIT_UNSAT))
        }
        Status::Unknown => unreachable!("an unlimited budget never runs out"),
    }
}

fn read_formula(path: &str) -> Result<Cnf, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?;
    dimacs::parse(&text).map_err(|error| format!("{path}: {error}"))
}

fn model_line(model: &[bool]) -> String {
    let literals: Vec<String> = model
        .iter()
        .enumerate()
        .map(|(index, &value)| {
            let number = index as i64 + 1;
            (if value { number } else { -number }).to_string()
        })
        .collect();
    literals.join(" ")
}
