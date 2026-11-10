<script setup lang="ts">
import { proxy } from "comlink";
import { computed, ref, shallowRef } from "vue";
import type { Attempt, Cell } from "@/wasm/star-battle.types";
import { oracle } from "../engine/client";
import RegionGrid from "../grid/RegionGrid.vue";

const KEY = "live-generator";
const SIZE = 10;
const STARS = 2;

// Before the first run, a plain grid with no region lines, so the figure already has its size.
const blank = Array.from({ length: SIZE * SIZE }, () => 0);

const attempt = shallowRef<Attempt>();
const count = ref(0);
const running = ref(false);
const done = ref(false);
const failed = ref(false);
let seed = 1;
let activeRun = 0;

async function run() {
  const id = ++activeRun;
  running.value = true;
  done.value = false;
  failed.value = false;
  const show = (latest: Attempt, tries: number) => {
    if (id !== activeRun) return;
    attempt.value = latest;
    count.value = tries;
  };
  try {
    const result = await oracle().generate(KEY, SIZE, STARS, seed++, proxy(show));
    if (result && id === activeRun) {
      show(result.attempt, result.count);
      done.value = true;
    }
  } catch {
    if (id === activeRun) failed.value = true;
  } finally {
    if (id === activeRun) running.value = false;
  }
}

function stop() {
  activeRun++;
  running.value = false;
  oracle().cancel(KEY);
}

const at = (cell: Cell) => cell.row * SIZE + cell.col;

const symbols = computed(() => {
  const marks: Record<number, string> = {};
  attempt.value?.stars.forEach((cell) => (marks[at(cell)] = "★"));
  attempt.value?.other?.filter((cell) => marks[at(cell)] === undefined).forEach((cell) => (marks[at(cell)] = "☆"));
  return marks;
});
const differs = computed(
  () => new Set(attempt.value?.other?.map(at).filter((index) => symbols.value[index] === "☆") ?? []),
);

const status = computed(() => {
  if (failed.value) return "The solver stopped with an error.";
  if (!attempt.value) return "Press Make a puzzle to watch one being made.";
  const tries = count.value.toLocaleString("en-GB");
  if (done.value) return `Attempt ${tries}: no other solution. A new puzzle.`;
  if (!running.value) return `Stopped at attempt ${tries}.`;
  return `Attempt ${tries}: a second solution (hollow stars). Trying again.`;
});
</script>

<template>
  <div class="generator">
    <RegionGrid
      :size="SIZE"
      :regions="attempt?.regions ?? blank"
      :symbols="symbols"
      :marked="differs"
      label="A 10×10 Star Battle grid being made: the chosen stars, the regions grown around them, and any other solution"
    />
    <div class="side">
      <div class="chart-controls">
        <button v-if="!running" type="button" @click="run">{{ done ? "Make another" : "Make a puzzle" }}</button>
        <button v-else type="button" @click="stop">Stop</button>
      </div>
      <p class="figure-status" :class="{ failed }" role="status">{{ status }}</p>
    </div>
  </div>
</template>

<style scoped>
.generator {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
  gap: 1.5rem;
  align-items: start;
}

/* Long enough for the longest message on a phone, so a change of status never moves anything. */
.figure-status {
  min-height: 4.5em;
}
</style>
