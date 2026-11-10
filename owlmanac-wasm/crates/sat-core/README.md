# sat-core

A CDCL SAT solver with no knowledge of puzzles or the browser. Start reading at [`src/search.rs`](src/search.rs): the main loop, where each step it names is one file in `src/steps/`.

## Layout

```
src/
  lib.rs       public API only
  solver.rs    the Solver struct, Config, Budget, add_clause, solve_assuming, resume, model
  search.rs    the main loop
  probe.rs     propagate_assuming: what follows from some assumptions, and why, with no search
  observer.rs  the hooks a trace or proof attaches to
  proof.rs     DratProof, an observer
  data/        types that guard their own invariants, no search logic
    lit.rs        Var, Lit (2v + sign)
    maps.rs       VarMap, LitMap: arrays indexed only by that type
    clause_db.rs  every clause in one flat literal array, behind a header table
    watches.rs    the clauses watching each literal, with blockers
    trail.rs      the assignments in order: history and propagation queue in one
    var_order.rs  VSIDS scores in an indexed binary heap, saved phases
  steps/       one algorithm stage per file, each an `impl Solver` block
    propagate.rs  analyze.rs  learn.rs  decide.rs  restart.rs  reduce.rs
  baseline/dpll.rs   the no-learning search, for measuring what learning adds
  ladder.rs          the configurations that add one idea at a time
  cardinality.rs     at most k, at least k, exactly k, as plain clauses
  cnf.rs  dimacs.rs  random.rs
tests/integration/   one test target: agreement (every Config against enumeration), incremental
                     (reused solvers, assumptions, deductions), proof (a small independent checker)
examples/ladder.rs   what each idea adds, on random 3-SAT
```

## Invariants

- A clause's positions 0 and 1 are its watched literals.
- A clause that is a reason keeps its implied literal at position 0.
- A literal's watch list is visited when that literal becomes false.
- Literals from the trail's `head` onwards are assigned but not yet propagated.
- A conflict at the root means the formula is unsatisfiable. No other conflict does.
- Each assumption sits at its own decision level, even if that level is empty because it was already true.

## Decisions

- **`#![forbid(unsafe_code)]`.** If benchmarks ever justify removing bounds checks, the one exception goes in `maps.rs`.
- **A `ClauseRef` indexes a header table, not the literal array.** Safe Rust can't view a `Vec<u32>` as `&[Lit]`, so headers can't sit inline. It costs one indirection, softened by blockers, and buys stable refs: compaction rewrites only headers, so watchers and reasons need no remapping.
- **Deletion runs straight after a restart,** so the only reasons in use are for root-level facts. It needs restarts turned on.
- **`search` keeps no local state between iterations,** so calling `solve` again after `Unknown` resumes it. That's what lets a Web Worker run it in slices.
- **DPLL is a separate loop, not a switch.** Backjumping without a learnt clause would skip branches, so DPLL has to backtrack chronologically and flip decisions.
- **Learning and backjumping are one rung of the ladder,** since both come out of the same analysis.
- **Assumptions go through the normal search,** as the first decisions. What is learnt under them stays valid afterwards, so a page can ask hundreds of related questions of one solver.
- **Proofs and traces are observers,** so the search holds no proof code. DPLL learns nothing and writes no proof.
- **With VSIDS off nothing bumps a score,** and the heap's tie-break on variable index makes it a fixed order.
