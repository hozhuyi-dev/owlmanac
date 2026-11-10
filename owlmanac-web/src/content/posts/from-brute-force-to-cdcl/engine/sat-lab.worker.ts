import { expose } from "comlink";
import init, { FormulaRun, SudokuRun } from "../wasm/sat_lab_wasm.js";
import type { Outcome, RungResult, Work } from "./types";

const ready = init();
const nextTurn = () => new Promise((resolve) => setTimeout(resolve));

// Each figure runs under its own key. Starting a run under a key, or cancelling the key, makes the
// previous run under it stop at its next slice, while other figures carry on.
const generation = new Map<string, number>();
const bump = (key: string) => {
  const next = (generation.get(key) ?? 0) + 1;
  generation.set(key, next);
  return next;
};

// The rung index of the full solver within the SAT ladder (DPLL is 0).
const FULL_SOLVER = 5;
const CONFLICTS_PER_SLICE = 100_000;
// Progress goes out about 20 times a second, however short the slices.
const PROGRESS_EVERY_MS = 50;

const api = {
  cancel(key: string) {
    bump(key);
  },

  /** Runs one rung on a Sudoku in slices, yielding between them so `cancel` can get through. */
  async run(
    key: string,
    boxSize: number,
    clues: Uint8Array<ArrayBuffer>,
    rung: number,
    extended: boolean,
    limitMs: number,
    onProgress: (result: RungResult) => void,
  ): Promise<RungResult> {
    await ready;
    const id = bump(key);
    let start = performance.now();
    const run = new SudokuRun(boxSize, clues, rung, extended);
    let ms = performance.now() - start;
    const result = (outcome: Outcome): RungResult => ({
      ...(run.report() as Work),
      outcome,
      ms,
      grid: run.currentGrid(),
    });
    let lastProgress = 0;

    try {
      for (;;) {
        start = performance.now();
        const outcome = run.advance() as Outcome;
        ms += performance.now() - start;
        if (outcome !== "running") return result(outcome);
        if (ms >= limitMs) return result("gaveUp");

        if (performance.now() - lastProgress >= PROGRESS_EVERY_MS) {
          lastProgress = performance.now();
          onProgress(result("running"));
        }
        await nextTurn();
        if (generation.get(key) !== id) return result("cancelled");
      }
    } finally {
      run.free();
    }
  },

  /** Solves one random 3-SAT formula with the full solver. */
  async solveRandom(numVars: number, numClauses: number, seed: number) {
    await ready;
    const run = FormulaRun.random(numVars, numClauses, 3, BigInt(seed), FULL_SOLVER);
    try {
      let outcome = run.advance(BigInt(CONFLICTS_PER_SLICE));
      while (outcome === "running") outcome = run.advance(BigInt(CONFLICTS_PER_SLICE));
      return { satisfiable: outcome === "solved", conflicts: (run.report() as Work).deadEnds };
    } finally {
      run.free();
    }
  },
};

export type SatLabApi = typeof api;

expose(api);
