import easyUrl from "../data/easy.json?url";
import hardUrl from "../data/hard.json?url";
import type { Puzzle } from "@/wasm/star-battle.types";

// The puzzle pool, from star-battle's `pool` binary. Each level's file is fetched only when that
// level is first played, and the puzzles a reader has played are remembered in their browser.

export type Level = "easy" | "hard";
export type PoolPuzzle = Puzzle & { level: Level; index: number };

type PoolFile = { size: number; stars: number; level: Level; puzzles: string[] };

const URLS: Record<Level, string> = { easy: easyUrl, hard: hardUrl };
const loaded = new Map<Level, Promise<PoolFile>>();

function load(level: Level): Promise<PoolFile> {
  let file = loaded.get(level);
  if (!file) {
    file = fetch(URLS[level]).then((response) => {
      if (!response.ok) throw new Error(`Couldn't load the ${level} puzzles.`);
      return response.json() as Promise<PoolFile>;
    });
    loaded.set(level, file);
  }
  return file;
}

// Browser storage can be missing or blocked; the game works without it, just without memory.
const storageKey = (level: Level) => `owlmanac.star-battle.played.${level}`;

function played(level: Level): Set<number> {
  try {
    return new Set(JSON.parse(localStorage.getItem(storageKey(level)) ?? "[]") as number[]);
  } catch {
    return new Set();
  }
}

function remember(level: Level, indices: Set<number>) {
  try {
    localStorage.setItem(storageKey(level), JSON.stringify([...indices]));
  } catch {}
}

/** A puzzle from this level the reader hasn't played, starting over once they've played them all. */
export async function nextPuzzle(level: Level): Promise<PoolPuzzle> {
  const file = await load(level);
  let seen = played(level);
  if (seen.size >= file.puzzles.length) seen = new Set();
  const unplayed = file.puzzles.map((_, index) => index).filter((index) => !seen.has(index));
  const index = unplayed[Math.floor(Math.random() * unplayed.length)];
  seen.add(index);
  remember(level, seen);
  return {
    level,
    index,
    size: file.size,
    stars: file.stars,
    regions: [...file.puzzles[index]].map((digit) => parseInt(digit, 36)),
  };
}
