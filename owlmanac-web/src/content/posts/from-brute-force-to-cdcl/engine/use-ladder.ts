import { proxy } from "comlink";
import { ref, shallowRef } from "vue";
import { useTask } from "@/core/async/use-task";
import { satLab } from "./client";
import { parseClues, type SudokuPuzzle } from "./puzzles";
import type { RungResult } from "./types";

const KEY = "ladder";

/** Mirrors `sudoku::rung_names`, so the table can be drawn before the WASM arrives. */
export const RUNG_NAMES = [
  "Brute force",
  "DPLL",
  "Clause learning",
  "+ VSIDS",
  "+ phase saving",
  "+ restarts",
  "+ clause deletion",
];

const WAITING: RungResult = { outcome: "waiting", ms: 0, guesses: 0, deadEnds: 0, propagations: 0 };
const waitingRows = () => RUNG_NAMES.map(() => WAITING);

/** Races every rung of the ladder on one puzzle, one after another. */
export function useLadder(limitMs: number) {
  const results = ref<RungResult[]>(waitingRows());
  const current = shallowRef<number>();
  // A worker message can still be on its way after a race is abandoned, so each race has an id.
  let activeRace = 0;

  const race = useTask(async (puzzle: SudokuPuzzle) => {
    const id = ++activeRace;
    const clues = parseClues(puzzle.clues);
    for (let rung = 0; rung < RUNG_NAMES.length; rung++) {
      current.value = rung;
      const show = (result: RungResult) => {
        if (id === activeRace) results.value[rung] = result;
      };
      show(await satLab().run(KEY, puzzle.boxSize, clues, rung, puzzle.extended, limitMs, proxy(show)));
      if (id !== activeRace) return;
    }
    current.value = undefined;
  });

  function start(puzzle: SudokuPuzzle) {
    results.value = waitingRows();
    race.start(puzzle);
  }

  function reset() {
    activeRace++;
    current.value = undefined;
    satLab().cancel(KEY);
    results.value = waitingRows();
  }

  return { results, current, error: race.error, start, reset };
}
