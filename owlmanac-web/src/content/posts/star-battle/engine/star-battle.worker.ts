import { expose } from "comlink";
import init, { StarBattle } from "@/wasm/star-battle/star_battle_wasm.js";
import type { Blame, Hint, Marks, Puzzle, WhyNot } from "@/wasm/star-battle.types";

const ready = init();
const sessions = new Map<string, StarBattle>();

function session(id: string): StarBattle {
  const found = sessions.get(id);
  if (!found) throw new Error(`No Star Battle session "${id}": call open first.`);
  return found;
}

const api = {
  /** Starts a session for this puzzle, replacing any puzzle the id had before. */
  async open(id: string, { size, stars, regions }: Puzzle): Promise<void> {
    await ready;
    sessions.get(id)?.free();
    sessions.set(id, new StarBattle(size, stars, Uint8Array.from(regions)));
  },
  isSolvable: (id: string, marks: Marks): boolean => session(id).isSolvable(marks),
  blame: (id: string, marks: Marks): Blame | undefined => session(id).blame(marks),
  hint: (id: string, marks: Marks): Hint | undefined => session(id).hint(marks),
  whyNot: (id: string, marks: Marks, row: number, col: number, star: boolean): WhyNot =>
    session(id).whyNot(marks, row, col, star),
};

export type StarBattleApi = typeof api;

expose(api);
