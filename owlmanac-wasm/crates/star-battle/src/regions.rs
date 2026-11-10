//! Making Star Battle puzzles with `sat-oracle`, three ways:
//!
//! - `Strategy::Retry`: random regions, kept if the puzzle has exactly one solution.
//! - `Strategy::AroundStars`: choose the stars first, grow regions around pairs of them, and keep
//!   the result if no other solution exists. The intended solution is always valid, so a try can
//!   only fail by being ambiguous, never by having no solution.
//! - `Strategy::Synthesize`: choose the stars first, then let a solver draw the regions, learning
//!   from each second solution the checker finds.

use crate::encode::Encoding;
use crate::puzzle::{Cell, Puzzle};
use sat_core::cardinality::{at_least, at_most, exactly};
use sat_core::random::Rng;
use sat_core::{Budget, Cnf, Config, Lit, Solver, Status, Var};
use sat_oracle::{
    Checker, Outcome, Proposer, Puzzle as OraclePuzzle, RandomDesign, Stats, Synthesis,
    Synthesizer, retry,
};
use serde::Serialize;
use std::collections::VecDeque;

/// Proposals to allow one intended solution before the synthesizer tries another.
const ROUNDS_PER_START: u32 = 2_000;
/// Random designs to try between checks on the clock. Retrying never gives up on its own.
const TRIES_PER_BATCH: u32 = 1_000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Strategy {
    Retry,
    AroundStars,
    Synthesize,
}

/// How hard generation had to work, for the post's charts.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Generation {
    /// Intended solutions the synthesizer tried. Always 1 for the other strategies.
    pub starts: u32,
    /// The work done on the attempt that succeeded: designs tried, second solutions, conflicts.
    pub oracle: Stats,
}

/// A puzzle with exactly one solution, the same one for the same seed and strategy.
pub fn generate(size: usize, stars: usize, seed: u64, strategy: Strategy) -> (Puzzle, Generation) {
    let mut rng = Rng::new(seed);
    let mut generation = Generation::default();
    let regions = loop {
        generation.starts += 1;
        let (outcome, stats) = match strategy {
            Strategy::Retry | Strategy::AroundStars => {
                let sampler = RandomRegions {
                    size,
                    stars,
                    around_stars: strategy == Strategy::AroundStars,
                };
                retry(&sampler, &mut rng, TRIES_PER_BATCH)
            }
            Strategy::Synthesize => {
                let intended = random_solution(size, stars, &mut rng);
                let groups = group_nearby(&intended, stars, &mut rng);
                let hint = grow_connected(&groups, size, &mut rng)
                    .unwrap_or_else(|| random_regions(size, &mut rng));
                let mut synthesizer =
                    Synthesizer::new(RegionDesign::new(size, stars, intended, &groups, hint));
                let outcome = synthesizer.run(ROUNDS_PER_START);
                (outcome, synthesizer.stats())
            }
        };
        if let Outcome::Unique(regions) = outcome {
            generation.oracle = stats;
            break regions;
        }
    };
    if strategy != Strategy::Synthesize {
        generation.starts = 1;
    }
    let puzzle = Puzzle::new(size, stars, regions).expect("generated regions are always valid");
    (puzzle, generation)
}

/// Region layouts drawn at random, for `retry`.
struct RandomRegions {
    size: usize,
    stars: usize,
    /// Grow regions around pairs of a chosen solution's stars, rather than from random cells.
    around_stars: bool,
}

impl OraclePuzzle for RandomRegions {
    type Design = Vec<usize>;
    type Solution = Vec<Cell>;

    fn clauses(&self, regions: &Vec<usize>) -> Cnf {
        region_clauses(self.size, self.stars, regions)
    }

    fn solution_vars(&self) -> Vec<Var> {
        cell_vars(self.size)
    }

    fn read_solution(&self, model: &[bool]) -> Vec<Cell> {
        read_stars(self.size, model)
    }
}

impl RandomDesign for RandomRegions {
    fn random_design(&self, rng: &mut Rng) -> Vec<usize> {
        if !self.around_stars {
            return random_regions(self.size, rng);
        }
        let (_, regions) = stars_then_regions(self.size, self.stars, rng);
        regions
    }
}

