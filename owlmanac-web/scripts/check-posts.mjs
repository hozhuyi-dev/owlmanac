// Checks that the site stays within its limits as posts are added (npm run check:posts; CI runs it after
// the build):
// - A post with anything generated (data/, a drawn cover, a WASM package, a notebook) has a README.md
//   saying how each is made.
// - A post's committed files (its folder and its notebook's data/, generated folders aside) stay under
//   POST_BUDGET: git keeps every version of them for good.
// - The build (dist/, when there is one) stays under SITE_BUDGET, half of GitHub Pages' 1 GB limit.
// Past a budget, move the data out (see "Later" in the README) rather than raise it.
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative, sep } from "node:path";
import { wasmCrates, web } from "./crates.mjs";

const MB = 1024 * 1024;
const POST_BUDGET = 10 * MB;
const SITE_BUDGET = 500 * MB;
const GENERATED = new Set(["wasm", "notebook"]);

const posts = join(web, "src/content/posts");
const notebooks = join(web, "../owlmanac-notebooks");
const outs = wasmCrates().map((crate) => crate.out);
const problems = [];

for (const slug of readdirSync(posts)) {
  const folder = join(posts, slug);
  if (!statSync(folder).isDirectory()) continue;
  const index = ["index.md", "index.mdx"].map((file) => join(folder, file)).find(existsSync);
  const notebook = index && /^notebook:\s*"?([\w-]+)"?\s*$/m.exec(readFileSync(index, "utf8"))?.[1];
  const generated =
    existsSync(join(folder, "data")) ||
    existsSync(join(folder, "cover.mjs")) ||
    notebook ||
    outs.some((out) => out.startsWith(folder + sep));
  if (generated && !existsSync(join(folder, "README.md"))) {
    problems.push(`${slug}: has generated parts but no README.md saying how they're made`);
  }
  const size = sizeOf(folder, GENERATED) + (notebook ? sizeOf(join(notebooks, notebook, "data")) : 0);
  if (size > POST_BUDGET) problems.push(`${slug}: ${mb(size)} committed, over the ${mb(POST_BUDGET)} budget`);
}

const dist = join(web, "dist");
if (existsSync(dist)) {
  const size = sizeOf(dist);
  console.log(`check:posts: the build is ${mb(size)} (budget ${mb(SITE_BUDGET)})`);
  if (size > SITE_BUDGET) problems.push(`${relative(web, dist)}: ${mb(size)}, over the ${mb(SITE_BUDGET)} budget`);
}

if (problems.length) {
  for (const problem of problems) console.error(`check:posts: ${problem}`);
  process.exit(1);
}
console.log("check:posts: every post is within its limits");

/** Bytes in a file or folder, leaving out the named subfolders at its top level. */
function sizeOf(path, skip = new Set()) {
  if (!existsSync(path)) return 0;
  const stat = statSync(path);
  if (!stat.isDirectory()) return stat.size;
  return readdirSync(path)
    .filter((entry) => !skip.has(entry))
    .reduce((total, entry) => total + sizeOf(join(path, entry)), 0);
}

function mb(bytes) {
  return `${(bytes / MB).toFixed(1)} MB`;
}
