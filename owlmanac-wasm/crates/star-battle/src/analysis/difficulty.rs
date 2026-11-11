//! How hard is a puzzle to reason through? Measured by the reasoning it needs, never by how long a
//! solver took, which depends as much on the solver's habits as on the puzzle. Two measures:
//!
//! - **Depth**, the deepest "what if". Depth 0 is forced moves alone. Depth 1 also allows a trial:
//!   "if I put a star here, the forced moves run into a contradiction, so it can't go here".
//!   Depth 2 allows a trial within a trial.
//! - **Length**, how far a trial's forced moves run before the contradiction, in rounds. Round 1
//!   is every move the trial forces outright, round 2 every move those force, and so on: a round
//!   is one look at the board. A trial is played out in your head, so a long chain is hard even
//!   when each move in it is simple.
//!
//! A player takes the easiest step on offer, so the grader does too: at each point it takes the
//! cheapest step (forced moves, then the shortest depth-1 chain, then depth 2), and the puzzle is
//! as hard as the hardest of those steps. Forced moves are played out round by round by the
//! grader itself, so a chain's length never depends on the order a solver happens to propagate in.

use crate::board::{Board, Placed};
use crate::encode::Encoding;
use crate::session::Session;
use sat_core::{Lit, Var};
use serde::Serialize;

/// The deepest trial the grader will try. Past it, a puzzle needs search.
pub const MAX_DEPTH: u32 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Grade {
    /// The deepest trial needed, or `None` if some cell needs more than `MAX_DEPTH`, or the puzzle
    /// has more than one solution.
    pub depth: Option<u32>,
    /// The longest chain a single "what if" needed, in rounds of forced moves, taking the shortest
    /// chain on offer at every point. Counts depth-1 steps only. 0 at depth 0.
    pub length: u32,
    /// How many cells needed a trial that deep. More means a longer hard stretch. 0 at depth 0.
    pub deepest_steps: u32,
}

/// A cell the player can work out next, and how many "what if"s deep the reasoning goes.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
pub struct Hint {
    pub placed: Placed,
    pub depth: u32,
}

impl Session {
    /// How hard it is to finish this board, or `None` if it can't be finished. Meant for a puzzle
    /// with exactly one solution: with more, no amount of reasoning settles every cell.
    pub fn grade(&self, board: &Board) -> Option<Grade> {
        let rules = Rules::new(&self.encoding);
        let mut known = rules.settle(&self.literals(board), &mut Vec::new())?;
        let mut grade = Grade {
            depth: Some(0),
            length: 0,
            deepest_steps: 0,
        };
        while !known.is_complete() {
            let Some(step) = rules.cheapest_step(&known) else {
                grade.depth = None;
                return Some(grade);
            };
            let deepest = grade.depth.expect("still within MAX_DEPTH");
            if step.depth > deepest {
                grade.depth = Some(step.depth);
                grade.deepest_steps = 0;
            }
            if step.depth >= deepest {
                grade.deepest_steps += 1;
            }
            if step.depth == 1 {
                grade.length = grade.length.max(step.length);
            }
            rules.play_out(&mut known, &[step.lit], &mut Vec::new())?;
        }
        Some(grade)
    }

    /// The easiest cell to work out next: one the forced moves give, else the cell whose "what if"
    /// clashes soonest, else a "what if" inside one. This is one step of `grade`. `None` if the
    /// board is finished, can't be finished, or needs deeper reasoning than that.
    pub fn hint(&self, board: &Board) -> Option<Hint> {
        let rules = Rules::new(&self.encoding);
        let marks = self.literals(board);
        let mut trail = Vec::new();
        let known = rules.settle(&marks, &mut trail)?;
        // The trail is in rounds, so the first new cell on it is the most plainly forced.
        if let Some(&forced) = trail.iter().find(|lit| !marks.contains(lit)) {
            return Some(Hint {
                placed: self.encoding.placed(forced),
                depth: 0,
            });
        }
        let step = rules.cheapest_step(&known)?;
        Some(Hint {
            placed: self.encoding.placed(step.lit),
            depth: step.depth,
        })
    }
}

/// What's known of each cell: `Some(true)` for a star, `Some(false)` for empty.
#[derive(Clone)]
struct Known(Vec<Option<bool>>);

impl Known {
    fn value(&self, lit: Lit) -> Option<bool> {
        self.0[lit.var().index()].map(|star| star == lit.is_positive())
    }

    fn set(&mut self, lit: Lit) {
        self.0[lit.var().index()] = Some(lit.is_positive());
    }

    fn is_complete(&self) -> bool {
        self.0.iter().all(Option::is_some)
    }

    /// Both trials for each cell not yet known, in cell order: a star, then empty.
    fn trials(&self) -> impl Iterator<Item = Lit> + '_ {
        (0..self.0.len())
            .filter(|&index| self.0[index].is_none())
            .flat_map(|index| {
                let var = Var::new(index as u32);
                [var.positive(), var.negative()]
            })
    }
}

/// One step of reasoning: `lit` follows, by a trial `depth` deep whose forced moves clash after
/// `length` rounds (`length` is measured for depth 1 only).
struct Step {
    lit: Lit,
    depth: u32,
    length: u32,
}

/// The puzzle's clauses, set up for playing out forced moves round by round.
struct Rules {
    clauses: Vec<Vec<Lit>>,
    /// For each literal, the clauses it appears in.
    containing: Vec<Vec<usize>>,
    /// Literals the rules state outright, such as both cells of a two-cell region.
    facts: Vec<Lit>,
    /// A rule no board can keep, such as a region with fewer cells than stars.
    impossible: bool,
    num_cells: usize,
}

