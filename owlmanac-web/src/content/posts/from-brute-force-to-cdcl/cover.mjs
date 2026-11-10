// The cover (npm run covers): the puzzle built against brute force, frozen part-way through the search.
import fs from "node:fs";
import { ACCENT, FG, FONT, HEIGHT, WIDTH, gridLines, svg } from "../../../../scripts/cover-kit.mjs";

export default async function cover() {
  const source = fs.readFileSync(new URL("./engine/puzzles.ts", import.meta.url), "utf8");
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
  return svg(gridLines({ x, y, cell, size: 9, regions: boxes }) + digits);
}
