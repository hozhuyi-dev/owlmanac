import { lazyWorker } from "@/core/worker/lazy-worker";
import type { StarBattleApi } from "./star-battle.worker";

export const starBattle = lazyWorker<StarBattleApi>(
  () => new Worker(new URL("./star-battle.worker.ts", import.meta.url), { type: "module" }),
);
