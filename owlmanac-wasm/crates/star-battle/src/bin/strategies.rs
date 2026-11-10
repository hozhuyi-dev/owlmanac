//! Compares the three ways of making a puzzle, for "Making Puzzles with a SAT Oracle". Makes the
//! same seeds with each strategy on every core, grades each puzzle, and writes one row per puzzle
//! to `strategies.json`. Run with:
//!
//! `cargo run --release -p star-battle --bin strategies -- <output folder> [puzzles per strategy]`
//!
//! Which puzzles come out, and their grades, are the same on every run; the times depend on the
//! machine, and on how busy it is.

use star_battle::{Board, Session, Strategy, generate};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use std::{env, fs, process};

const SIZE: usize = 10;
const STARS: usize = 2;
const DEFAULT_PER_STRATEGY: u64 = 200;

struct Row {
    strategy: &'static str,
    seed: u64,
    ms: f64,
    designs: u32,
    depth: Option<u32>,
    deepest_steps: u32,
    /// Each cell's region, row by row, one digit per cell.
    regions: String,
}

fn name(strategy: Strategy) -> &'static str {
    match strategy {
        Strategy::Retry => "retry",
        Strategy::AroundStars => "aroundStars",
        Strategy::Synthesize => "synthesize",
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some(folder) = args.first() else {
        eprintln!("usage: strategies <output folder> [puzzles per strategy]");
        process::exit(1);
    };
    let per_strategy = args.get(1).map_or(DEFAULT_PER_STRATEGY, |count| {
        count.parse().expect("a number")
    });
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());

    let mut rows = Vec::new();
    for strategy in [Strategy::Retry, Strategy::AroundStars, Strategy::Synthesize] {
        let started = Instant::now();
        let mut made = make(strategy, per_strategy, threads);
        made.sort_by_key(|row| row.seed);
        eprintln!(
            "{}: {} puzzles in {:.0} s",
            name(strategy),
            made.len(),
            started.elapsed().as_secs_f64()
        );
        rows.extend(made);
    }
    write(folder, threads, &rows);
}

fn make(strategy: Strategy, count: u64, threads: usize) -> Vec<Row> {
    let next = AtomicU64::new(0);
    let rows = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                loop {
                    let seed = next.fetch_add(1, Ordering::Relaxed);
                    if seed >= count {
                        return;
                    }
                    let started = Instant::now();
                    let (puzzle, generation) = generate(SIZE, STARS, seed, strategy);
                    let ms = started.elapsed().as_secs_f64() * 1000.0;
                    let regions: String = puzzle
                        .cells()
                        .map(|cell| {
                            char::from_digit(puzzle.region_of(cell) as u32, 36)
                                .expect("fewer than 36 regions")
                        })
                        .collect();
                    let grade = Session::new(puzzle)
                        .grade(&Board::new())
                        .expect("generated puzzles have a solution");
                    rows.lock().unwrap().push(Row {
                        strategy: name(strategy),
                        seed,
                        ms,
                        designs: generation.oracle.rounds,
                        depth: grade.depth,
                        deepest_steps: grade.deepest_steps,
                        regions,
                    });
                }
            });
        }
    });
    rows.into_inner().unwrap()
}

fn write(folder: &str, threads: usize, rows: &[Row]) {
    let lines: Vec<String> = rows
        .iter()
        .map(|row| {
            let depth = row.depth.map_or("null".to_string(), |depth| depth.to_string());
            format!(
                "{{\"strategy\":\"{}\",\"seed\":{},\"ms\":{:.0},\"designs\":{},\"depth\":{depth},\"deepestSteps\":{},\"regions\":\"{}\"}}",
                row.strategy, row.seed, row.ms, row.designs, row.deepest_steps, row.regions
            )
        })
        .collect();
    let json = format!(
        "{{\"size\":{SIZE},\"stars\":{STARS},\"threads\":{threads},\"puzzles\":[\n{}\n]}}\n",
        lines.join(",\n")
    );
    fs::create_dir_all(folder).unwrap_or_else(|error| panic!("{folder}: {error}"));
    let path = format!("{folder}/strategies.json");
    fs::write(&path, json).unwrap_or_else(|error| panic!("{path}: {error}"));
    eprintln!("wrote {} rows to {path}", rows.len());
}
