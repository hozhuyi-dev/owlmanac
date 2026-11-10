//! What a run of the synthesizer ends with, and how hard it worked.

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Outcome<D> {
    /// A design whose puzzle has exactly the intended solution.
    Unique(D),
    /// No design can make the intended solution the only one.
    Impossible,
    /// The round limit came first. Run again to carry on.
    OutOfRounds,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Stats {
    /// Designs proposed.
    pub rounds: u32,
    /// Designs rejected for breaking a rule checked outside the solver, each adding cuts.
    pub cuts: u32,
    /// Second solutions found, each turned into a rule for the proposer.
    pub counterexamples: u32,
    /// Dead ends the proposer hit, finding designs.
    pub proposer_conflicts: u64,
    /// Dead ends the checkers hit, most of them proving no second solution exists.
    pub checker_conflicts: u64,
}
