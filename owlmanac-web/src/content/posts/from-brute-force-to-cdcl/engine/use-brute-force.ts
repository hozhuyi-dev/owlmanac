import { proxy } from "comlink";
import { ref, shallowRef } from "vue";
import { satLab } from "./client";
import { parseClues, type SudokuPuzzle } from "./puzzles";
import type { RungResult } from "./types";

const KEY = "brute-force";
const BRUTE_FORCE = 0;

/** Brute force on one puzzle, until it finishes, hits `limitMs`, or is stopped. */
export function useBruteForce(puzzle: SudokuPuzzle, limitMs: number) {
  const result = shallowRef<RungResult>();
  const running = ref(false);
  const error = shallowRef<unknown>();
  /** Wall-clock time since Start, which is what a reader watching expects. `result.ms` counts only
   * the solver's own time, leaving out the pauses between slices, so it runs slow against a clock. */
  const elapsedMs = ref(0);
  // A worker message can still arrive after Stop, so each run has an id.
  let activeRun = 0;

  async function start() {
    const id = ++activeRun;
    const startedAt = performance.now();
    running.value = true;
    error.value = undefined;
    elapsedMs.value = 0;
    const show = (next: RungResult) => {
      if (id !== activeRun) return;
      result.value = next;
      elapsedMs.value = performance.now() - startedAt;
    };
    const clues = parseClues(puzzle.clues);
    try {
      show(await satLab().run(KEY, puzzle.boxSize, clues, BRUTE_FORCE, puzzle.extended, limitMs, proxy(show)));
    } catch (failure) {
      if (id === activeRun) error.value = failure;
    } finally {
      if (id === activeRun) running.value = false;
    }
  }

  function stop() {
    activeRun++;
    running.value = false;
    satLab().cancel(KEY);
  }

  return { result, running, error, elapsedMs, start, stop };
}
