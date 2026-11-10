# owlmanac-wasm

The heavy computations behind Owlmanac's posts, in Rust. Each crate keeps its design notes in its own README.

## Crates

| Crate | Speaks | Knows nothing about |
| --- | --- | --- |
| [`sat-core`](crates/sat-core/README.md): the CDCL solver | `Var`, `Lit`, clauses | puzzles, WASM |
| [`sat-oracle`](crates/sat-oracle/README.md): designing puzzles with one solution, two solvers in a loop | `Problem`, designs, counterexamples | any particular puzzle, WASM |
| [`sudoku`](crates/sudoku/README.md), [`star-battle`](crates/star-battle/README.md): encoders, analyses, generators | `Cell`, `Rule`, `Board`, never `Lit` in the public API | WASM, JS |
| `sat-cli`: answers and proofs for outside checkers, native only | files | WASM |
| `sat-lab-wasm`, `star-battle-wasm`: bindings, one per post | JS values, typed arrays | algorithms: a method only converts and calls |

## Rules

- One vocabulary per layer, as in the table.
- A WASM crate per post, so a page downloads only what it uses.
- Modules use `data.rs` beside `data/`, not `data/mod.rs`.
- Bindings return typed arrays for bulk data (event streams, grid overlays) and plain JS objects through `serde-wasm-bindgen` for small results (an explanation, a verdict).
- Single-threaded, in a Web Worker: threads need COOP/COEP headers, which GitHub Pages can't send. A running WASM call can't be interrupted, so anything long runs in budgeted slices and yields between them.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets
cargo run --release -p sat-core --example ladder   # what each CDCL idea adds
uv run scripts/check_external.py <drat-trim path>  # answers against CaDiCaL, proofs with drat-trim (needs a release sat-cli)
```

On Windows, compile `drat-trim` with `gcc -O2 -Dgetc_unlocked=getc -o drat-trim.exe drat-trim.c`: MinGW has no `getc_unlocked`.
