<script setup lang="ts">
import * as Plot from "@observablehq/plot";
import { ref } from "vue";
import { parquet, query } from "@/lib/duckdb";
import { usePlot } from "@/lib/plot";

// A square patch of sky centred on (ra, dec), `width` degrees across.
const props = defineProps<{ src: string; ra: number; dec: number; width: number }>();

type Star = { source_id: string; ra: number; dec: number; g_mag: number };

const MAX_BRIGHTNESS = 12; // 21 − G for the brightest stars in the patch

const el = ref<HTMLElement | null>(null);
const slider = ref(21.5);
const faintest = ref(21.5);
const shown = ref<number>();
let stars: { mag: number; rows: Promise<Star[]> } | undefined;

// The magnitude cut runs in DuckDB, so dragging the slider queries the parquet file again.
function load(mag: number): Promise<Star[]> {
  if (stars?.mag !== mag) {
    const rows = parquet(props.src).then((table) =>
      query<Star>(
        `SELECT source_id, ra, dec, g_mag FROM ${table} WHERE g_mag <= ${mag} ORDER BY g_mag DESC`,
      ),
    );
    stars = { mag, rows };
  }
  return stars.rows;
}

usePlot(
  el,
  async (width) => {
    const rows = await load(faintest.value);
    shown.value = rows.length;
    const half = props.width / 2;
    const brightness = (d: Star) => 21 - d.g_mag;
    return Plot.plot({
      width,
      height: Math.min(width, 480),
      marginLeft: 50,
      x: { domain: [props.ra - half, props.ra + half], label: "Right ascension (°)" },
      y: { domain: [props.dec - half, props.dec + half], label: "Declination (°)" },
      r: { domain: [0, MAX_BRIGHTNESS], range: [0.5, 9], clamp: true },
      color: {
        scheme: "viridis",
        domain: [0, MAX_BRIGHTNESS],
        label: "Relative brightness (21 − G)",
        legend: true,
      },
      marks: [
        Plot.dot(rows, {
          x: "ra",
          y: "dec",
          r: brightness,
          fill: brightness,
          fillOpacity: 0.85,
          channels: { Source: "source_id", "G magnitude": "g_mag" },
          tip: { format: { x: ".4f", y: ".4f", r: false, fill: ".1f" } },
        }),
      ],
    });
  },
  [faintest],
);
</script>

<template>
  <div>
    <label class="chart-controls">
      <span>Faintest star shown: G ≤ {{ slider.toFixed(1) }}</span>
      <input
        v-model.number="slider"
        type="range"
        min="10"
        max="21.5"
        step="0.1"
        @change="faintest = slider"
      />
      <span class="muted">{{ shown ?? "…" }} stars</span>
    </label>
    <div ref="el" class="chart" />
  </div>
</template>
