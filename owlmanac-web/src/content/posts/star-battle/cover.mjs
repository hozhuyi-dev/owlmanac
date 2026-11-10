// The cover (npm run covers): a Hard puzzle from the pool, solved, its stars glowing like a small
// constellation. Needs the shared star-battle WASM (npm run wasm).
import fs from "node:fs";
import init, { StarBattle } from "../../../wasm/star-battle/star_battle_wasm.js";
import { ACCENT, FG, HEIGHT, WIDTH, gridLines, random, starPath, svg } from "../../../../scripts/cover-kit.mjs";

export default async function cover() {
  await init({ module_or_path: fs.readFileSync(new URL("../../../wasm/star-battle/star_battle_wasm_bg.wasm", import.meta.url)) });
  const size = 10;
  const pool = JSON.parse(fs.readFileSync(new URL("./data/hard.json", import.meta.url), "utf8"));
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
  return svg(glow + dust + lines + stars);
}
