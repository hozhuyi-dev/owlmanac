import { computed, onMounted, ref, shallowRef } from "vue";
import { useTask } from "@/core/async/use-task";
import type { Blame, Hint, Marks, Placed } from "@/wasm/star-battle.types";
import { starBattle } from "./client";
import { depthText, explain, markText, type Explanation } from "./explain";
import { nextPuzzle, type Level, type PoolPuzzle } from "./pool";

const SESSION = "game";
const NEXT_MARK: Record<number, number> = { 0: 1, 1: -1, [-1]: 0 };
export const SIZE = 10;
export const STARS = 2;

export type Verdict = { solvable: boolean; blame?: Blame };

/** An answer to a question, kept with the board it was about, so it's shown only for that board. */
type Answer = { board: Marks; explanation: Explanation; hint?: Hint };

/**
 * One game: a puzzle from the pool, the player's marks, and the solver's answers about them. The
 * marks live here, so a click shows at once; answers arrive afterwards, only for the latest board.
 */
export function useGame() {
  const level = ref<Level>("easy");
  const puzzle = shallowRef<PoolPuzzle>();
  const marks = shallowRef<Marks>(new Int8Array(SIZE * SIZE));

  // Every answer is about the session's current puzzle; opening a new one replaces it.
  let opened: Promise<void> = Promise.resolve();

  const verdict = useTask(async (board: Marks): Promise<Verdict> => {
    await opened;
    const solvable = await starBattle().isSolvable(SESSION, board);
    return { solvable, blame: solvable ? undefined : await starBattle().blame(SESSION, board) };
  });

  const loading = useTask(async (chosen: Level) => {
    const next = await nextPuzzle(chosen);
    opened = starBattle().open(SESSION, next);
    await opened;
    puzzle.value = next;
    setMarks(new Int8Array(SIZE * SIZE));
  });

  const question = useTask(async (ask: () => Promise<Answer>) => ask());
  const answer = computed(() => (question.result.value?.board === marks.value ? question.result.value : undefined));

  function setMarks(next: Marks) {
    marks.value = next;
    verdict.start(next);
  }

  function cycleMark(index: number) {
    const next = marks.value.slice();
    next[index] = NEXT_MARK[next[index]];
    setMarks(next);
  }

  function newPuzzle(chosen = level.value) {
    level.value = chosen;
    loading.start(chosen);
  }

  /** Why can't a star go on this cell? */
  function askWhyNot(index: number) {
    const board = marks.value;
    const proposal: Placed = { cell: { row: Math.floor(index / SIZE), col: index % SIZE }, mark: "star" };
    question.start(async () => {
      const why = await starBattle().whyNot(SESSION, board, proposal.cell.row, proposal.cell.col, true);
      return { board, explanation: explain(why, proposal) };
    });
  }

  /** The easiest cell to work out next, explained by asking why its opposite can't be. */
  function askHint() {
    const board = marks.value;
    question.start(async () => {
      const hint = await starBattle().hint(SESSION, board);
      if (!hint) return { board, explanation: { lines: ["No hint: the board is finished, or a mark is wrong."], cells: [] } };
      const opposite: Placed = { cell: hint.placed.cell, mark: hint.placed.mark === "star" ? "empty" : "star" };
      const why = await starBattle().whyNot(SESSION, board, opposite.cell.row, opposite.cell.col, opposite.mark === "star");
      const reasoning = explain(why, opposite);
      const lines = [`Try ${markText(hint.placed)}. ${depthText(hint.depth)}`];
      if (why.kind === "deduced") lines.push(...reasoning.lines);
      return { board, hint, explanation: { lines, cells: [hint.placed.cell, ...reasoning.cells] } };
    });
  }

  const stars = computed(() => marks.value.filter((mark) => mark === 1).length);
  const solved = computed(
    () => stars.value === SIZE * STARS && !verdict.pending.value && verdict.result.value?.solvable === true,
  );

  onMounted(() => newPuzzle("easy"));

  return {
    level,
    puzzle,
    marks,
    verdict: verdict.result,
    checking: verdict.pending,
    answer,
    thinking: question.pending,
    solved,
    loading: loading.pending,
    loadingError: loading.error,
    solverError: computed(() => verdict.error.value ?? question.error.value),
    cycleMark,
    clear: () => setMarks(new Int8Array(SIZE * SIZE)),
    newPuzzle,
    askWhyNot,
    askHint,
  };
}
