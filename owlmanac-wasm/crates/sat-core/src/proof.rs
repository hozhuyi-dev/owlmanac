//! A DRAT proof: every clause learnt and deleted, in order, ending with the empty clause. An
//! independent checker can replay it, so an `Unsat` answer needn't be taken on trust.
//!
//! Only CDCL writes one. DPLL learns nothing, so it has no steps to record.

use crate::data::lit::Lit;
use crate::observer::Observer;
use std::fmt::Write;

#[derive(Clone, Default, Debug)]
pub struct DratProof {
    text: String,
}

impl DratProof {
    pub fn as_str(&self) -> &str {
        &self.text
    }

    fn write_clause(&mut self, clause: &[Lit]) {
        for lit in clause {
            write!(self.text, "{} ", lit.to_dimacs()).expect("writing to a String can't fail");
        }
        self.text.push_str("0\n");
    }
}

impl Observer for DratProof {
    fn on_learn(&mut self, clause: &[Lit], _lbd: u32) {
        self.write_clause(clause);
    }

    fn on_delete(&mut self, clause: &[Lit]) {
        self.text.push_str("d ");
        self.write_clause(clause);
    }

    fn on_unsat(&mut self) {
        self.write_clause(&[]);
    }
}
