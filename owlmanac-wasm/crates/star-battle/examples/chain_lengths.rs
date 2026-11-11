// How long are the chains in the pool's puzzles? Grades every puzzle in a pool folder and prints,
// for each depth, how many puzzles have each length. Usage: chain_lengths <folder with easy.json, medium.json, hard.json>
use star_battle::{Board, Puzzle, Session};
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::{env, fs};

fn main() {
    let folder = env::args().nth(1).expect("usage: chain_lengths <folder>");
    let mut puzzles = Vec::new();
    for level in ["easy", "medium", "hard"] {
        let text = fs::read_to_string(format!("{folder}/{level}.json")).unwrap();
        puzzles.extend(text.lines().filter_map(|line| {
            let regions = line
                .trim()
                .trim_end_matches(',')
                .strip_prefix('"')?
                .strip_suffix('"')?;
            Some(regions.to_string())
        }));
    }

    let counts: Mutex<BTreeMap<(Option<u32>, u32), usize>> = Mutex::new(BTreeMap::new());
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let chunk = puzzles.len().div_ceil(threads);
    let started = std::time::Instant::now();
    std::thread::scope(|scope| {
        for part in puzzles.chunks(chunk) {
            let counts = &counts;
            scope.spawn(move || {
                for regions in part {
                    let cells = regions
                        .chars()
                        .map(|c| c.to_digit(36).unwrap() as usize)
                        .collect();
                    let session = Session::new(Puzzle::new(10, 2, cells).unwrap());
                    let grade = session.grade(&Board::new()).unwrap();
                    *counts
                        .lock()
                        .unwrap()
                        .entry((grade.depth, grade.length))
                        .or_default() += 1;
                }
            });
        }
    });
    eprintln!(
        "graded {} puzzles in {:?}",
        puzzles.len(),
        started.elapsed()
    );
    for ((depth, length), count) in counts.into_inner().unwrap() {
        println!("depth {depth:?}  length {length:>2}  {count:>5}");
    }
}
