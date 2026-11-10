// The WASM crates in owlmanac-wasm that the site uses. A crate is one when its Cargo.toml says where its
// package goes, relative to owlmanac-web:
//
//   [package.metadata.owlmanac]
//   out = "src/content/posts/<slug>/wasm"   # one post's; src/wasm/<crate> once a second post uses it
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const web = fileURLToPath(new URL("..", import.meta.url));
export const workspace = join(web, "../owlmanac-wasm");

/** Each WASM crate: its name, folder, the folders of the crates it depends on (itself included), and its out. */
export function wasmCrates() {
  const crates = join(workspace, "crates");
  return readdirSync(crates)
    .map((name) => ({ name, dir: join(crates, name), manifest: manifestOf(join(crates, name)) }))
    .filter(({ manifest }) => manifest)
    .flatMap(({ name, dir, manifest }) => {
      const out = /\[package\.metadata\.owlmanac\][^[]*?^out\s*=\s*"([^"]+)"/m.exec(manifest)?.[1];
      return out ? [{ name, dir, dirs: [...pathDependencies(dir)].sort(), out: resolve(web, out) }] : [];
    });
}

/** A crate's folder and those of every crate it depends on by path, all the way down. */
function pathDependencies(dir, seen = new Set()) {
  if (seen.has(dir)) return seen;
  seen.add(dir);
  for (const [, path] of manifestOf(dir).matchAll(/\bpath\s*=\s*"([^"]+)"/g)) pathDependencies(resolve(dir, path), seen);
  return seen;
}

function manifestOf(dir) {
  const file = join(dir, "Cargo.toml");
  return existsSync(file) ? readFileSync(file, "utf8") : "";
}
