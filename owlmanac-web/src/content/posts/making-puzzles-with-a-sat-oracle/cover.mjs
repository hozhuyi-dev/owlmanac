// The cover (npm run covers): a real attempt at a puzzle, with a second solution still to be ruled out.
// Needs the shared star-battle WASM (npm run wasm).
import fs from "node:fs";
import init, { Generator } from "../../../wasm/star-battle/star_battle_wasm.js";
import { ACCENT, BG, HEIGHT, SERIES, WIDTH, gridLines, starPath, svg } from "../../../../scripts/cover-kit.mjs";

export default async function cover() {
  await init({ module_or_path: fs.readFileSync(new URL("../../../wasm/star-battle/star_battle_wasm_bg.wasm", import.meta.url)) });
  const size = 10;
  const generator = new Generator(size, 2, 7);
  let attempt;
  // An attempt whose second solution differs in at least a few cells, so it's easy to see.
  do {
    attempt = generator.attempt();
  } while (!attempt.other || attempt.other.filter((cell) => !attempt.stars.some((star) => star.row === cell.row && star.col === cell.col)).length < 4);

  const cell = 78;
  const x = (WIDTH - cell * size) / 2;
  const y = (HEIGHT - cell * size) / 2;
  const colours = colourRegions(attempt.regions, size);
  const fills = attempt.regions
    .map((region, index) => `<rect x="${x + (index % size) * cell}" y="${y + Math.floor(index / size) * cell}" width="${cell}" height="${cell}" fill="${mix(colours[region], 0.32)}"/>`)
    .join("");
  const same = (a, b) => a.row === b.row && a.col === b.col;
  const centre = ({ row, col }) => [x + col * cell + cell / 2, y + row * cell + cell / 2];
  const stars = attempt.stars.map((star) => `<path d="${starPath(...centre(star), cell * 0.32)}" fill="${ACCENT}"/>`).join("");
  const others = attempt.other
    .filter((cellAt) => !attempt.stars.some((star) => same(star, cellAt)))
    .map((cellAt) => `<path d="${starPath(...centre(cellAt), cell * 0.32)}" fill="none" stroke="${SERIES[1]}" stroke-width="4" stroke-linejoin="round"/>`)
    .join("");
  return svg(fills + gridLines({ x, y, cell, size, regions: attempt.regions }) + stars + others);
}

/** Gives each region one of the series colours, never the same as a neighbour's. */
function colourRegions(regions, size) {
  const count = Math.max(...regions) + 1;
  const neighbours = Array.from({ length: count }, () => new Set());
  regions.forEach((region, index) => {
    const right = index % size < size - 1 ? regions[index + 1] : region;
    const below = index + size < regions.length ? regions[index + size] : region;
    for (const other of [right, below]) {
      if (other !== region) {
        neighbours[region].add(other);
        neighbours[other].add(region);
      }
    }
  });
  const colours = [];
  for (let region = 0; region < count; region++) {
    const taken = new Set([...neighbours[region]].map((other) => colours[other]));
    colours[region] = SERIES.findIndex((_, index) => !taken.has(index));
  }
  return colours.map((index) => SERIES[index]);
}

const mix = (colour, amount) => {
  const [r, g, b] = [1, 3, 5].map((at) => parseInt(colour.slice(at, at + 2), 16));
  const [R, G, B] = [1, 3, 5].map((at) => parseInt(BG.slice(at, at + 2), 16));
  const blend = (a, base) => Math.round(base + (a - base) * amount);
  return `rgb(${blend(r, R)},${blend(g, G)},${blend(b, B)})`;
};
