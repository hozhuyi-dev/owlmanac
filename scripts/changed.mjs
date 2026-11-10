#!/usr/bin/env node
// Which parts of the repo changed since a commit, so CI checks only what a pull request touches
// (.github/workflows/ci.yml). Writes web, wasm and notebooks (true or false) to $GITHUB_OUTPUT, or prints
// them when run by hand:
//
//   node scripts/changed.mjs <since>   # e.g. HEAD^1, the branch a pull request merges into
//
// With no commit, or one this checkout doesn't have, everything counts as changed: a run never skips what
// it can't vouch for. The workflows and these scripts change every part.

import { execFileSync } from "node:child_process";
import { appendFileSync } from "node:fs";

const SHARED = [/^\.github\/workflows\//, /^scripts\//];

const PARTS = {
  web: [/^owlmanac-web\//],
  wasm: [/^owlmanac-wasm\//],
  notebooks: [/^owlmanac-notebooks\//],
};

function changedFiles(since) {
  if (!since) return undefined;
  try {
    const out = execFileSync("git", ["diff", "--name-only", since, "HEAD"], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    });
    return out.split("\n").filter(Boolean);
  } catch {
    return undefined;
  }
}

const since = process.argv[2]?.trim();
const files = changedFiles(since);
const touches = (patterns) => files === undefined || files.some((file) => patterns.some((p) => p.test(file)));
const shared = touches(SHARED);
const result = Object.fromEntries(Object.entries(PARTS).map(([part, patterns]) => [part, shared || touches(patterns)]));

console.log(
  files === undefined
    ? `No commit to compare with${since ? ` (${since} isn't here)` : ""}: everything counts as changed.`
    : `${files.length} file(s) changed since ${since}.`,
);
const lines = Object.entries(result).map(([part, changed]) => `${part}=${changed}`);
console.log(lines.join("\n"));
if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, `${lines.join("\n")}\n`);