impl Rules {
    fn new(encoding: &Encoding) -> Self {
        let cnf = encoding.cnf();
        let mut containing = vec![Vec::new(); 2 * encoding.num_cells()];
        for (index, clause) in cnf.clauses.iter().enumerate() {
            for lit in clause {
                containing[lit.index()].push(index);
            }
        }
        Rules {
            facts: cnf
                .clauses
                .iter()
                .filter(|clause| clause.len() == 1)
                .map(|clause| clause[0])
                .collect(),
            impossible: cnf.clauses.iter().any(Vec::is_empty),
            clauses: cnf.clauses,
            containing,
            num_cells: encoding.num_cells(),
        }
    }

    /// Everything the forced moves give from the rules and `marks`, or `None` if they clash.
    /// Everything placed is pushed onto `trail`, round by round.
    fn settle(&self, marks: &[Lit], trail: &mut Vec<Lit>) -> Option<Known> {
        if self.impossible {
            return None;
        }
        let mut known = Known(vec![None; self.num_cells]);
        let mut start = self.facts.clone();
        start.extend_from_slice(marks);
        self.play_out(&mut known, &start, trail)?;
        Some(known)
    }

    /// Places `lits` (round 0), then each round everything the last one forced, until nothing more
    /// is forced, adding it all to `known`. `None` if a rule breaks, which happens in the round
    /// `clash` says. `known` must already hold everything its own forced moves give.
    fn play_out(&self, known: &mut Known, lits: &[Lit], trail: &mut Vec<Lit>) -> Option<()> {
        match self.clash(known, lits, trail) {
            Some(_) => None,
            None => Some(()),
        }
    }

    /// The round in which playing out `lits` from `known` breaks a rule, or `None` if it doesn't.
    fn clash(&self, known: &mut Known, lits: &[Lit], trail: &mut Vec<Lit>) -> Option<u32> {
        let mut round = 0;
        let mut this_round = lits.to_vec();
        let mut placed = Vec::new();
        loop {
            placed.clear();
            for &lit in &this_round {
                match known.value(lit) {
                    Some(true) => {}
                    // Forced both ways in the same round.
                    Some(false) => return Some(round),
                    None => {
                        known.set(lit);
                        placed.push(lit);
                    }
                }
            }
            if placed.is_empty() {
                return None;
            }
            trail.extend_from_slice(&placed);

            this_round.clear();
            for &lit in &placed {
                for &index in &self.containing[(!lit).index()] {
                    let mut open = None;
                    let mut open_count = 0;
                    let mut satisfied = false;
                    for &other in &self.clauses[index] {
                        match known.value(other) {
                            Some(true) => {
                                satisfied = true;
                                break;
                            }
                            Some(false) => {}
                            None => {
                                open = Some(other);
                                open_count += 1;
                            }
                        }
                    }
                    match (satisfied, open_count) {
                        (true, _) => {}
                        (false, 0) => return Some(round),
                        (false, 1) => this_round.push(open.expect("one open literal")),
                        _ => {}
                    }
                }
            }
            round += 1;
        }
    }

    /// How many rounds the forced moves after `trial` run before a rule breaks, or `None` if
    /// they never do.
    fn chain(&self, known: &Known, trial: Lit) -> Option<u32> {
        self.clash(&mut known.clone(), &[trial], &mut Vec::new())
    }

    /// Whether `trial` leads to a contradiction using trials up to `depth` deep inside it.
    fn refutes(&self, known: &Known, trial: Lit, depth: u32) -> bool {
        let mut known = known.clone();
        if self
            .play_out(&mut known, &[trial], &mut Vec::new())
            .is_none()
        {
            return true;
        }
        depth > 0 && !self.close(&mut known, depth)
    }

    /// Adds to `known` everything trials up to `depth` deep give, or returns false if that reaches
    /// a contradiction. It sweeps the cells until a sweep settles none: one new fact can make
    /// another trial fail, and the order doesn't change where it ends up.
    fn close(&self, known: &mut Known, depth: u32) -> bool {
        loop {
            let mut progress = false;
            for index in 0..self.num_cells {
                if known.0[index].is_some() {
                    continue;
                }
                let var = Var::new(index as u32);
                for trial in [var.positive(), var.negative()] {
                    if self.refutes(known, trial, depth - 1) {
                        if self.play_out(known, &[!trial], &mut Vec::new()).is_none() {
                            return false;
                        }
                        progress = true;
                        break;
                    }
                }
            }
            if !progress {
                return true;
            }
        }
    }

    /// The easiest step from `known`, which holds everything the forced moves give: the depth-1
    /// trial that clashes soonest (the first in cell order among equals), else the first trial
    /// that a deeper one refutes. `None` if no trial up to `MAX_DEPTH` settles a cell.
    fn cheapest_step(&self, known: &Known) -> Option<Step> {
        let shortest = known
            .trials()
            .filter_map(|trial| Some((self.chain(known, trial)?, trial)))
            .min_by_key(|&(length, _)| length);
        if let Some((length, trial)) = shortest {
            return Some(Step {
                lit: !trial,
                depth: 1,
                length,
            });
        }
        (2..=MAX_DEPTH).find_map(|depth| {
            let trial = known
                .trials()
                .find(|&trial| self.refutes(known, trial, depth - 1))?;
            Some(Step {
                lit: !trial,
                depth,
                length: 0,
            })
        })
    }
}
