import { expose } from "comlink";
import init, { Generator, StarBattle } from "@/wasm/star-battle/star_battle_wasm.js";
import type { Attempt, Grade, LearnedRule, Solutions } from "@/wasm/star-battle.types";

const ready = init();
const nextTurn = () => new Promise((resolve) => setTimeout(resolve));
// Progress goes out about 10 times a second, so a reader can follow the attempts.
const PROGRESS_EVERY_MS = 100;

// Each figure's long run has a key. Starting a run under a key, or cancelling the key, stops the
// previous run under it at its next attempt.
const generation = new Map<string, number>();
const bump = (key: string) => {
  const next = (generation.get(key) ?? 0) + 1;
  generation.set(key, next);
  return next;
};

/** Runs `use` on a puzzle made from these regions, or returns null if they don't make one. */
function withPuzzle<T>(size: number, stars: number, regions: number[], use: (puzzle: StarBattle) => T): T | null {
  let puzzle: StarBattle;
  try {
    puzzle = new StarBattle(size, stars, Uint8Array.from(regions));
  } catch {
    return null;
  }
  try {
    return use(puzzle);
  } finally {
    puzzle.free();
  }
}

const unmarked = (size: number) => new Int8Array(size * size);

const api = {
  /** Up to `limit` solutions, or null if a region has no cells. */
  async solutions(size: number, stars: number, regions: number[], limit: number): Promise<Solutions | null> {
    await ready;
    return withPuzzle(size, stars, regions, (puzzle) => puzzle.solutions(unmarked(size), limit));
  },

  async learnedRules(size: number, stars: number, regions: number[], limit: number): Promise<LearnedRule[] | null> {
    await ready;
    return withPuzzle(size, stars, regions, (puzzle) => puzzle.learnedRules(limit));
  },

  async grade(size: number, stars: number, regions: number[]): Promise<Grade | null> {
    await ready;
    return withPuzzle(size, stars, regions, (puzzle) => puzzle.grade(unmarked(size)));
  },

  cancel(key: string) {
    bump(key);
  },

  /** Tries stars-first puzzles until one is unique, reporting the latest attempt as it goes. */
  async generate(
    key: string,
    size: number,
    stars: number,
    seed: number,
    onProgress: (attempt: Attempt, count: number) => void,
  ): Promise<{ attempt: Attempt; count: number } | null> {
    await ready;
    const id = bump(key);
    const generator = new Generator(size, stars, seed);
    let lastProgress = 0;
    try {
      for (let count = 1; ; count++) {
        const attempt: Attempt = generator.attempt();
        if (attempt.other === undefined) return { attempt, count };
        if (performance.now() - lastProgress >= PROGRESS_EVERY_MS) {
          lastProgress = performance.now();
          onProgress(attempt, count);
        }
        await nextTurn();
        if (generation.get(key) !== id) return null;
      }
    } finally {
      generator.free();
    }
  },
};

export type OracleApi = typeof api;

expose(api);
