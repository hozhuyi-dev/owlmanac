import { describe, expect, it } from "vitest";
import { useTask } from "./use-task";

/** A promise to settle by hand, so a test decides which call finishes first. */
function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((res, rej) => ((resolve = res), (reject = rej)));
  return { promise, resolve, reject };
}

describe("useTask", () => {
  it("keeps a call's result, and is pending only while it runs", async () => {
    const task = useTask(async (n: number) => n * 2);
    const run = task.start(21);
    expect(task.pending.value).toBe(true);
    await run;
    expect(task.result.value).toBe(42);
    expect(task.pending.value).toBe(false);
  });

  it("lets the latest call win when an earlier one finishes late", async () => {
    const slow = deferred<string>();
    const fast = deferred<string>();
    const task = useTask((which: "slow" | "fast") => (which === "slow" ? slow.promise : fast.promise));

    const first = task.start("slow");
    const second = task.start("fast");
    fast.resolve("fast");
    await second;
    expect(task.result.value).toBe("fast");
    expect(task.pending.value).toBe(false);

    slow.resolve("slow");
    await first;
    expect(task.result.value).toBe("fast");
  });

  it("stays pending until the latest call settles, whatever earlier ones do", async () => {
    const earlier = deferred<number>();
    const latest = deferred<number>();
    const task = useTask((n: number) => (n === 1 ? earlier.promise : latest.promise));
    const first = task.start(1);
    task.start(2);
    earlier.resolve(1);
    await first;
    expect(task.pending.value).toBe(true);
    latest.resolve(2);
    await latest.promise;
    await Promise.resolve();
    expect(task.pending.value).toBe(false);
  });

  it("keeps the last good result when a call fails, and clears the error on the next success", async () => {
    let fail = false;
    const task = useTask(async () => {
      if (fail) throw new Error("solver stopped");
      return "grid";
    });
    await task.start();
    fail = true;
    await task.start();
    expect(task.result.value).toBe("grid");
    expect(task.error.value).toBeInstanceOf(Error);

    fail = false;
    await task.start();
    expect(task.error.value).toBeUndefined();
  });

  it("ignores a failure from a call that was overtaken", async () => {
    const doomed = deferred<string>();
    const task = useTask((ok: boolean) => (ok ? Promise.resolve("fine") : doomed.promise));
    const first = task.start(false);
    await task.start(true);
    doomed.reject(new Error("too late to matter"));
    await first;
    expect(task.error.value).toBeUndefined();
    expect(task.result.value).toBe("fine");
  });
});
