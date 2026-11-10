# sudoku

Sudoku of any box size, solved by brute force and by SAT so the two can be compared. Start reading at [`src/ladder.rs`](src/ladder.rs).

## Layout

```
src/
  grid.rs         Grid, parsing, validity
  brute_force.rs  the student's solver: fill cells in order, step back when stuck
  encode.rs       Grid to clauses, Minimal or Extended
  ladder.rs       one puzzle on one rung, advancing in slices
  generate.rs     reproducible puzzles with exactly one solution
tests/integration/   rungs against each other, the cursor search against a recursive one
```

## Decisions

- **Brute force is written with a cursor, not recursion,** so it can stop and resume between slices. A test checks it makes the same guesses and hits the same dead ends as the recursive form.
- **Two encodings.** Minimal has 8,829 clauses on a 9×9 and Extended 11,988. The extra clauses are logically redundant but let propagation find hidden singles.
- **`generate` is `sat-oracle`'s `dig`.** Sudoku only says what its rules and clues are: a solved grid with every cell a clue. Dig takes clues away while the answer stays unique, which ends with every remaining clue needed.

## What the ladder shows

Each idea needs the right puzzle to show up. Measured in the browser, conflicts rounded:

| Puzzle | Encoding | Shows |
| --- | --- | --- |
| 9×9 built against brute force | Extended | Brute force is still going after 100 million guesses. Propagation alone solves it, with no guesses. Every SAT rung is identical. |
| 16×16, as sparse as it gets | Minimal | DPLL passes 400,000 conflicts without finishing. Clause learning needs 717. VSIDS barely helps, and phase saving makes it worse. |
| 25×25, as sparse as it gets | Extended | DPLL and plain learning run out of time. VSIDS solves it in about 12,000 conflicts. |

Restarts and clause deletion show nothing on any of these: they need the random 3-SAT of `sat-core`'s `examples/ladder.rs`.
