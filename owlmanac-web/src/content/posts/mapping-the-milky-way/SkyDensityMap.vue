<script setup lang="ts">
import * as Plot from "@observablehq/plot";
import { ref } from "vue";
import { parquet, query } from "@/lib/duckdb";
import { usePlot } from "@/lib/plot";

const props = defineProps<{ src: string; frame: "equatorial" | "galactic" }>();

type Pixel = { healpix: number; star_count: number; lon: number; lat: number };

const el = ref<HTMLElement | null>(null);
const [lon, lat, lonLabel, latLabel] =
  props.frame === "galactic" ? ["l", "b", "l (°)", "b (°)"] : ["ra", "dec", "RA (°)", "Dec (°)"];
let pixels: Promise<Pixel[]> | undefined;

usePlot(el, async (width) => {
  pixels ??= parquet(props.src).then((table) =>
    query<Pixel>(
      `SELECT healpix, star_count::DOUBLE AS star_count, ${lon} AS lon, ${lat} AS lat FROM ${table}`,
    ),
  );
  const rows = await pixels;
  return Plot.plot({
    width,
    projection: { type: "equal-earth" },
    color: { type: "log", scheme: "turbo", label: "Stars per pixel", legend: true },
    marks: [
      Plot.sphere({ stroke: "currentColor", strokeOpacity: 0.3 }),
      Plot.graticule({ strokeOpacity: 0.12 }),
      // Longitude increases to the left, as it does looking up at the sky.
      Plot.dot(rows, {
        x: (d: Pixel) => -d.lon,
        y: "lat",
        r: width / 190,
        fill: "star_count",
        channels: { HEALPix: "healpix", [lonLabel]: "lon", [latLabel]: "lat" },
        tip: { format: { x: false, y: false, [lonLabel]: ".1f", [latLabel]: ".1f" } },
      }),
    ],
  });
});
</script>

<template>
  <div ref="el" class="chart" />
</template>
