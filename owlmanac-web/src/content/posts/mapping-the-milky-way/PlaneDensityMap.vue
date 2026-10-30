<script setup lang="ts">
import * as Plot from "@observablehq/plot";
import { ref } from "vue";
import { parquet, query } from "@/lib/duckdb";
import { usePlot } from "@/lib/plot";

// `limit` picks the grid: stars within that many parsecs of the Sun.
const props = defineProps<{ src: string; limit: number }>();

type Bin = { x: number; y: number; size: number; star_count: number };

const el = ref<HTMLElement | null>(null);
let bins: Promise<Bin[]> | undefined;

usePlot(el, async (width) => {
  bins ??= parquet(props.src).then((table) =>
    query<Bin>(
      `SELECT x, y, bin_size::DOUBLE AS size, star_count::DOUBLE AS star_count
       FROM ${table} WHERE distance_limit = ${props.limit}`,
    ),
  );
  const rows = await bins;
  const reach = Math.max(...rows.map((b) => Math.max(-b.x, b.x + b.size, -b.y, b.y + b.size)));
  const side = Math.min(width, 560);
  return Plot.plot({
    width: side,
    height: side,
    marginLeft: 55,
    x: { domain: [-reach, reach], label: "Galactic X (pc), towards the Galactic Centre →" },
    y: { domain: [-reach, reach], label: "↑ Galactic Y (pc), direction of rotation" },
    color: { type: "log", scheme: "turbo", label: "Stars per bin", legend: true },
    marks: [
      Plot.rect(rows, {
        x1: "x",
        x2: (d: Bin) => d.x + d.size,
        y1: "y",
        y2: (d: Bin) => d.y + d.size,
        fill: "star_count",
        channels: { "X (pc)": "x", "Y (pc)": "y" },
        tip: { format: { x1: false, x2: false, y1: false, y2: false } },
      }),
      Plot.dot([{ x: 0, y: 0 }], { x: "x", y: "y", symbol: "star", r: 6, fill: "var(--bg)", stroke: "currentColor" }),
      Plot.text([{ x: 0, y: 0 }], { x: "x", y: "y", text: () => "Sun", dy: -14, fill: "currentColor", stroke: "var(--bg)", strokeWidth: 3, paintOrder: "stroke" }),
    ],
  });
});
</script>

<template>
  <div ref="el" class="chart" />
</template>
