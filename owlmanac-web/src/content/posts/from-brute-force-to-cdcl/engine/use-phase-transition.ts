import { onMounted, shallowRef } from "vue";
import { satLab } from "./client";

export const SIZES = [50, 100, 150];
export const RATIOS = Array.from({ length: 13 }, (_, index) => 3 + index * 0.25);
const FORMULAS_PER_POINT = 30;

export type Point = {
  vars: number;
  ratio: number;
  /** Percentage of the formulas that had a solution. */
  satisfiable: number;
  medianConflicts: number;
};

function median(values: number[]): number {
  const sorted = [...values].sort((a, b) => a - b);
  const middle = sorted.length >> 1;
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
}

/**
 * Solves random 3-SAT formulas across sizes and clause-to-variable ratios, in the reader's
 * browser. Seeds are fixed, so every reader gets the same formulas and the same answers.
 */
export function usePhaseTransition() {
  const points = shallowRef<Point[]>([]);
  const solved = shallowRef(0);
  const error = shallowRef<unknown>();
  const total = SIZES.length * RATIOS.length * FORMULAS_PER_POINT;

  async function sweep() {
    for (const vars of SIZES) {
      for (const [ratioIndex, ratio] of RATIOS.entries()) {
        const clauses = Math.round(vars * ratio);
        let satisfiable = 0;
        const conflicts: number[] = [];
        for (let index = 0; index < FORMULAS_PER_POINT; index++) {
          const seed = vars * 100_000 + ratioIndex * 1_000 + index;
          const answer = await satLab().solveRandom(vars, clauses, seed);
          if (answer.satisfiable) satisfiable++;
          conflicts.push(answer.conflicts);
          solved.value++;
        }
        const point = {
          vars,
          ratio,
          satisfiable: (100 * satisfiable) / FORMULAS_PER_POINT,
          medianConflicts: median(conflicts),
        };
        points.value = [...points.value, point];
      }
    }
  }

  onMounted(() => sweep().catch((failure) => (error.value = failure)));

  return { points, solved, total, error };
}