/// Chooses a valid placement of stars, then grows regions around nearby pairs of them, trying
/// again until the pairs can all be joined.
fn stars_then_regions(size: usize, stars: usize, rng: &mut Rng) -> (Vec<Cell>, Vec<usize>) {
    loop {
        let intended = random_solution(size, stars, rng);
        let groups = group_nearby(&intended, stars, rng);
        if let Some(regions) = grow_connected(&groups, size, rng) {
            return (intended, regions);
        }
    }
}

/// One try of `Strategy::AroundStars`, kept whole so a page can show it.
#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct Attempt {
    /// The stars chosen first, row by row.
    pub stars: Vec<Cell>,
    /// Each cell's region, row by row.
    pub regions: Vec<usize>,
    /// Another solution of these regions, or `None` if the chosen stars are the only one.
    pub other: Option<Vec<Cell>>,
}

/// `Strategy::AroundStars` one try at a time, for watching it work.
pub struct Attempts {
    size: usize,
    stars: usize,
    rng: Rng,
}

impl Attempts {
    pub fn new(size: usize, stars: usize, seed: u64) -> Self {
        Attempts {
            size,
            stars,
            rng: Rng::new(seed),
        }
    }

    pub fn next_attempt(&mut self) -> Attempt {
        let (mut stars, regions) = stars_then_regions(self.size, self.stars, &mut self.rng);
        stars.sort();
        let sampler = RandomRegions {
            size: self.size,
            stars: self.stars,
            around_stars: true,
        };
        let mut checker = Checker::new(&sampler, &regions);
        let intended: Vec<Lit> = stars
            .iter()
            .map(|cell| Var::new((cell.row * self.size + cell.col) as u32).positive())
            .collect();
        checker.exclude(&intended);
        let other = checker.next_solution();
        Attempt {
            stars,
            regions,
            other,
        }
    }
}

/// The puzzle a region layout makes, over one variable per cell.
fn region_clauses(size: usize, stars: usize, regions: &[usize]) -> Cnf {
    let puzzle = Puzzle::new(size, stars, regions.to_vec()).expect("every region has a cell");
    Encoding::new(&puzzle).cnf()
}

fn cell_vars(size: usize) -> Vec<Var> {
    (0..size * size)
        .map(|index| Var::new(index as u32))
        .collect()
}

fn read_stars(size: usize, model: &[bool]) -> Vec<Cell> {
    (0..size * size)
        .filter(|&index| model[index])
        .map(|index| Cell {
            row: index / size,
            col: index % size,
        })
        .collect()
}

/// Design variable "cell `c` is in region `r`", one per pair. Puzzle variables are the cells.
struct RegionDesign {
    size: usize,
    stars: usize,
    /// The intended solution's stars, row by row.
    intended: Vec<Cell>,
    /// For each intended star, the region it's pinned to.
    pinned: Vec<(Cell, usize)>,
    /// Regions to suggest as the proposer's starting point, each cell's region row by row.
    hint: Vec<usize>,
}

impl RegionDesign {
    fn new(
        size: usize,
        stars: usize,
        mut intended: Vec<Cell>,
        groups: &[Vec<Cell>],
        hint: Vec<usize>,
    ) -> Self {
        intended.sort();
        let pinned = groups
            .iter()
            .enumerate()
            .flat_map(|(region, group)| group.iter().map(move |&cell| (cell, region)))
            .collect();
        RegionDesign {
            size,
            stars,
            intended,
            pinned,
            hint,
        }
    }

    fn in_region(&self, cell: Cell, region: usize) -> Lit {
        Var::new(((cell.row * self.size + cell.col) * self.size + region) as u32).positive()
    }

    fn cell(&self, index: usize) -> Cell {
        Cell {
            row: index / self.size,
            col: index % self.size,
        }
    }
}

