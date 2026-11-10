<script setup lang="ts">
import { computed, onMounted, ref, shallowRef } from "vue";
import type { LearnedRule } from "@/wasm/star-battle.types";
import { oracle } from "../engine/client";
import { ROWS, SIZE, STARS, regionsOf } from "../engine/strategies";
import RegionGrid from "../grid/RegionGrid.vue";

// The first Easy and the first Hard puzzle made by choosing the stars first.
const LIMIT = 1000;
const picks = [
  { level: "Easy", row: ROWS.find((row) => row.strategy === "aroundStars" && row.depth === 1)! },
  { level: "Hard", row: ROWS.find((row) => row.strategy === "aroundStars" && row.depth === 2)! },
].map(({ level, row }) => ({ level, regions: regionsOf(row) }));

const learned = shallowRef<(LearnedRule[] | null)[]>([]);
const failed = ref(false);
const shown = ref([0, 0]);

onMounted(async () => {
  try {
    learned.value = await Promise.all(picks.map((pick) => oracle().learnedRules(SIZE, STARS, pick.regions, LIMIT)));
  } catch {
    failed.value = true;
  }
});

// One learnt rule on the grid: ★ where it says "a star", × where it says "no star".
function symbols(side: number) {
  const rule = learned.value[side]?.[shown.value[side]];
  const marks: Record<number, string> = {};
  rule?.cannotAllHold.forEach(({ cell, mark }) => (marks[cell.row * SIZE + cell.col] = mark === "star" ? "★" : "×"));
  return marks;
}

function step(side: number, by: number) {
  const count = learned.value[side]?.length ?? 0;
  if (count === 0) return;
  const next = [...shown.value];
  next[side] = (next[side] + by + count) % count;
  shown.value = next;
}

const counts = computed(() => learned.value.map((rules) => rules?.length));

function caption(side: number) {
  if (failed.value) return "The solver stopped with an error.";
  const count = counts.value[side];
  if (count === undefined || count === null) return "Asking the solver…";
  if (count === 0) return "Learnt nothing: forced moves and lucky guesses were enough.";
  const which = shown.value[side] + 1;
  return `Rule ${which} of ${count.toLocaleString("en-GB")}${count === LIMIT ? "+" : ""}: these can't all hold.`;
}
</script>

<template>
  <div class="pair">
    <div v-for="(pick, side) in picks" :key="pick.level" class="side">
      <p class="level">{{ pick.level }}</p>
      <RegionGrid
        :size="SIZE"
        :regions="pick.regions"
        :symbols="symbols(side)"
        :label="`An ${pick.level} puzzle, with one rule the solver learnt drawn on it`"
      />
      <div class="chart-controls">
        <button type="button" :disabled="!counts[side]" aria-label="Previous rule" @click="step(side, -1)">←</button>
        <button type="button" :disabled="!counts[side]" aria-label="Next rule" @click="step(side, 1)">→</button>
      </div>
      <p class="figure-status" :class="{ failed }" role="status">{{ caption(side) }}</p>
    </div>
  </div>
</template>

<style scoped>
.pair {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
  gap: 1.5rem;
}

.level {
  margin: 0 0 0.4rem;
  font-weight: 600;
}

.chart-controls {
  margin-top: 0.75rem;
}

/* Room for the longest caption, so stepping through rules never moves anything. */
.figure-status {
  min-height: 3em;
}
</style>
