// Builds each WASM crate the site uses (scripts/crates.mjs) into its package with wasm-pack, skipping any
// whose inputs haven't changed since its last build (scripts/stamp.mjs): its source, the crates it depends
// on, the workspace's Cargo.toml and Cargo.lock, and this script. Generated, not committed.
// Run by `npm run wasm`, which `npm run dev` and `npm run build` run first. Needs Rust and wasm-pack.
//
//   npm run wasm                        # build what changed
//   npm run wasm -- star-battle-wasm    # rebuild these crates, changed or not
//   npm run wasm -- --force             # rebuild every crate
import { spawnSync } from "node:child_process";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { wasmCrates, web, workspace } from "./crates.mjs";
import { hashInputs, isFresh, parseArgs, writeStamp } from "./stamp.mjs";

const { force, names } = parseArgs();
const crates = wasmCrates();
const unknown = names.filter((name) => !crates.some((crate) => crate.name === name));
if (unknown.length) {
  console.error(`wasm: no WASM crate named ${unknown.join(", ")} (have: ${crates.map((c) => c.name).join(", ")})`);
  process.exit(1);
}

for (const crate of crates) {
  if (names.length && !names.includes(crate.name)) continue;
  const args = ["build", crate.dir, "--target", "web", "--release", "--out-dir", crate.out];
  const inputs = [...crate.dirs, join(workspace, "Cargo.toml"), join(workspace, "Cargo.lock"), fileURLToPath(import.meta.url)];
  const hash = hashInputs(inputs, [args.slice(2, 5).join(" ")]);
  const where = relative(web, crate.out).replaceAll("\\", "/");
  if (!force && !names.length && isFresh(crate.out, hash)) {
    console.log(`wasm: ${crate.name} is up to date in ${where}`);
    continue;
  }
  console.log(`wasm: building ${crate.name} into ${where}`);
  const run = spawnSync("wasm-pack", args, { stdio: "inherit" });
  if (run.status !== 0) process.exit(run.status ?? 1);
  writeStamp(crate.out, hash);
}
