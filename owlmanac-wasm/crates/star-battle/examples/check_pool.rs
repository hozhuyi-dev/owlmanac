// Checks a pool written by the `pool` binary: every puzzle has one solution and connected regions,
// sits in the right level, and appears only once. Usage: check_pool <folder with easy.json, medium.json, hard.json>
use star_battle::{Board, Puzzle, Session};
use std::collections::BTreeSet;
use std::{env, fs};

fn connected(regions: &[usize], size: usize) -> bool {
    (0..size).all(|region| {
        let cells: Vec<usize> = (0..regions.len())
            .filter(|&i| regions[i] == region)
            .collect();
        let mut seen = vec![false; regions.len()];
        let mut stack = vec![cells[0]];
        seen[cells[0]] = true;
        let mut reached = 0;
        while let Some(i) = stack.pop() {
            reached += 1;
            let (r, c) = (i / size, i % size);
            let around = [
                (r > 0).then(|| i - size),
                (r + 1 < size).then(|| i + size),
                (c > 0).then(|| i - 1),
                (c + 1 < size).then(|| i + 1),
            ];
            for n in around.into_iter().flatten() {
                if regions[n] == region && !seen[n] {
                    seen[n] = true;
                    stack.push(n);
                }
            }
        }
        reached == cells.len()
    })
}

fn main() {
    let folder = env::args().nth(1).expect("usage: check_pool <folder>");
    let mut all = BTreeSet::new();
    for level in ["easy", "medium", "hard"] {
        let text = fs::read_to_string(format!("{folder}/{level}.json")).unwrap();
        let puzzles: Vec<&str> = text
            .lines()
            .filter_map(|line| {
                line.trim()
                    .trim_end_matches(',')
                    .strip_prefix('"')?
                    .strip_suffix('"')
            })
            .collect();
        let mut problems = 0;
        for regions in &puzzles {
            let cells: Vec<usize> = regions
                .chars()
                .map(|c| c.to_digit(36).unwrap() as usize)
                .collect();
            let puzzle = Puzzle::new(10, 2, cells.clone()).unwrap();
            let session = Session::new(puzzle);
            let found = session.solutions(&Board::new(), 2);
            let grade = session.grade(&Board::new()).unwrap();
            // As the pool binary grades: Easy chains clash within 2 rounds, Medium's run longer.
            let right_level = match grade.depth {
                Some(0 | 1) if grade.length <= 2 => level == "easy",
                Some(0 | 1) => level == "medium",
                _ => level == "hard",
            };
            if found.found.len() != 1
                || !found.complete
                || !connected(&cells, 10)
                || !right_level
                || !all.insert(regions.to_string())
            {
                problems += 1;
            }
        }
        println!("{level}: {} puzzles, {problems} problems", puzzles.len());
    }
}