impl RegionDesign {
    /// Keeps every region in one piece, stated up front: each cell must be reachable from its
    /// region's anchor star within `2 × size` steps, moving only between cells of the same region.
    /// Checking pieces lazily instead fails badly here: there are so many ways to draw broken
    /// regions that the proposer never runs out of new ones.
    ///
    /// Every helper variable also gets a suggested value from the hint, so the proposer starts
    /// from a whole consistent picture rather than having to work out the distances itself.
    fn connect(&self, proposer: &mut Proposer) {
        let cells = self.size * self.size;
        let steps = 2 * self.size;
        let anchors = self.anchors();
        let hinted = self.hinted_distances(&anchors);
        let within = |cell: usize, k: usize| hinted[cell].is_some_and(|distance| distance <= k);

        // reach[c][k]: cell c is within k steps of its region's anchor.
        let reach: Vec<Vec<Lit>> = (0..cells)
            .map(|c| {
                (0..=steps)
                    .map(|k| {
                        let lit = proposer.new_var().positive();
                        proposer.suggest(if within(c, k) { lit } else { !lit });
                        lit
                    })
                    .collect()
            })
            .collect();

        for c in 0..cells {
            let at_anchor = reach[c][0];
            proposer.add_clause(&[if anchors.contains(&c) {
                at_anchor
            } else {
                !at_anchor
            }]);
            proposer.add_clause(&[reach[c][steps]]);

            // same: this neighbour is in the same region as c.
            let same: Vec<(usize, Lit)> = neighbours(c, self.size)
                .map(|n| {
                    let same = proposer.new_var().positive();
                    proposer.suggest(if self.hint[c] == self.hint[n] {
                        same
                    } else {
                        !same
                    });
                    for region in 0..self.size {
                        proposer.add_clause(&[
                            !same,
                            !self.in_region(self.cell(c), region),
                            self.in_region(self.cell(n), region),
                        ]);
                    }
                    (n, same)
                })
                .collect();

            // Within k steps means within k - 1 already, or next to a same-region cell that is.
            for k in 1..=steps {
                let mut clause = vec![!reach[c][k], reach[c][k - 1]];
                for &(n, same_region) in &same {
                    let via = proposer.new_var().positive();
                    let hinted_via = self.hint[c] == self.hint[n] && within(n, k - 1);
                    proposer.suggest(if hinted_via { via } else { !via });
                    proposer.add_clause(&[!via, same_region]);
                    proposer.add_clause(&[!via, reach[n][k - 1]]);
                    clause.push(via);
                }
                proposer.add_clause(&clause);
            }
        }
    }

    /// Each region's anchor: the first star pinned to it.
    fn anchors(&self) -> Vec<usize> {
        (0..self.size)
            .map(|region| {
                let (cell, _) = self
                    .pinned
                    .iter()
                    .find(|&&(_, pinned)| pinned == region)
                    .expect("every region has a star pinned to it");
                cell.row * self.size + cell.col
            })
            .collect()
    }

    /// How many steps each cell is from its region's anchor in the hint, moving within the region.
    /// `None` for a cell the hint leaves cut off.
    fn hinted_distances(&self, anchors: &[usize]) -> Vec<Option<usize>> {
        let mut distance = vec![None; self.size * self.size];
        let mut queue: VecDeque<usize> = anchors.iter().copied().collect();
        anchors
            .iter()
            .for_each(|&anchor| distance[anchor] = Some(0));
        while let Some(index) = queue.pop_front() {
            for n in neighbours(index, self.size) {
                if distance[n].is_none() && self.hint[n] == self.hint[index] {
                    distance[n] = distance[index].map(|d| d + 1);
                    queue.push_back(n);
                }
            }
        }
        distance
    }
}

impl OraclePuzzle for RegionDesign {
    type Design = Vec<usize>;
    type Solution = Vec<Cell>;

    fn clauses(&self, regions: &Vec<usize>) -> Cnf {
        region_clauses(self.size, self.stars, regions)
    }

    fn solution_vars(&self) -> Vec<Var> {
        cell_vars(self.size)
    }

    fn read_solution(&self, model: &[bool]) -> Vec<Cell> {
        read_stars(self.size, model)
    }
}

impl Synthesis for RegionDesign {
    /// Every cell is in exactly one region, each intended star in its pinned one, and every
    /// region in one piece.
    fn define(&self, proposer: &mut Proposer) {
        for _ in 0..self.size * self.size * self.size {
            proposer.new_var();
        }
        for index in 0..self.size * self.size {
            let choices: Vec<Lit> = (0..self.size)
                .map(|region| self.in_region(self.cell(index), region))
                .collect();
            proposer.add_clauses(exactly(&choices, 1));
        }
        for &(cell, region) in &self.pinned {
            proposer.add_clause(&[self.in_region(cell, region)]);
        }
        self.connect(proposer);
        for (index, &region) in self.hint.iter().enumerate() {
            proposer.suggest(self.in_region(self.cell(index), region));
        }
    }

    fn read_design(&self, proposer: &Proposer) -> Vec<usize> {
        (0..self.size * self.size)
            .map(|index| {
                (0..self.size)
                    .find(|&region| proposer.value(self.in_region(self.cell(index), region)))
                    .expect("every cell has a region")
            })
            .collect()
    }

