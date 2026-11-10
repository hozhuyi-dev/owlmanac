import { lazyWorker } from "@/core/worker/lazy-worker";
import type { SatLabApi } from "./sat-lab.worker";

export const satLab = lazyWorker<SatLabApi>(
  () => new Worker(new URL("./sat-lab.worker.ts", import.meta.url), { type: "module" }),
);
