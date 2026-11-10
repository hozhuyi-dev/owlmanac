import { lazyWorker } from "@/core/worker/lazy-worker";
import type { OracleApi } from "./oracle.worker";

export const oracle = lazyWorker<OracleApi>(
  () => new Worker(new URL("./oracle.worker.ts", import.meta.url), { type: "module" }),
);