    /// For each piece of a region cut off from its first star: the piece can only stay in the
    /// region if one of its neighbours joins too.
    fn cuts(&self, regions: &Vec<usize>) -> Vec<Vec<Lit>> {
        let mut cuts = Vec::new();
        for region in 0..self.size {
            let (anchor, _) = *self
                .pinned
                .iter()
                .find(|&&(_, pinned)| pinned == region)
                .expect("every region has a star pinned to it");
            let anchor = anchor.row * self.size + anchor.col;
            for piece in pieces(regions, self.size, region) {
                if piece.contains(&anchor) {
                    continue;
                }
                let mut cut: Vec<Lit> = piece
                    .iter()
                    .map(|&index| !self.in_region(self.cell(index), region))
                    .collect();
                for index in piece.iter().flat_map(|&index| neighbours(index, self.size)) {
                    let joins = self.in_region(self.cell(index), region);
                    if !piece.contains(&index) && !cut.contains(&joins) {
                        cut.push(joins);
                    }
                }
                cuts.push(cut);
            }
        }
        cuts
    }

    /// Every solution has the same number of stars, so one with all the intended stars is it.
    fn intended(&self) -> Vec<Lit> {
        self.intended
            .iter()
            .map(|cell| Var::new((cell.row * self.size + cell.col) as u32).positive())
            .collect()
    }

    /// Some region must hold other than `stars` of `other`'s stars. Stars shared with the
    /// intended solution have fixed regions, so only the rest need counting.
    fn rule_out(&self, other: &Vec<Cell>, proposer: &mut Proposer) {
        let free: Vec<Cell> = other
            .iter()
            .copied()
            .filter(|cell| !self.pinned.iter().any(|(pinned, _)| pinned == cell))
            .collect();
        let mut breaks_some_region = Vec::new();
        for region in 0..self.size {
            let fixed = other
                .iter()
                .filter(|cell| self.pinned.contains(&(**cell, region)))
                .count();
            let needed = self.stars as isize - fixed as isize;
            let here: Vec<Lit> = free
                .iter()
                .map(|&cell| self.in_region(cell, region))
                .collect();
            // Too few of the free stars in this region, or too many.
            if needed > 0 {
                breaks_some_region.push(proposer.guard(at_most(&here, needed as usize - 1)));
            }
            if needed < here.len() as isize {
                breaks_some_region
                    .push(proposer.guard(at_least(&here, (needed + 1).max(0) as usize)));
            }
        }
        proposer.add_clause(&breaks_some_region);
    }
}

/// A random placement of stars obeying the row, column and touching rules, ignoring regions.
fn random_solution(size: usize, stars: usize, rng: &mut Rng) -> Vec<Cell> {
    let mut solver = Solver::new(Config::default());
    let var = |row: usize, col: usize| Var::new((row * size + col) as u32).positive();
    for _ in 0..size * size {
        solver.new_var();
    }
    for line in 0..size {
        let row: Vec<Lit> = (0..size).map(|col| var(line, col)).collect();
        let col: Vec<Lit> = (0..size).map(|row| var(row, line)).collect();
        exactly(&row, stars)
            .iter()
            .chain(&exactly(&col, stars))
            .for_each(|clause| solver.add_clause(clause));
    }
    for index in 0..size * size {
        let (row, col) = (index / size, index % size);
        for (dr, dc) in [(0, 1), (1, -1), (1, 0), (1, 1)] {
            let (r, c) = (row as isize + dr, col as isize + dc);
            if r < size as isize && (0..size as isize).contains(&c) {
                solver.add_clause(&[!var(row, col), !var(r as usize, c as usize)]);
            }
        }
    }

    // Try cells in a random order, keeping each star that still leaves a full placement possible.
    let mut order: Vec<usize> = (0..size * size).collect();
    for last in (1..order.len()).rev() {
        order.swap(last, rng.below(last as u32 + 1) as usize);
    }
    let mut chosen: Vec<Lit> = Vec::new();
    for index in order {
        if chosen.len() == size * stars {
            break;
        }
        chosen.push(var(index / size, index % size));
        if solver.solve_assuming(&chosen, Budget::unlimited()) != Status::Sat {
            chosen.pop();
        }
    }
    chosen
        .iter()
        .map(|lit| Cell {
            row: lit.var().index() / size,
            col: lit.var().index() % size,
        })
        .collect()
}

