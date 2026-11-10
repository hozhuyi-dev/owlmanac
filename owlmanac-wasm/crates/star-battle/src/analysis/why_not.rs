//! Why can't this mark go here? Answers with a chain of deductions when one exists.

use crate::board::{Board, Placed};
use crate::rules::Rule;
use crate::session::Session;
use sat_core::{Contradiction, Deduction, Lit};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct Step {
    /// What was worked out.
    pub placed: Placed,
    /// The rule that forced it.
    pub rule: Rule,
    /// The earlier marks that made the rule bite.
    pub because: Vec<Placed>,
}

/// A rule that can't be met given these marks.
#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct Clash {
    pub rule: Rule,
    pub marks: Vec<Placed>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct Chain {
    /// Only the steps the clash depends on, in the order they follow.
    pub steps: Vec<Step>,
    pub clash: Clash,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum WhyNot {
    /// Nothing is wrong with it: some solution has this mark.
    Possible,
    /// Following the rules step by step runs into a clash.
    Deduced(Chain),
    /// No solution has this mark, but showing it takes trying cases, so this is only a minimal
    /// set of marks and rules that can't stand together.
    NeedsCases(crate::analysis::blame::Blame),
}

impl Session {
    /// Asks about `proposal`, replacing any mark already on its cell.
    pub fn why_not(&mut self, board: &Board, proposal: Placed) -> WhyNot {
        let mut without = board.clone();
        without.clear(proposal.cell);

        if let Some(chain) = self.deduction_chain(&without, proposal) {
            return WhyNot::Deduced(chain);
        }
        match self.blame(&without.with(proposal)) {
            None => WhyNot::Possible,
            Some(blame) => WhyNot::NeedsCases(blame),
        }
    }

    fn deduction_chain(&mut self, board: &Board, proposal: Placed) -> Option<Chain> {
        let mut assumptions = self.deduce.guards().to_vec();
        assumptions.extend(self.literals(board));
        assumptions.push(self.encoding.lit(proposal));

        let deductions = self.deduce.solver.propagate_assuming(&assumptions);
        let contradiction = deductions.contradiction?;
        let (clash, mut needed) = self.clash_from(&contradiction, &deductions.steps);

        let mut kept = Vec::new();
        for deduction in deductions.steps.iter().rev() {
            let Some(reason) = &deduction.reason else {
                continue;
            };
            if self.deduce.rule_of(deduction.lit).is_some() || !needed.contains(&deduction.lit) {
                continue;
            }
            let (rule, because) = self.read_clause(reason, Some(deduction.lit));
            needed.extend(because.iter().map(|&placed| self.encoding.lit(placed)));
            kept.push(Step {
                placed: self.encoding.placed(deduction.lit),
                rule,
                because,
            });
        }
        kept.reverse();
        Some(Chain { steps: kept, clash })
    }

    /// Describes what went wrong, and which assigned literals it rests on.
    fn clash_from(
        &self,
        contradiction: &Contradiction,
        steps: &[Deduction],
    ) -> (Clash, BTreeSet<Lit>) {
        let (rule, marks, assumed) = match contradiction {
            Contradiction::Clause(clause) => {
                let (rule, marks) = self.read_clause(clause, None);
                (rule, marks, None)
            }
            Contradiction::Assumption(lit) => match self.deduce.rule_of(*lit) {
                Some(rule) => (rule, Vec::new(), None),
                None => {
                    // The cell was already forced the other way, so the clause that forced it
                    // is the one this assumption breaks.
                    let reason = steps
                        .iter()
                        .find(|step| step.lit == !*lit)
                        .and_then(|step| step.reason.as_ref())
                        .expect("a board has one mark per cell, so the opposite was deduced");
                    let (rule, marks) = self.read_clause(reason, Some(!*lit));
                    (rule, marks, Some(self.encoding.placed(*lit)))
                }
            },
        };

        let needed = marks
            .iter()
            .map(|&placed| self.encoding.lit(placed))
            .collect();
        let mut all_marks = marks;
        all_marks.extend(assumed);
        (
            Clash {
                rule,
                marks: all_marks,
            },
            needed,
        )
    }

    /// Reads a guarded clause whose literals are false, apart from `implied`: which rule it is,
    /// and the marks that falsify it.
    fn read_clause(&self, clause: &[Lit], implied: Option<Lit>) -> (Rule, Vec<Placed>) {
        let rule = clause
            .iter()
            .find_map(|&lit| self.deduce.rule_of(lit))
            .expect("every clause of this solver carries its rule's guard");
        let marks = clause
            .iter()
            .filter(|&&lit| Some(lit) != implied && self.deduce.rule_of(lit).is_none())
            .map(|&lit| self.encoding.placed(!lit))
            .collect();
        (rule, marks)
    }
}
