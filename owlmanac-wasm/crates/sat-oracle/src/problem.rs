//! What a puzzle tells `sat-oracle` about itself: one shared trait, and one per strategy.
//!
//! There are two kinds of variable. *Design* variables describe the puzzle being made: which cell
//! is in which region, which clues to show. *Puzzle* variables describe a solution of the puzzle
//! that a design makes, and live in the checker.

use crate::data::proposer::Proposer;
use sat_core::random::Rng;
use sat_core::{Cnf, Lit, Var};

/// What every strategy needs: the puzzle a design makes, and its solutions.
pub trait Puzzle {
    /// The puzzle being made: regions, or clues.
    type Design;
    /// A solution of a designed puzzle.
    type Solution;

    /// The puzzle a design makes, as clauses over the puzzle variables.
    fn clauses(&self, design: &Self::Design) -> Cnf;

    /// The puzzle variables that make up a solution. Two solutions differ if one of these does.
    fn solution_vars(&self) -> Vec<Var>;

    /// Reads a solution out of a model of the puzzle.
    fn read_solution(&self, model: &[bool]) -> Self::Solution;
}

/// For `retry`: designs drawn at random, kept if their puzzle has exactly one solution.
pub trait RandomDesign: Puzzle {
    fn random_design(&self, rng: &mut Rng) -> Self::Design;
}

/// For `Synthesizer`: designs found by a solver, which learns from each that fails.
pub trait Synthesis: Puzzle {
    /// Creates the design variables, and adds the rules every design must follow, including
    /// whatever makes the intended solution valid.
    fn define(&self, proposer: &mut Proposer);

    /// Reads the design the proposer has just found.
    fn read_design(&self, proposer: &Proposer) -> Self::Design;

    /// Clauses for rules too awkward to state up front that this design breaks. Each must rule
    /// out this design and keep every valid one. Empty if it's fine.
    fn cuts(&self, design: &Self::Design) -> Vec<Vec<Lit>>;

    /// The intended solution, as puzzle literals. A solution making all of them true is it.
    fn intended(&self) -> Vec<Lit>;

    /// Adds rules to the proposer that stop `other` being a solution of any design it finds next.
    fn rule_out(&self, other: &Self::Solution, proposer: &mut Proposer);
}

/// For `dig`: a design made of clues, each of which can only rule solutions out. Taking a clue
/// away can only let solutions in, so removing clues one at a time while the answer stays unique
/// always ends at a puzzle where every clue is needed.
pub trait Clues {
    /// How many clues the full design has.
    fn clue_count(&self) -> usize;

    /// The rules that hold whatever clues are shown.
    fn rules(&self) -> Cnf;

    /// Clue `index`'s clauses, over the same variables as `rules`.
    fn clue(&self, index: usize) -> Vec<Vec<Lit>>;

    /// The intended solution, as literals. It must satisfy the rules and every clue.
    fn intended(&self) -> Vec<Lit>;
}
