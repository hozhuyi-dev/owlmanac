<script setup lang="ts">
import { computed, ref } from "vue";
import { PUZZLES } from "../engine/puzzles";
import { RUNG_NAMES, useLadder } from "../engine/use-ladder";
import type { Outcome } from "../engine/types";

const LIMIT_MS = 5000;

const { results, current, error, start, reset } = useLadder(LIMIT_MS);

const selected = ref(0);
const puzzle = computed(() => PUZZLES[selected.value]);

const OUTCOMES: Record<Outcome, string> = {
  waiting: "",
  running: "Running…",
  solved: "Solved",
  noSolution: "No solution",
  gaveUp: `Stopped at ${LIMIT_MS / 1000} s`,
  cancelled: "",
};

const status = computed(() => {
  if (error.value) return "The solver stopped with an error.";
  if (current.value !== undefined) return `Racing ${RUNG_NAMES[current.value]}, ${current.value + 1} of ${RUNG_NAMES.length}…`;
  if (results.value.every((result) => result.outcome === "waiting")) return "Pick a puzzle and run the ladder.";
  return "Done. Pick another puzzle, or run this one again.";
});

const count = (value: number) => value.toLocaleString("en-GB");
const milliseconds = (value: number) => `${count(Math.round(value))} ms`;
const shown = (outcome: Outcome) => outcome !== "waiting" && outcome !== "cancelled";
</script>

<template>
  <div class="ladder">
    <div class="chart-controls">
      <select v-model="selected" aria-label="Puzzle" @change="reset">
        <option v-for="(option, index) in PUZZLES" :key="option.name" :value="index">{{ option.name }}</option>
      </select>
      <button type="button" @click="start(puzzle)">Run the ladder</button>
    </div>
    <!-- Every lesson sits in the same spot, only the current one visible, so switching never changes the height. -->
    <div class="lessons">
      <p v-for="(option, index) in PUZZLES" :key="option.name" :class="{ hidden: index !== selected }" :aria-hidden="index !== selected">
        {{ option.lesson }}
      </p>
    </div>
    <div class="scroll">
      <table>
        <colgroup>
          <col class="solver" />
          <col class="result" />
          <col class="number" />
          <col class="number" />
          <col class="number" />
          <col class="time" />
        </colgroup>
        <thead>
          <tr>
            <th scope="col">Solver</th>
            <th scope="col">Result</th>
            <th scope="col" class="number">Dead ends</th>
            <th scope="col" class="number">Guesses</th>
            <th scope="col" class="number">Propagations</th>
            <th scope="col" class="number">Time</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(name, rung) in RUNG_NAMES" :key="name" :data-outcome="results[rung].outcome">
            <th scope="row">{{ name }}</th>
            <td>{{ OUTCOMES[results[rung].outcome] }}</td>
            <template v-if="shown(results[rung].outcome)">
              <td class="number">{{ count(results[rung].deadEnds) }}</td>
              <td class="number">{{ count(results[rung].guesses) }}</td>
              <td class="number">{{ count(results[rung].propagations) }}</td>
              <td class="number">{{ milliseconds(results[rung].ms) }}</td>
            </template>
            <template v-else>
              <td v-for="column in 4" :key="column"></td>
            </template>
          </tr>
        </tbody>
      </table>
    </div>
    <p class="figure-status" :class="{ failed: error }" role="status">{{ status }}</p>
  </div>
</template>

<style scoped>
.lessons {
  display: grid;
  margin: 0.75rem 0;
  color: var(--muted);
}

.lessons p {
  grid-area: 1 / 1;
  margin: 0;
}

.lessons .hidden {
  visibility: hidden;
}

.scroll {
  overflow-x: auto;
}

/* Fixed columns, wide enough for the largest counts, so nothing shifts as results arrive. */
table {
  width: 100%;
  min-width: 38rem;
  table-layout: fixed;
  border-collapse: collapse;
  font-size: 0.875rem;
  font-variant-numeric: tabular-nums;
}

col.solver {
  width: 22%;
}

col.result {
  width: 17%;
}

col.number {
  width: 16%;
}

col.time {
  width: 13%;
}

th,
td {
  overflow: hidden;
  padding: 0.35rem 0.5rem 0.35rem 0;
  border-bottom: 1px solid var(--line);
  text-align: left;
  white-space: nowrap;
}

.number {
  text-align: right;
}
</style>