/// Grows a region around each group of stars: first joining each group's stars by a shortest path
/// through unclaimed cells, then spreading outward at random until the grid is covered. Every
/// star is claimed from the start, so no region takes another's. `None` if some group can't be
/// joined, because other groups' paths are in the way.
fn grow_connected(groups: &[Vec<Cell>], size: usize, rng: &mut Rng) -> Option<Vec<usize>> {
    const UNCLAIMED: usize = usize::MAX;
    let mut regions = vec![UNCLAIMED; size * size];
    for (region, group) in groups.iter().enumerate() {
        for cell in group {
            regions[cell.row * size + cell.col] = region;
        }
    }
    for (region, group) in groups.iter().enumerate() {
        let first = group[0].row * size + group[0].col;
        for cell in &group[1..] {
            let path = shortest_path(&regions, size, first, cell.row * size + cell.col, region)?;
            path.into_iter().for_each(|index| regions[index] = region);
        }
    }

    let mut frontier: Vec<usize> = (0..regions.len())
        .filter(|&index| regions[index] != UNCLAIMED)
        .collect();
    while !frontier.is_empty() {
        let pick = rng.below(frontier.len() as u32) as usize;
        let index = frontier[pick];
        let open: Vec<usize> = neighbours(index, size)
            .filter(|&n| regions[n] == UNCLAIMED)
            .collect();
        if open.is_empty() {
            frontier.swap_remove(pick);
        } else {
            let claimed = open[rng.below(open.len() as u32) as usize];
            regions[claimed] = regions[index];
            frontier.push(claimed);
        }
    }
    Some(regions)
}

/// The cells on a shortest path from `from` to `to`, moving only through unclaimed cells and
/// cells already in `region`.
fn shortest_path(
    regions: &[usize],
    size: usize,
    from: usize,
    to: usize,
    region: usize,
) -> Option<Vec<usize>> {
    let mut previous = vec![None; regions.len()];
    let mut queue = VecDeque::from([from]);
    previous[from] = Some(from);
    while let Some(index) = queue.pop_front() {
        if index == to {
            let mut path = vec![to];
            let mut at = to;
            while at != from {
                at = previous[at].expect("every cell on the path was reached");
                path.push(at);
            }
            return Some(path);
        }
        for n in neighbours(index, size) {
            let passable = regions[n] == usize::MAX || regions[n] == region;
            if passable && previous[n].is_none() {
                previous[n] = Some(index);
                queue.push_back(n);
            }
        }
    }
    None
}

/// Grows `size` regions from random single cells, one cell at a time: always connected.
fn random_regions(size: usize, rng: &mut Rng) -> Vec<usize> {
    const UNCLAIMED: usize = usize::MAX;
    let mut regions = vec![UNCLAIMED; size * size];
    let mut frontier = Vec::new();
    for region in 0..size {
        loop {
            let index = rng.below((size * size) as u32) as usize;
            if regions[index] == UNCLAIMED {
                regions[index] = region;
                frontier.push(index);
                break;
            }
        }
    }
    while !frontier.is_empty() {
        let pick = rng.below(frontier.len() as u32) as usize;
        let index = frontier[pick];
        let open: Vec<usize> = neighbours(index, size)
            .filter(|&n| regions[n] == UNCLAIMED)
            .collect();
        if open.is_empty() {
            frontier.swap_remove(pick);
        } else {
            let claimed = open[rng.below(open.len() as u32) as usize];
            regions[claimed] = regions[index];
            frontier.push(claimed);
        }
    }
    regions
}

/// Splits the stars into groups of `stars`, each a random star and its nearest ungrouped ones.
fn group_nearby(cells: &[Cell], stars: usize, rng: &mut Rng) -> Vec<Vec<Cell>> {
    let mut left = cells.to_vec();
    let mut groups = Vec::new();
    while !left.is_empty() {
        let first = left.swap_remove(rng.below(left.len() as u32) as usize);
        let mut group = vec![first];
        while group.len() < stars {
            let distance =
                |cell: &Cell| first.row.abs_diff(cell.row) + first.col.abs_diff(cell.col);
            let nearest = (0..left.len())
                .min_by_key(|&index| distance(&left[index]))
                .expect("stars come in whole groups");
            group.push(left.swap_remove(nearest));
        }
        groups.push(group);
    }
    groups
}

