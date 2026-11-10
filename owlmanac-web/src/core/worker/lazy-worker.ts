import { wrap, type Remote } from "comlink";

/**
 * One worker per page, started by the first call. Never at import: islands also render at build,
 * where `Worker` doesn't exist. `spawn` is written out by each post because Vite needs a literal
 * `new Worker(new URL(...))` to bundle it.
 */
export function lazyWorker<Api extends object>(spawn: () => Worker): () => Remote<Api> {
  let remote: Remote<Api> | undefined;
  return () => (remote ??= wrap<Api>(spawn()));
}
