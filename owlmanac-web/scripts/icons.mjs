// Draws the site's PNG icons from public/favicon.svg, for the browsers that don't show an SVG favicon:
// every browser on iOS (Safari and Chrome alike) and its home screen. Run after changing the mark.
//
//   npm run icons
//
// The SVG favicon is the owl alone, inked for the system's theme. A PNG can't follow the theme, so these
// are the mark in the site's dark-theme colours on a tile, which reads on light and dark tab bars alike:
// - favicon-96.png       the tab icon: a rounded tile, the owl filling most of it
// - apple-touch-icon.png 180 × 180, the home-screen icon: a square tile (iOS rounds it), the owl inset
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import sharp from "sharp";

const publicDir = fileURLToPath(new URL("../public/", import.meta.url));
const svg = readFileSync(`${publicDir}favicon.svg`, "utf8");
const BG = "#2b2b2b";
const INK = "#fafafa";

// The mark's paths, inked, without the SVG's own theme styles.
const found = svg.match(/<path[\s\S]*?\/>/g);
if (!found) throw new Error("icons: no paths in public/favicon.svg");
const paths = found.map((path) => path.replace("<path", `<path stroke="${INK}" fill="${path.includes('class="branch"') ? "none" : INK}"`)).join("");

/** The 64-unit mark on a tile, at `inset` units from each edge, with corners of `radius` units. */
function tile(inset, radius) {
  const scale = (64 - 2 * inset) / 64;
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">
  <rect width="64" height="64" rx="${radius}" fill="${BG}"/>
  <g transform="translate(${inset} ${inset}) scale(${scale})">${paths}</g>
</svg>`;
}

const icons = [
  { file: "favicon-96.png", size: 96, svg: tile(5, 12) },
  { file: "apple-touch-icon.png", size: 180, svg: tile(11, 0) },
];
for (const { file, size, svg: markup } of icons) {
  await sharp(Buffer.from(markup), { density: (72 * size) / 64 })
    .resize(size, size)
    .png({ compressionLevel: 9, palette: true })
    .toFile(`${publicDir}${file}`);
  console.log(`icons: wrote public/${file}`);
}
