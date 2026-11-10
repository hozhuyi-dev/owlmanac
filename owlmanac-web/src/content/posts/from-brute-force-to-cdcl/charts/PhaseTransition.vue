<script setup lang="ts">
import * as Plot from "@observablehq/plot";
import { computed, ref } from "vue";
import { SERIES } from "@/core/plot/palette";
import { usePlot } from "@/core/plot/use-plot";
import { RATIOS, SIZES, usePhaseTransition, type Point } from "../engine/use-phase-transition";

const HEIGHT = 240;
const PEAK = 4.26;
const colours = SIZES.map((_, index) => SERIES[index]);

const { points, solved, total, error } = usePhaseTransition();

const satisfiableEl = ref<HTMLElement | null>(null);
const conflictsEl = ref<HTMLElement | null>(null);

const label = (vars: number) => `${vars} variables`;
const percent = (value: number) => `${Math.round(value)}%`;
const count = (value: number) => Math.round(value).toLocaleString("en-GB");

const status = computed(() => {
  if (error.value) return "The solver stopped with an error.";
  if (solved.value < total) return `Solving formula ${count(solved.value + 1)} of ${count(total)}…`;
  return `Solved all ${count(total)} formulas in your browser.`;
});

// The lines meet at 0% on the first chart, so only the second, where they stay apart, labels its ends.
function chart(
  width: number,
  y: "satisfiable" | "medianConflicts",
  yOptions: Plot.ScaleOptions,
  format: (value: number) => string,
  labelEnds: boolean,
) {
  const rows = points.value;
  const ends = SIZES.map((vars) => rows.filter((point) => point.vars === vars).at(-1)).filter(
    (point): point is Point => point !== undefined,
  );
  return Plot.plot({
    width,
    height: HEIGHT,
    marginRight: 100,
    x: { domain: [RATIOS[0], RATIOS.at(-1)!], label: "Clauses per variable" },
    y: { grid: true, ...yOptions },
    color: { type: "ordinal", domain: SIZES, range: colours },
    marks: [
      Plot.ruleX([PEAK], { strokeOpacity: 0.35 }),
      Plot.line(rows, { x: "ratio", y, stroke: "vars", strokeWidth: 2, z: "vars" }),
      Plot.dot(rows, { x: "ratio", y, fill: "vars", r: 4, stroke: "var(--bg)", strokeWidth: 2 }),
      labelEnds
        ? Plot.text(ends, { x: "ratio", y, text: (point: Point) => label(point.vars), dx: 8, textAnchor: "start" })
        : null,
      Plot.tip(
        rows,
        Plot.pointer({
          x: "ratio",
          y,
          channels: { Formulas: (point: Point) => label(point.vars), Value: (point: Point) => format(point[y]) },
          format: { x: ".2f", y: false, stroke: false, fill: false },
        }),
      ),
    ],
  });
}

usePlot(
  satisfiableEl,
  (width) => chart(width, "satisfiable", { domain: [0, 100], label: "Satisfiable (%)" }, percent, false),
  [points],
);
// A log scale can't show zero, and the easiest formulas need no conflicts at all, so it starts at 1.
usePlot(
  conflictsEl,
  (width) =>
    chart(
      width,
      "medianConflicts",
      {
        type: "log",
        clamp: true,
        domain: [1, 10_000],
        ticks: [1, 10, 100, 1_000, 10_000],
        tickFormat: ",",
        label: "Median conflicts (log scale)",
      },
      count,
      true,
    ),
  [points],
);
</script>

<template>
  <p class="figure-status" :class="{ failed: error }" role="status">{{ status }}</p>
  <ul class="chart-legend" aria-label="Formula sizes">
    <li v-for="(vars, index) in SIZES" :key="vars">
      <span class="key" :style="{ background: colours[index] }" />{{ label(vars) }}
    </li>
  </ul>
  <div ref="satisfiableEl" class="chart" :style="{ '--chart-height': `${HEIGHT}px` }" />
  <div ref="conflictsEl" class="chart" :style="{ '--chart-height': `${HEIGHT}px` }" />
  <details>
    <summary>The numbers</summary>
    <table>
      <thead>
        <tr>
          <th scope="col">Variables</th>
          <th scope="col">Clauses per variable</th>
          <th scope="col">Satisfiable</th>
          <th scope="col">Median conflicts</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="point in points" :key="`${point.vars}-${point.ratio}`">
          <td>{{ point.vars }}</td>
          <td>{{ point.ratio.toFixed(2) }}</td>
          <td>{{ percent(point.satisfiable) }}</td>
          <td>{{ count(point.medianConflicts) }}</td>
        </tr>
      </tbody>
    </table>
  </details>
</template>

<style scoped>
details {
  margin-top: 0.5rem;
  font-size: 0.875rem;
}

table {
  font-variant-numeric: tabular-nums;
}
</style>
