# sat-oracle

Making puzzles with exactly one solution, using SAT solvers as the oracle. Generic over the puzzle: a puzzle describes itself through the traits in [`problem.rs`](src/problem.rs) and never names anything puzzle-specific to this crate. `sudoku` and `star-battle` both use it.

## One question, three strategies

Every strategy rests on the same question: does the puzzle this design makes have exactly one solution? The checker finds a solution, blocks it, and asks again; "no more" is a proof.

| Strategy | How | Needs | Used by |
| --- | --- | --- | --- |
| [`retry`](src/retry.rs) | Draw a design at random, keep it if unique | `RandomDesign` | Star Battle |
| [`dig`](src/dig.rs) | Show every clue, then take clues away while the answer stays unique | `Clues` | Sudoku |
| [`Synthesizer`](src/synthesis.rs) | One solver proposes designs, another finds second solutions, and the proposer learns from each | `Synthesis` | Star Battle |

**Dig** works for any puzzle whose design is a set of clues, each of which can only rule solutions out. Taking a clue away can only let solutions in, so once a clue has to stay it always will, and the result needs every clue it shows.

**Synthesis** is counterexample-guided synthesis (CEGIS), for designs that aren't clue sets, like Star Battle's regions. It's clause learning one level up: CDCL learns a clause from each dead end, and the proposer learns a rule from each second solution, so it never proposes the same mistake twice.

## Layout

```
src/
  lib.rs           public API only
  problem.rs       Puzzle (shared by every strategy), and RandomDesign, Clues, Synthesis
  retry.rs         generate and test
  dig.rs           remove clues while unique
  synthesizer.rs   the Synthesizer struct and run(rounds)
  synthesis.rs     its loop: start reading here
  data/
    checker.rs     the second solver: one solution after another, and is_unique
    proposer.rs    the synthesizer's solver, as a puzzle sees it
    outcome.rs     Outcome (Unique, Impossible, OutOfRounds) and Stats
  steps/           one stage of the synthesis loop per file
    propose.rs  refine.rs  check.rs  learn.rs
tests/reveal.rs    every strategy on a toy problem with no puzzle in it
```

## Decisions

- **One shared trait, one per strategy.** `Puzzle` is what checking needs (a design's clauses, and what a solution is); each strategy asks only for what it uses.
- **The intended solution comes first** for dig and synthesis, and the puzzle makes it valid. The checker then only has to block that one solution to look for any other.
- **Dig keeps one solver for every check,** each clue behind a guard assumed true while it's shown, so what one check learns carries to the next.
- **Synthesis can add rules lazily,** through `cuts`, but only for rules rarely broken. Star Battle's connected regions had to be stated up front: there are so many ways to draw broken regions that lazy cuts never caught up.
- **`run` takes a round limit and can be called again,** like `sat-core`'s budget, so a page could run it in slices.