/// The connected pieces of one region, each as a list of cell indices.
fn pieces(regions: &[usize], size: usize, region: usize) -> Vec<Vec<usize>> {
    let mut seen = vec![false; regions.len()];
    let mut found = Vec::new();
    for start in 0..regions.len() {
        if regions[start] != region || seen[start] {
            continue;
        }
        let mut piece = Vec::new();
        let mut stack = vec![start];
        seen[start] = true;
        while let Some(index) = stack.pop() {
            piece.push(index);
            for neighbour in neighbours(index, size) {
                if regions[neighbour] == region && !seen[neighbour] {
                    seen[neighbour] = true;
                    stack.push(neighbour);
                }
            }
        }
        found.push(piece);
    }
    found
}

fn neighbours(index: usize, size: usize) -> impl Iterator<Item = usize> {
    let (row, col) = (index / size, index % size);
    [
        (row > 0).then(|| index - size),
        (row + 1 < size).then(|| index + size),
        (col > 0).then(|| index - 1),
        (col + 1 < size).then(|| index + 1),
    ]
    .into_iter()
    .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Board, Mark, Placed};
    use crate::session::Session;

    const STRATEGIES: [Strategy; 3] =
        [Strategy::Retry, Strategy::AroundStars, Strategy::Synthesize];

    #[test]
    fn every_strategy_makes_puzzles_with_one_solution_and_connected_regions() {
        for strategy in STRATEGIES {
            for seed in 0..4 {
                let (puzzle, _) = generate(8, 1, seed, strategy);
                let regions: Vec<usize> =
                    puzzle.cells().map(|cell| puzzle.region_of(cell)).collect();
                assert!(
                    (0..8).all(|region| pieces(&regions, 8, region).len() == 1),
                    "{strategy:?} seed {seed}"
                );
                let found = Session::new(puzzle).solutions(&Board::new(), 2);
                assert_eq!(found.found.len(), 1, "{strategy:?} seed {seed}");
                assert!(found.complete);
            }
        }
    }

    #[test]
    fn an_attempt_is_unique_exactly_when_it_finds_no_other_solution() {
        // A board with exactly these stars, and every other cell ruled out.
        let only = |stars: &[Cell]| {
            let mut board = Board::new();
            for row in 0..8 {
                for col in 0..8 {
                    let cell = Cell { row, col };
                    let mark = if stars.contains(&cell) {
                        Mark::Star
                    } else {
                        Mark::Empty
                    };
                    board.place(Placed { cell, mark });
                }
            }
            board
        };
        let mut attempts = Attempts::new(8, 1, 3);
        for _ in 0..30 {
            let attempt = attempts.next_attempt();
            let mut session = Session::new(Puzzle::new(8, 1, attempt.regions.clone()).unwrap());
            assert!(session.is_solvable(&only(&attempt.stars)));
            let unique = session.solutions(&Board::new(), 2).found.len() == 1;
            assert_eq!(attempt.other.is_none(), unique);
            if let Some(other) = &attempt.other {
                assert_ne!(other, &attempt.stars);
                assert!(session.is_solvable(&only(other)));
            }
        }
    }

    #[test]
    fn grown_regions_keep_each_group_together_and_apart_from_the_others() {
        let mut rng = Rng::new(5);
        let intended = random_solution(10, 2, &mut rng);
        let groups = group_nearby(&intended, 2, &mut rng);
        let regions = (0..50)
            .find_map(|_| grow_connected(&groups, 10, &mut rng))
            .expect("growth succeeds sometimes");
        for (region, group) in groups.iter().enumerate() {
            assert!(
                group
                    .iter()
                    .all(|cell| regions[cell.row * 10 + cell.col] == region)
            );
            assert_eq!(pieces(&regions, 10, region).len(), 1);
        }
    }

    #[test]
    fn the_same_seed_gives_the_same_puzzle() {
        for strategy in STRATEGIES {
            assert_eq!(generate(8, 1, 7, strategy).0, generate(8, 1, 7, strategy).0);
        }
    }

    #[test]
    fn random_solutions_obey_rows_columns_and_touching() {
        let mut rng = Rng::new(9);
        let stars = random_solution(10, 2, &mut rng);
        assert_eq!(stars.len(), 20);
        for line in 0..10 {
            assert_eq!(stars.iter().filter(|cell| cell.row == line).count(), 2);
            assert_eq!(stars.iter().filter(|cell| cell.col == line).count(), 2);
        }
        for a in &stars {
            for b in &stars {
                assert!(!(a != b && a.row.abs_diff(b.row) <= 1 && a.col.abs_diff(b.col) <= 1));
            }
        }
    }
}
