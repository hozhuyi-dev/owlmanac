// Draws the covers for the SAT series from the posts' own puzzles, as 1600×900 PNGs in each
// post's folder. Run with `npm run covers`, after `npm run wasm:star-battle`. The covers are
// committed; the same inputs always draw the same pictures.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import sharp from "sharp";
import init, { Generator, StarBattle } from "../src/wasm/star-battle/star_battle_wasm.js";

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const posts = path.join(root, "src/content/posts");

const WIDTH = 1600;
const HEIGHT = 900;
// The dark theme's tokens from global.css: covers are drawn in dark mode.
const BG = "#2b2b2b";
const LINE = "#4a4a4a";
const FG = "#fafafa";
const ACCENT = "#9bd4e4";
const FONT = "Inter, 'Segoe UI', Arial, sans-serif";
// Series colours, read from the site's palette so the covers match its charts.
const SERIES = [...fs.readFileSync(path.join(root, "src/core/plot/palette.ts"), "utf8").matchAll(/"(#[0-9a-f]{6})"/g)].map(
  (match) => match[1],
);

/** A small, seeded random number generator, so covers come out the same every time. */
function random(seed) {
  let state = seed >>> 0 || 1;
  return () => {
    state ^= state << 13;
    state ^= state >>> 17;
    state ^= state << 5;
    return (state >>> 0) / 2 ** 32;
  };
}

function starPath(cx, cy, outer) {
  const inner = outer * 0.42;
  const points = Array.from({ length: 10 }, (_, index) => {
    const radius = index % 2 ? inner : outer;
    const angle = -Math.PI / 2 + (index * Math.PI) / 5;
    return `${(cx + radius * Math.cos(angle)).toFixed(1)},${(cy + radius * Math.sin(angle)).toFixed(1)}`;
  });
  return `M${points.join("L")}Z`;
}

/** Grid lines: thin between cells, thick around the edge and between regions (or boxes). */
function gridLines({ x, y, cell, size, regions, thin = LINE, thick = FG, thickWidth = 4 }) {
  const lines = [];
  const at = (row, col) => regions[row * size + col];
  for (let row = 0; row < size; row++) {
    for (let col = 0; col < size; col++) {
      const left = x + col * cell;
      const top = y + row * cell;
      if (col > 0) {
        const border = at(row, col - 1) !== at(row, col);
        lines.push(line(left, top, left, top + cell, border ? thick : thin, border ? thickWidth : 1.5));
      }
      if (row > 0) {
        const border = at(row - 1, col) !== at(row, col);
        lines.push(line(left, top, left + cell, top, border ? thick : thin, border ? thickWidth : 1.5));
      }
    }
  }
  lines.push(`<rect x="${x}" y="${y}" width="${cell * size}" height="${cell * size}" fill="none" stroke="${thick}" stroke-width="${thickWidth + 1}"/>`);
  return lines.join("");
}

const line = (x1, y1, x2, y2, stroke, width) =>
  `<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="${stroke}" stroke-width="${width}" stroke-linecap="square"/>`;

const svg = (body) =>
  `<svg xmlns="http://www.w3.org/2000/svg" width="${WIDTH}" height="${HEIGHT}" viewBox="0 0 ${WIDTH} ${HEIGHT}">` +
  `<rect width="${WIDTH}" height="${HEIGHT}" fill="${BG}"/>${body}</svg>`;

async function save(slug, markup) {
  const file = path.join(posts, slug, "cover.png");
  await sharp(Buffer.from(markup)).png({ compressionLevel: 9 }).toFile(file);
  console.log(`wrote ${path.relative(root, file)} (${Math.round(fs.statSync(file).size / 1024)} KB)`);
}

