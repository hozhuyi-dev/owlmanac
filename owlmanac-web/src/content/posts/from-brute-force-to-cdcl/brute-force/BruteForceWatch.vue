<script setup lang="ts">
import { computed } from "vue";
import { PUZZLES } from "../engine/puzzles";
import { useBruteForce } from "../engine/use-brute-force";

const LIMIT_MS = 120_000;
const puzzle = PUZZLES[0];

const { result, running, error, elapsedMs, start, stop } = useBruteForce(puzzle, LIMIT_MS);

const clues = [...puzzle.clues].map((cell) => (cell === "." ? 0 : Number(cell)));
const count = (value: number) => value.toLocaleString("en-GB");

// The clues, with whatever digits the search has written in so far.
const cells = computed(() => {
  const grid = result.value?.grid;
  return clues.map((clue, index) => ({ digit: clue || grid?.[index] || 0, clue: clue !== 0 }));
});

const status = computed(() => {
  if (error.value) return "The solver couldn't start in this browser.";
  const outcome = result.value?.outcome;
  if (!result.value || outcome === "cancelled") return "Press Start, and watch it fill in the grid.";
  const seconds = (elapsedMs.value / 1000).toFixed(1);
  if (outcome === "solved") return `Solved after ${seconds} s.`;
  if (outcome === "gaveUp") return `Still stuck after ${Math.round(elapsedMs.value / 1000)} s, so I stopped it for you.`;
  return `${seconds} s and counting.`;
});
</script>

<template>
  <div class="watch">
    <div class="grid" role="img" aria-label="The puzzle: a 9×9 Sudoku with 17 clues, and the digits the solver is trying">
      <span v-for="(cell, index) in cells" :key="index" :class="{ guess: !cell.clue }">{{ cell.digit || "" }}</span>
    </div>
    <div class="panel">
      <dl class="tiles">
        <div>
          <dt>Guesses</dt>
          <dd>{{ count(result?.guesses ?? 0) }}</dd>
        </div>
        <div>
          <dt>Dead ends</dt>
          <dd>{{ count(result?.deadEnds ?? 0) }}</dd>
        </div>
      </dl>
      <div class="chart-controls">
        <button v-if="!running" type="button" @click="start">Start</button>
        <button v-else type="button" @click="stop">Stop</button>
      </div>
      <p class="figure-status" :class="{ failed: error }" role="status">{{ status }}</p>
    </div>
  </div>
</template>

<style scoped>
.watch {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
  gap: 1.5rem;
  align-items: center;
}

.grid {
  display: grid;
  grid-template-columns: repeat(9, minmax(0, 1fr));
  width: 100%;
  max-width: 18rem;
  border: 2px solid var(--fg);
  font-family: var(--font-mono);
}

.grid span {
  display: grid;
  aspect-ratio: 1;
  place-items: center;
  border-right: 1px solid var(--line);
  border-bottom: 1px solid var(--line);
}

.grid span:nth-child(9n) {
  border-right: 0;
}

.grid span:nth-child(3n):not(:nth-child(9n)) {
  border-right: 2px solid var(--fg);
}

.grid span:nth-child(n + 19):nth-child(-n + 27),
.grid span:nth-child(n + 46):nth-child(-n + 54) {
  border-bottom: 2px solid var(--fg);
}

.grid span:nth-child(n + 73) {
  border-bottom: 0;
}

/* Clues in bold; the solver's guesses in the accent colour. */
.grid span:not(.guess) {
  font-weight: 700;
}

.grid span.guess {
  color: var(--accent);
}

.tiles {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem 2rem;
  margin: 0 0 1rem;
}

dt {
  color: var(--muted);
  font-size: 0.875rem;
}

/* Wide enough for billions, so the layout never shifts as the counts grow. */
dd {
  min-width: 13ch;
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}
</style>
