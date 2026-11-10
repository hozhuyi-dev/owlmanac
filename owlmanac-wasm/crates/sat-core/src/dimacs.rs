//! Reads and writes DIMACS CNF, the standard text format for SAT formulas.

use crate::cnf::Cnf;
use crate::data::lit::Lit;
use std::fmt;
use std::fmt::Write;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DimacsError {
    MissingHeader,
    BadHeader(String),
    BadLiteral(String),
    VariableOutOfRange(i32),
    UnterminatedClause,
}

impl fmt::Display for DimacsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DimacsError::MissingHeader => write!(f, "no `p cnf` header before the first clause"),
            DimacsError::BadHeader(line) => write!(f, "bad header: {line}"),
            DimacsError::BadLiteral(token) => write!(f, "bad literal: {token}"),
            DimacsError::VariableOutOfRange(number) => {
                write!(f, "literal {number} is outside the declared variables")
            }
            DimacsError::UnterminatedClause => write!(f, "the last clause has no terminating 0"),
        }
    }
}

impl std::error::Error for DimacsError {}

pub fn parse(text: &str) -> Result<Cnf, DimacsError> {
    let mut num_vars = None;
    let mut clauses = Vec::new();
    let mut clause = Vec::new();

    for line in text.lines().map(str::trim) {
        // SATLIB's benchmark files end with a line holding only `%`.
        if line.starts_with('%') {
            break;
        }
        if line.is_empty() || line.starts_with('c') {
            continue;
        }
        if line.starts_with('p') {
            num_vars = Some(parse_header(line)?);
            continue;
        }

        let declared = num_vars.ok_or(DimacsError::MissingHeader)?;
        for token in line.split_whitespace() {
            let number: i32 = token
                .parse()
                .map_err(|_| DimacsError::BadLiteral(token.to_string()))?;
            if number == 0 {
                clauses.push(std::mem::take(&mut clause));
            } else if number.unsigned_abs() > declared {
                return Err(DimacsError::VariableOutOfRange(number));
            } else {
                clause.push(Lit::from_dimacs(number));
            }
        }
    }

    if !clause.is_empty() {
        return Err(DimacsError::UnterminatedClause);
    }
    let num_vars = num_vars.ok_or(DimacsError::MissingHeader)?;
    Ok(Cnf { num_vars, clauses })
}

pub fn write(cnf: &Cnf) -> String {
    let mut text = format!("p cnf {} {}\n", cnf.num_vars, cnf.clauses.len());
    for clause in &cnf.clauses {
        for lit in clause {
            write!(text, "{} ", lit.to_dimacs()).expect("writing to a String can't fail");
        }
        text.push_str("0\n");
    }
    text
}

/// Reads `p cnf <variables> <clauses>`, returning the variable count. The clause count is only a
/// hint, so it isn't enforced.
fn parse_header(line: &str) -> Result<u32, DimacsError> {
    let bad_header = || DimacsError::BadHeader(line.to_string());
    match line.split_whitespace().collect::<Vec<_>>().as_slice() {
        ["p", "cnf", vars, clauses] => {
            clauses.parse::<u64>().map_err(|_| bad_header())?;
            vars.parse().map_err(|_| bad_header())
        }
        _ => Err(bad_header()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_clauses_with_comments_and_line_breaks() {
        let cnf = parse("c a comment\np cnf 3 2\n1 -3 0\n2\n3 0\n").unwrap();

        assert_eq!(cnf.num_vars, 3);
        assert_eq!(
            cnf.clauses,
            vec![
                vec![Lit::from_dimacs(1), Lit::from_dimacs(-3)],
                vec![Lit::from_dimacs(2), Lit::from_dimacs(3)],
            ]
        );
    }

    #[test]
    fn writing_then_reading_gives_the_same_formula() {
        let cnf = parse("p cnf 3 2\n1 -3 0\n-2 3 0\n").unwrap();
        assert_eq!(parse(&write(&cnf)), Ok(cnf));
    }

    #[test]
    fn stops_at_satlibs_end_marker() {
        let cnf = parse("p cnf 1 1\n1 0\n%\n0\n").unwrap();
        assert_eq!(cnf.clauses.len(), 1);
    }

    #[test]
    fn keeps_an_empty_clause() {
        let cnf = parse("p cnf 1 1\n0\n").unwrap();
        assert_eq!(cnf.clauses, vec![Vec::new()]);
    }

    #[test]
    fn rejects_malformed_input() {
        assert_eq!(parse("1 0\n"), Err(DimacsError::MissingHeader));
        assert_eq!(
            parse("p cnf 1 1\n2 0\n"),
            Err(DimacsError::VariableOutOfRange(2))
        );
        assert_eq!(
            parse("p cnf 2 1\n1 2\n"),
            Err(DimacsError::UnterminatedClause)
        );
        assert_eq!(
            parse("p cnf 1 1\nx 0\n"),
            Err(DimacsError::BadLiteral("x".to_string()))
        );
        assert!(matches!(
            parse("p dnf 1 1\n"),
            Err(DimacsError::BadHeader(_))
        ));
    }
}
