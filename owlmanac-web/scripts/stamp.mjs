// Lets a generator skip work whose inputs haven't changed. A generated folder keeps a .stamp file: a hash
// of everything that went into it (files, folders, and any other values that change the output). Used by
// scripts/wasm.mjs and scripts/notebooks.mjs; CI caches the generated folders, stamps included, so it skips
// the same work.
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";

// Never inputs: build output, caches and environments.
const SKIP = new Set(["target", "node_modules", "__pycache__", "__marimo__", ".venv", ".ruff_cache", ".git"]);

/** A hash of these files and folders (by content and path within each), and of these values. */
export function hashInputs(paths, values = []) {
  const hash = createHash("sha256");
  paths.forEach((path, i) => {
    for (const file of filesIn(path)) {
      hash.update(`${i}:${file.slice(path.length).replaceAll("\\", "/")}\0`);
      hash.update(readFileSync(file));
    }
  });
  for (const value of values) hash.update(`value:${value}\0`);
  return hash.digest("hex");
}

/** Whether `out` was made from inputs with this hash. */
export function isFresh(out, hash) {
  try {
    return readFileSync(join(out, ".stamp"), "utf8") === hash;
  } catch {
    return false;
  }
}

/** Marks `out` as made from inputs with this hash. Write it last, once the output is complete. */
export function writeStamp(out, hash) {
  writeFileSync(join(out, ".stamp"), hash);
}

/** Command-line arguments: `--force`, and any names to regenerate whether or not they changed. */
export function parseArgs(argv = process.argv.slice(2)) {
  return { force: argv.includes("--force"), names: argv.filter((arg) => !arg.startsWith("--")) };
}

function filesIn(path) {
  if (!existsSync(path)) return [];
  if (!statSync(path).isDirectory()) return [path];
  return readdirSync(path)
    .filter((entry) => !SKIP.has(entry))
    .sort()
    .flatMap((entry) => filesIn(join(path, entry)));
}
