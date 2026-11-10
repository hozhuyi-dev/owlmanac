import data from "../data/strategies.json";

// The strategy comparison, from star-battle's `strategies` binary: one row per puzzle.

export type StrategyName = "retry" | "aroundStars" | "synthesize";

export type Row = {
  strategy: StrategyName;
  seed: number;
  ms: number;
  designs: number;
  depth: number | null;
  deepestSteps: number;
  regions: string;
};

export const SIZE: number = data.size;
export const STARS: number = data.stars;
export const THREADS: number = data.threads;
export const ROWS = data.puzzles as Row[];

export const STRATEGIES: { name: StrategyName; label: string }[] = [
  { name: "retry", label: "Random regions" },
  { name: "aroundStars", label: "Stars first" },
  { name: "synthesize", label: "Two solvers arguing" },
];

function median(values: number[]): number {
  const sorted = [...values].sort((a, b) => a - b);
  const middle = sorted.length >> 1;
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
}

export type Summary = {
  name: StrategyName;
  label: string;
  puzzles: number;
  medianSeconds: number;
  medianDesigns: number;
  /** Percentage of its puzzles graded Easy (depth 1 or less). */
  easy: number;
  hard: number;
};

export const SUMMARIES: Summary[] = STRATEGIES.map(({ name, label }) => {
  const rows = ROWS.filter((row) => row.strategy === name);
  const easy = rows.filter((row) => row.depth !== null && row.depth <= 1).length;
  return {
    name,
    label,
    puzzles: rows.length,
    medianSeconds: median(rows.map((row) => row.ms / 1000)),
    medianDesigns: median(rows.map((row) => row.designs)),
    easy: (100 * easy) / rows.length,
    hard: (100 * (rows.length - easy)) / rows.length,
  };
});

/** Each cell's region, as numbers, from a row's string. */
export const regionsOf = (row: Row) => [...row.regions].map((digit) => parseInt(digit, 36));