/** Post 1: the puzzle built against brute force, frozen part-way through the search. */
async function bruteForceCover() {
  const source = fs.readFileSync(path.join(posts, "from-brute-force-to-cdcl/engine/puzzles.ts"), "utf8");
  const clues = [...source.match(/clues: "([.0-9]{81})"/)[1]].map((digit) => (digit === "." ? 0 : Number(digit)));

  // The post's brute-force search, cell by cell, digits in order, stopped after 2 million guesses.
  const grid = [...clues];
  const blanks = grid.flatMap((digit, index) => (digit ? [] : [index]));
  const fits = (index, digit) =>
    grid.every((other, at) => {
      if (at === index || other !== digit) return true;
      const [r, c, R, C] = [Math.floor(index / 9), index % 9, Math.floor(at / 9), at % 9];
      return r !== R && c !== C && (Math.floor(r / 3) !== Math.floor(R / 3) || Math.floor(c / 3) !== Math.floor(C / 3));
    });
  let filled = 0;
  for (let guesses = 0; guesses < 2_000_000 && filled < blanks.length; ) {
    const index = blanks[filled];
    let digit = grid[index] + 1;
    for (; digit <= 9; digit++) {
      guesses++;
      if (fits(index, digit)) break;
    }
    if (digit <= 9) {
      grid[index] = digit;
      filled++;
    } else {
      grid[index] = 0;
      filled--;
    }
  }

  const cell = 86;
  const x = (WIDTH - cell * 9) / 2;
  const y = (HEIGHT - cell * 9) / 2;
  const boxes = Array.from({ length: 81 }, (_, index) => Math.floor(Math.floor(index / 9) / 3) * 3 + Math.floor((index % 9) / 3));
  const digits = grid
    .map((digit, index) => {
      if (!digit) return "";
      const clue = clues[index] !== 0;
      const cx = x + (index % 9) * cell + cell / 2;
      const cy = y + Math.floor(index / 9) * cell + cell / 2;
      return `<text x="${cx}" y="${cy}" dy="0.35em" text-anchor="middle" font-family="${FONT}" font-size="46" font-weight="${clue ? 700 : 400}" fill="${clue ? FG : ACCENT}">${digit}</text>`;
    })
    .join("");
  await save("from-brute-force-to-cdcl", svg(gridLines({ x, y, cell, size: 9, regions: boxes }) + digits));
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

/** Post 2: a real attempt at a puzzle, with a second solution still to be ruled out. */
async function oracleCover() {
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
  await save("making-puzzles-with-a-sat-oracle", svg(fills + gridLines({ x, y, cell, size, regions: attempt.regions }) + stars + others));
}

/** Post 3: a Hard puzzle from the pool, solved, its stars glowing like a small constellation. */
async function gameCover() {
  const size = 10;
  const pool = JSON.parse(fs.readFileSync(path.join(posts, "star-battle/data/hard.json"), "utf8"));
  const regions = [...pool.puzzles[0]].map((digit) => parseInt(digit, 36));
  const game = new StarBattle(size, 2, Uint8Array.from(regions));
  const [solution] = game.solutions(new Int8Array(size * size), 1).found;

  const cell = 78;
  const x = (WIDTH - cell * size) / 2;
  const y = (HEIGHT - cell * size) / 2;
  const next = random(2026);
  const dust = Array.from({ length: 260 }, () => {
    const opacity = (0.08 + next() * 0.3).toFixed(2);
    return `<circle cx="${(next() * WIDTH).toFixed(1)}" cy="${(next() * HEIGHT).toFixed(1)}" r="${(0.6 + next() * 1.4).toFixed(1)}" fill="${FG}" opacity="${opacity}"/>`;
  }).join("");
  const glow =
    `<defs><radialGradient id="glow"><stop offset="0" stop-color="${ACCENT}" stop-opacity="0.55"/>` +
    `<stop offset="1" stop-color="${ACCENT}" stop-opacity="0"/></radialGradient></defs>`;
  const stars = solution
    .map(({ row, col }) => {
      const [cx, cy] = [x + col * cell + cell / 2, y + row * cell + cell / 2];
      return `<circle cx="${cx}" cy="${cy}" r="${cell * 0.75}" fill="url(#glow)"/><path d="${starPath(cx, cy, cell * 0.26)}" fill="#fffbe8"/>`;
    })
    .join("");
  const lines = gridLines({ x, y, cell, size, regions, thin: "#353535", thick: "#6b6b6b", thickWidth: 3 });
  await save("star-battle", svg(glow + dust + lines + stars));
}

await init({ module_or_path: fs.readFileSync(path.join(root, "src/wasm/star-battle/star_battle_wasm_bg.wasm")) });
await bruteForceCover();
await oracleCover();
await gameCover();
