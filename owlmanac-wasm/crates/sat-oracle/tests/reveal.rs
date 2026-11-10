//! Every strategy on a toy problem with no puzzle in sight: four hidden bits, exactly two of them
//! set, and the design is which bits to reveal so that only one answer fits.

use sat_core::cardinality::{at_most, exactly};
use sat_core::random::Rng;
use sat_core::{Cnf, Lit, Var};
use sat_oracle::{
    Clues, Outcome, Proposer, Puzzle, RandomDesign, Synthesis, Synthesizer, dig, is_unique, retry,
};

const BITS: usize = 4;
const INTENDED: [bool; BITS] = [true, true, false, false];

/// `max_reveals` limits how many bits the design may reveal.
struct Reveal {
    max_reveals: usize,
}

/// Design variable `i` is "reveal bit `i`". Puzzle variable `i` is bit `i` itself.
fn reveal(bit: usize) -> Lit {
    Var::new(bit as u32).positive()
}

fn bit(index: usize, value: bool) -> Lit {
    let var = Var::new(index as u32);
    if value {
        var.positive()
    } else {
        var.negative()
    }
}

fn two_set() -> Vec<Vec<Lit>> {
    let bits: Vec<Lit> = (0..BITS).map(|index| bit(index, true)).collect();
    exactly(&bits, 2)
}

fn intended() -> Vec<Lit> {
    INTENDED
        .iter()
        .enumerate()
        .map(|(index, &value)| bit(index, value))
        .collect()
}

impl Puzzle for Reveal {
    type Design = Vec<bool>;
    type Solution = Vec<bool>;

    fn clauses(&self, design: &Vec<bool>) -> Cnf {
        let mut clauses = two_set();
        for (index, &shown) in design.iter().enumerate() {
            if shown {
                clauses.push(vec![bit(index, INTENDED[index])]);
            }
        }
        Cnf {
            num_vars: BITS as u32,
            clauses,
        }
    }

    fn solution_vars(&self) -> Vec<Var> {
        (0..BITS).map(|index| Var::new(index as u32)).collect()
    }

    fn read_solution(&self, model: &[bool]) -> Vec<bool> {
        model[..BITS].to_vec()
    }
}

impl Synthesis for Reveal {
    fn define(&self, proposer: &mut Proposer) {
        (0..BITS).for_each(|_| {
            proposer.new_var();
        });
        let reveals: Vec<Lit> = (0..BITS).map(reveal).collect();
        proposer.add_clauses(at_most(&reveals, self.max_reveals));
    }

    fn read_design(&self, proposer: &Proposer) -> Vec<bool> {
        (0..BITS)
            .map(|index| proposer.value(reveal(index)))
            .collect()
    }

    fn cuts(&self, _design: &Vec<bool>) -> Vec<Vec<Lit>> {
        Vec::new()
    }

    fn intended(&self) -> Vec<Lit> {
        intended()
    }

    /// Some revealed bit must differ from `other`.
    fn rule_out(&self, other: &Vec<bool>, proposer: &mut Proposer) {
        let clause: Vec<Lit> = (0..BITS)
            .filter(|&index| other[index] != INTENDED[index])
            .map(reveal)
            .collect();
        proposer.add_clause(&clause);
    }
}

impl RandomDesign for Reveal {
    fn random_design(&self, rng: &mut Rng) -> Vec<bool> {
        (0..BITS).map(|_| rng.below(2) == 1).collect()
    }
}

impl Clues for Reveal {
    fn clue_count(&self) -> usize {
        BITS
    }

    fn rules(&self) -> Cnf {
        Cnf {
            num_vars: BITS as u32,
            clauses: two_set(),
        }
    }

    fn clue(&self, index: usize) -> Vec<Vec<Lit>> {
        vec![vec![bit(index, INTENDED[index])]]
    }

    fn intended(&self) -> Vec<Lit> {
        intended()
    }
}

/// How many answers fit a design, by trying every one.
fn answers(design: &[bool]) -> usize {
    (0u32..1 << BITS)
        .map(|bits| {
            (0..BITS)
                .map(|index| bits >> index & 1 == 1)
                .collect::<Vec<bool>>()
        })
        .filter(|candidate| candidate.iter().filter(|&&set| set).count() == 2)
        .filter(|candidate| {
            (0..BITS).all(|index| !design[index] || candidate[index] == INTENDED[index])
        })
        .count()
}

#[test]
fn synthesis_finds_a_design_with_only_the_intended_answer() {
    let mut synthesizer = Synthesizer::new(Reveal { max_reveals: 4 });

    let Outcome::Unique(design) = synthesizer.run(100) else {
        panic!("revealing enough bits always works");
    };

    assert_eq!(answers(&design), 1);
    assert!(synthesizer.stats().counterexamples > 0);
}

#[test]
fn synthesis_proves_when_no_design_can_work() {
    // One revealed bit always leaves at least two answers.
    let mut synthesizer = Synthesizer::new(Reveal { max_reveals: 1 });
    assert_eq!(synthesizer.run(100), Outcome::Impossible);
}

#[test]
fn a_round_limit_stops_synthesis_and_running_again_carries_on() {
    let mut synthesizer = Synthesizer::new(Reveal { max_reveals: 4 });
    assert_eq!(synthesizer.run(1), Outcome::OutOfRounds);
    assert!(matches!(synthesizer.run(100), Outcome::Unique(_)));
}

#[test]
fn retry_keeps_only_a_unique_design() {
    let problem = Reveal { max_reveals: 4 };
    let mut rng = Rng::new(1);

    let (Outcome::Unique(design), stats) = retry(&problem, &mut rng, 1_000) else {
        panic!("random designs are often unique");
    };

    assert_eq!(answers(&design), 1);
    assert!(is_unique(&problem, &design));
    assert!(stats.rounds >= 1);
}

#[test]
fn digging_ends_with_every_remaining_clue_needed() {
    let problem = Reveal { max_reveals: 4 };
    for seed in 0..10 {
        let (shown, _) = dig(&problem, &mut Rng::new(seed), 0);

        assert_eq!(answers(&shown), 1, "seed {seed}");
        for index in (0..BITS).filter(|&index| shown[index]) {
            let mut without = shown.clone();
            without[index] = false;
            assert!(
                answers(&without) > 1,
                "seed {seed}: clue {index} wasn't needed"
            );
        }
    }
}
