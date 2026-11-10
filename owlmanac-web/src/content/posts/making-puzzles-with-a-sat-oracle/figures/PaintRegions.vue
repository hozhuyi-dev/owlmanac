<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useTask } from "@/core/async/use-task";
import { SERIES } from "@/core/plot/palette";
import type { Cell } from "@/wasm/star-battle.types";
import { oracle } from "../engine/client";
import { PAINT_START } from "../engine/puzzles";
import RegionGrid from "../grid/RegionGrid.vue";

const { size, stars } = PAINT_START;
const regions = ref([...PAINT_START.regions]);
const brush = ref(0);
const colours = SERIES.slice(0, size);

// A copy, because a worker can only be sent plain data, never Vue's reactive wrapper around it.
const check = useTask(async (layout: number[]) => oracle().solutions(size, stars, [...layout], 2));

function paint(index: number) {
  if (regions.value[index] === brush.value) return;
  const next = [...regions.value];
  next[index] = brush.value;
  regions.value = next;
  check.start(next);
}

function reset() {
  regions.value = [...PAINT_START.regions];
  check.start(regions.value);
}

onMounted(() => check.start(regions.value));

const at = (cell: Cell) => cell.row * size + cell.col;
const found = computed(() => check.result.value?.found ?? []);

// The first solution's stars, and where a second one differs from it.
const symbols = computed(() => {
  const marks: Record<number, string> = {};
  found.value[0]?.forEach((cell) => (marks[at(cell)] = "★"));
  found.value[1]?.filter((cell) => marks[at(cell)] === undefined).forEach((cell) => (marks[at(cell)] = "☆"));
  return marks;
});
const differs = computed(() => new Set(found.value[1]?.map(at).filter((index) => symbols.value[index] === "☆")));

const status = computed(() => {
  if (check.error.value) return "The solver stopped with an error.";
  const result = check.result.value;
  if (result === undefined) return "Asking the solver…";
  if (result === null) return "Every region needs at least one cell.";
  if (result.found.length === 0) return "No solution: these regions can't be filled.";
  if (result.found.length === 1) return "Exactly one solution. A proper puzzle.";
  return "At least two solutions: the hollow stars show where a second one differs.";
});
</script>

<template>
  <div class="paint">
    <RegionGrid
      :size="size"
      :regions="regions"
      :colours="colours"
      :symbols="symbols"
      :marked="differs"
      paintable
      label="A 6×6 Star Battle grid you can repaint, with the solver's answer drawn on it"
      @paint="paint"
    />
    <div class="tools">
      <p class="label">Paint with</p>
      <div class="brushes" role="radiogroup" aria-label="Region to paint">
        <button
          v-for="(colour, region) in colours"
          :key="region"
          type="button"
          role="radio"
          :aria-checked="brush === region"
          :aria-label="`Region ${region + 1}`"
          :style="{ background: colour }"
          @click="brush = region"
        />
      </div>
      <div class="chart-controls">
        <button type="button" @click="reset">Start again</button>
      </div>
      <p class="figure-status" :class="{ failed: check.error.value }" role="status">{{ status }}</p>
    </div>
  </div>
</template>

<style scoped>
.paint {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
  gap: 1.5rem;
  align-items: start;
}

.label {
  margin: 0 0 0.4rem;
  color: var(--muted);
  font-size: 0.875rem;
}

.brushes {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.brushes button {
  width: 2rem;
  height: 2rem;
  border: 2px solid transparent;
  border-radius: 50%;
  cursor: pointer;
}

.brushes button[aria-checked="true"] {
  border-color: var(--fg);
  box-shadow: 0 0 0 2px var(--bg) inset;
}

/* Long enough for the longest message, so a change of verdict never moves anything. */
.figure-status {
  min-height: 3em;
}
</style>
