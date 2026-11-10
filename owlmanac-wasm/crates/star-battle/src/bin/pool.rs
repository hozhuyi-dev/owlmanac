//! Makes the game's puzzle pool: 10×10 two-star puzzles with one solution, graded Easy or Hard,
//! no two alike, written as one JSON file per level. Run with:
//!
//! `cargo run --release -p star-battle --bin pool -- <puzzles per level> <output folder>`
//!
//! Seeds are handed out in fixed batches and the results sorted by seed, so the same arguments
//! always give the same files, however the threads happen to be scheduled.

use star_battle::{Board, Puzzle, Session, Strategy, generate};
use std::collections::BTreeSet;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{env, fs, process};

const SIZE: usize = 10;
const STARS: usize = 2;
const SEEDS_PER_BATCH: u64 = 512;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Level {
    Easy,
    Hard,
}

impl Level {
    fn name(self) -> &'static str {
        match self {
            Level::Easy => "easy",
            Level::Hard => "hard",
        }
    }

    /// Easy needs at most one "what if" at a time; Hard needs a trial inside a trial, or more.
    fn of(depth: Option<u32>) -> Level {
        match depth {
            Some(depth) if depth <= 1 => Level::Easy,
            _ => Level::Hard,
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let [per_level, folder] = args.as_slice() else {
        eprintln!("usage: pool <puzzles per level> <output folder>");
        process::exit(1);
    };
    let per_level: usize = per_level.parse().expect("puzzles per level is a number");

    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let mut made: Vec<(u64, Level, String)> = Vec::new();
    let mut first_seed = 0;
    loop {
        made.extend(make_batch(first_seed, threads));
        first_seed += SEEDS_PER_BATCH;
        made.sort();
        let pools = pick(&made, per_level);
        let counts: Vec<usize> = pools.iter().map(Vec::len).collect();
        eprintln!("seeds 0..{first_seed}: {counts:?} unique puzzles (easy, hard)");
        if counts.iter().all(|&count| count >= per_level) {
            for (level, puzzles) in [Level::Easy, Level::Hard].into_iter().zip(pools) {
                write(folder, level, &puzzles);
            }
            return;
        }
    }
}

/// Generates and grades one batch of seeds on every core.
fn make_batch(first_seed: u64, threads: usize) -> Vec<(u64, Level, String)> {
    let next = AtomicU64::new(first_seed);
    let made = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                loop {
                    let seed = next.fetch_add(1, Ordering::Relaxed);
                    if seed >= first_seed + SEEDS_PER_BATCH {
                        return;
                    }
                    let (puzzle, _) = generate(SIZE, STARS, seed, Strategy::AroundStars);
                    let grade = Session::new(puzzle.clone())
                        .grade(&Board::new())
                        .expect("generated puzzles have a solution");
                    made.lock()
                        .unwrap()
                        .push((seed, Level::of(grade.depth), canonical(&puzzle)));
                }
            });
        }
    });
    made.into_inner().unwrap()
}

/// The first `per_level` puzzles of each level in seed order, skipping any already seen.
fn pick(made: &[(u64, Level, String)], per_level: usize) -> [Vec<String>; 2] {
    let mut seen = BTreeSet::new();
    let mut pools = [Vec::new(), Vec::new()];
    for (_, level, regions) in made {
        let pool = &mut pools[*level as usize];
        if pool.len() < per_level && seen.insert(regions.clone()) {
            pool.push(regions.clone());
        }
    }
    pools
}

/// One string per puzzle that's the same for every rotation, reflection and renumbering of its
/// regions: each of the 8 orientations, regions renumbered by first appearance, smallest wins.
fn canonical(puzzle: &Puzzle) -> String {
    let at = |row: usize, col: usize| puzzle.region_of(star_battle::Cell { row, col });
    let last = SIZE - 1;
    let orientations: [&dyn Fn(usize, usize) -> usize; 8] = [
        &|r, c| at(r, c),
        &|r, c| at(c, last - r),
        &|r, c| at(last - r, last - c),
        &|r, c| at(last - c, r),
        &|r, c| at(r, last - c),
        &|r, c| at(last - r, c),
        &|r, c| at(c, r),
        &|r, c| at(last - c, last - r),
    ];
    orientations
        .iter()
        .map(|region_at| {
            let mut names: Vec<usize> = Vec::new();
            let mut text = String::with_capacity(SIZE * SIZE);
            for row in 0..SIZE {
                for col in 0..SIZE {
                    let region = region_at(row, col);
                    let name = names
                        .iter()
                        .position(|&seen| seen == region)
                        .unwrap_or_else(|| {
                            names.push(region);
                            names.len() - 1
                        });
                    text.push(char::from_digit(name as u32, 36).expect("fewer than 36 regions"));
                }
            }
            text
        })
        .min()
        .expect("there are 8 orientations")
}

fn write(folder: &str, level: Level, puzzles: &[String]) {
    let list: Vec<String> = puzzles
        .iter()
        .map(|regions| format!("\"{regions}\""))
        .collect();
    let json = format!(
        "{{\"size\":{SIZE},\"stars\":{STARS},\"level\":\"{}\",\"puzzles\":[\n{}\n]}}\n",
        level.name(),
        list.join(",\n")
    );
    let path = format!("{folder}/{}.json", level.name());
    fs::write(&path, json).unwrap_or_else(|error| panic!("{path}: {error}"));
    eprintln!("wrote {} puzzles to {path}", puzzles.len());
}
