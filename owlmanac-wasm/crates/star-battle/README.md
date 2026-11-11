# star-battle

Star Battle on top of `sat-core`. The public API speaks of cells, marks and rules, never of literals. Start reading at [`src/session.rs`](src/session.rs); each thing a reader can ask is one file in `src/analysis/`.

## Layout

```
src/
  puzzle.rs     Puzzle, Cell, validation
  board.rs      Board (the player's marks), Mark, Placed
  rules.rs      Rule::{Row, Column, Region, NoTouching}
  encode.rs     Puzzle to clauses; Guarded, a solver that can name rules
  session.rs    one puzzle being played, with its solvers
  analysis/     check (can it be finished?), blame, why_not, forced, count, learned, difficulty
  regions.rs    making puzzles with sat-oracle, three ways (Strategy), and generate()
  bin/pool.rs   the game's puzzle pool: graded Easy, Medium or Hard, deduplicated, one JSON file per level
tests/integration/   analyses on puzzles with a known solution, agreement with brute force
```

## The puzzle pool

```sh
cargo run --release -p star-battle --bin pool -- 2000 ../owlmanac-web/src/content/posts/star-battle/data
cargo run --release -p star-battle --example check_pool -- ../owlmanac-web/src/content/posts/star-battle/data
cargo run --release -p star-battle --example chain_lengths -- ../owlmanac-web/src/content/posts/star-battle/data  # how many puzzles at each depth and length
```

10×10, two stars, made with `Strategy::AroundStars`. Easy needs at most one "what if" at a time (depth 1), each breaking a rule within 2 rounds of forced moves (length ≤ 2); Medium is depth 1 with a longer chain somewhere; Hard needs a trial inside a trial (depth 2). The cut at length 2 splits depth-1 puzzles about 60:40; the pool needed about 9,200 seeds for 2,000 of each, so Medium is the scarcest, at about 22% of what's made. Two puzzles count as the same if one is a rotation, reflection or renumbering of the other. Seeds go out in fixed batches and results are sorted by seed, so the same arguments always give the same files. The files are committed, like a post's parquet.

## Decisions

- **One variable per cell, no helper variables.** Every literal is then a cell, so every learnt clause can be drawn on the grid. The cost is clause count, fine at these sizes: a 10×10, two-star row is 130 clauses.
- **Three solvers per session.** `play` answers yes-or-no questions and keeps what it learns. `blame` has each rule behind a guard variable, assumed true per query, so a failed query names the rules involved. `deduce` is guarded too but only ever propagates: having never searched it has no learnt clauses, so every reason is one rule and can be read out.
- **Blame is minimal, not smallest.** Nothing in it can be dropped, but a smaller set may exist elsewhere. At least one blamed mark is wrong, which is all the page claims.
- **Rules are tried in a fixed order,** so a star beside another in the same row is reported as breaking the row rule, not the touching rule.
- **Difficulty is the reasoning a person needs, not solver effort.** `grade` measures depth, the deepest "what if" (0 for forced moves alone, 1 for a single trial, 2 for a trial within a trial), and length, how many rounds a trial's forced moves run before a rule breaks (round 1 is what the trial forces outright, round 2 what those force). A trial is played out in your head, so a long chain is hard even when each move is simple.
- **A grade is its hardest unavoidable step.** At each point `grade` takes the cheapest step on offer, forced moves, then the shortest depth-1 chain, then depth 2, and reports the worst of them, as Sudoku graders do. Taking the first trial that works instead would grade by cell order.
- **The grader plays out forced moves itself,** round by round over the clauses, rather than through `sat-core`. A solver's propagation order decides which reason each deduction gets, and so how long a chain looks; rounds don't depend on it. Over the 4,000 earlier pool puzzles it agrees with the old grader's depth on every one.
- **A hint is one step of the grade:** the easiest cell to work out next, forced moves first (the earliest round), then the "what if" that clashes soonest, then one inside another. Following hints alone solves every fixture, and on Hard pool puzzles each takes about 10 ms in the browser, under a second at worst.
- **Three ways to make a puzzle, measured on 10×10 two-star over 90 s on 16 threads:**

  | Strategy | Puzzles per minute | Depth 1 | Depth 2 |
  | --- | --- | --- | --- |
  | `Retry`: random regions, kept if unique | 58 | 35% | 65% |
  | `AroundStars`: stars first, regions grown around nearby pairs | 352 | 59% | 41% |
  | `Synthesize`: stars first, a solver draws the regions, learning from each second solution | 30 | 76% | 24% |

  Around stars wins because the intended solution is always valid, so a try can only fail by being ambiguous. Synthesis needs only about 7 proposals per puzzle, but drawing connected regions inside a solver is costly: connectivity had to be stated up front, as reachability from each region's first star, since lazy cuts never caught up with the ways to draw broken regions.
- **Repair was tried and dropped.** Moving a cell where two solutions differ was barely better than retrying, mostly because a move usually leaves no solution at all.
