// Draws posts' covers from their own data. A post whose folder has a cover.mjs gets a cover.png beside it:
// cover.mjs's default export returns the picture as 1600 × 900 SVG markup (helpers in scripts/cover-kit.mjs),
// and this saves it as a PNG. The covers are committed; the same inputs always draw the same pictures.
//
//   npm run covers                   # every drawn cover
//   npm run covers -- star-battle    # these posts' covers
//
// A cover drawn from a WASM engine's output needs that package built first (npm run wasm).
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import sharp from "sharp";
import { parseArgs } from "./stamp.mjs";

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const posts = path.join(root, "src/content/posts");
const { names } = parseArgs();

const drawn = fs.readdirSync(posts).filter((slug) => fs.existsSync(path.join(posts, slug, "cover.mjs")));
const unknown = names.filter((name) => !drawn.includes(name));
if (unknown.length) {
  console.error(`covers: no cover.mjs in ${unknown.join(", ")} (drawn covers: ${drawn.join(", ")})`);
  process.exit(1);
}

for (const slug of drawn) {
  if (names.length && !names.includes(slug)) continue;
  const { default: cover } = await import(pathToFileURL(path.join(posts, slug, "cover.mjs")).href);
  const file = path.join(posts, slug, "cover.png");
  await sharp(Buffer.from(await cover())).png({ compressionLevel: 9 }).toFile(file);
  console.log(`wrote ${path.relative(root, file)} (${Math.round(fs.statSync(file).size / 1024)} KB)`);
}
