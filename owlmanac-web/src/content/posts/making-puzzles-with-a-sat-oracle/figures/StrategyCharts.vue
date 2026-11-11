<script setup lang="ts">
import * as Plot from "@observablehq/plot";
import { ref } from "vue";
import { SERIES } from "@/core/plot/palette";
import { usePlot } from "@/core/plot/use-plot";
import { SUMMARIES, THREADS, type Summary } from "../engine/strategies";

const HEIGHT = 150;
const EASY_COLOUR = SERIES[2];
const HARD_COLOUR = SERIES[1];

const timeEl = ref<HTMLElement | null>(null);
const levelEl = ref<HTMLElement | null>(null);

const seconds = (value: number) => `${value.toFixed(1)} s`;
const percent = (value: number) => `${Math.round(value)}%`;

usePlot(timeEl, (width) =>
  Plot.plot({
    width,
    height: HEIGHT,
    marginLeft: 150,
    marginRight: 50,
    x: { label: "Median seconds to make one puzzle", grid: true },
    y: { label: null, domain: SUMMARIES.map((row) => row.label) },
    marks: [
      Plot.barX(SUMMARIES, { x: "medianSeconds", y: "label", fill: SERIES[0], rx: 2 }),
      Plot.text(SUMMARIES, { x: "medianSeconds", y: "label", text: (row: Summary) => seconds(row.medianSeconds), dx: 6, textAnchor: "start" }),
      Plot.ruleX([0]),
      Plot.tip(
        SUMMARIES,
        Plot.pointerY({
          x: "medianSeconds",
          y: "label",
          channels: { "Designs tried": "medianDesigns", Puzzles: "puzzles" },
          format: { x: ".1f" },
        }),
      ),
    ],
  }),
);

// Easy or Medium (depth 1) and Hard (depth 2) as two segments of one bar per strategy, adding to 100%.
const segments = SUMMARIES.flatMap((row) => [
  { label: row.label, level: "Easy or Medium", share: row.easy },
  { label: row.label, level: "Hard", share: row.hard },
]);

usePlot(levelEl, (width) =>
  Plot.plot({
    width,
    height: HEIGHT,
    marginLeft: 150,
    marginRight: 50,
    x: { label: "Share of puzzles (%)", domain: [0, 100], grid: true },
    y: { label: null, domain: SUMMARIES.map((row) => row.label) },
    color: { domain: ["Easy or Medium", "Hard"], range: [EASY_COLOUR, HARD_COLOUR] },
    marks: [
      Plot.barX(segments, Plot.stackX({ x: "share", y: "label", fill: "level", stroke: "var(--bg)", strokeWidth: 2 })),
      Plot.tip(
        segments,
        Plot.pointer(Plot.stackX({ x: "share", y: "label", channels: { Level: "level" }, format: { x: ".0f", fill: false } })),
      ),
    ],
  }),
);
</script>

<template>
  <div class="charts">
    <div ref="timeEl" class="chart" :style="{ '--chart-height': `${HEIGHT}px` }" />
    <ul class="chart-legend" aria-label="Levels">
      <li><span class="key" :style="{ background: EASY_COLOUR }" />Easy or Medium</li>
      <li><span class="key" :style="{ background: HARD_COLOUR }" />Hard</li>
    </ul>
    <div ref="levelEl" class="chart" :style="{ '--chart-height': `${HEIGHT}px` }" />
    <details>
      <summary>The numbers</summary>
      <table>
        <thead>
          <tr>
            <th scope="col">Strategy</th>
            <th scope="col">Puzzles</th>
            <th scope="col">Median time</th>
            <th scope="col">Median designs tried</th>
            <th scope="col">Easy or Medium</th>
            <th scope="col">Hard</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in SUMMARIES" :key="row.name">
            <td>{{ row.label }}</td>
            <td>{{ row.puzzles }}</td>
            <td>{{ seconds(row.medianSeconds) }}</td>
            <td>{{ row.medianDesigns }}</td>
            <td>{{ percent(row.easy) }}</td>
            <td>{{ percent(row.hard) }}</td>
          </tr>
        </tbody>
      </table>
      <p>Made natively, {{ THREADS }} puzzles at a time, so each took longer than it would alone.</p>
    </details>
  </div>
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
