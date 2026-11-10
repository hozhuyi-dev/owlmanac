// What drawn covers share: the canvas, the dark theme's colours, and drawing helpers two or more covers
// use. A post's own drawing is its cover.mjs (see scripts/covers.mjs); a helper moves here when a second
// cover needs it.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");

export const WIDTH = 1600;
export const HEIGHT = 900;
// The dark theme's tokens from global.css: covers are drawn in dark mode.
export const BG = "#2b2b2b";
export const LINE = "#4a4a4a";
export const FG = "#fafafa";
export const ACCENT = "#9bd4e4";
export const FONT = "Inter, 'Segoe UI', Arial, sans-serif";
// Series colours, read from the site's palette so the covers match its charts.
export const SERIES = [...fs.readFileSync(path.join(root, "src/core/plot/palette.ts"), "utf8").matchAll(/"(#[0-9a-f]{6})"/g)].map(
  (match) => match[1],
);

/** The whole picture: a 1600 × 900 SVG on the dark theme's background. */
export const svg = (body) =>
  `<svg xmlns="http://www.w3.org/2000/svg" width="${WIDTH}" height="${HEIGHT}" viewBox="0 0 ${WIDTH} ${HEIGHT}">` +
  `<rect width="${WIDTH}" height="${HEIGHT}" fill="${BG}"/>${body}</svg>`;

/** A small, seeded random number generator, so covers come out the same every time. */
export function random(seed) {
  let state = seed >>> 0 || 1;
  return () => {
    state ^= state << 13;
    state ^= state >>> 17;
    state ^= state << 5;
    return (state >>> 0) / 2 ** 32;
  };
}

export function starPath(cx, cy, outer) {
  const inner = outer * 0.42;
  const points = Array.from({ length: 10 }, (_, index) => {
    const radius = index % 2 ? inner : outer;
    const angle = -Math.PI / 2 + (index * Math.PI) / 5;
    return `${(cx + radius * Math.cos(angle)).toFixed(1)},${(cy + radius * Math.sin(angle)).toFixed(1)}`;
  });
  return `M${points.join("L")}Z`;
}

/** Grid lines: thin between cells, thick around the edge and between regions (or boxes). */
export function gridLines({ x, y, cell, size, regions, thin = LINE, thick = FG, thickWidth = 4 }) {
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
