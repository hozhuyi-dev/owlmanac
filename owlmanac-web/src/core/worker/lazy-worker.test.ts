import { describe, expect, it, vi } from "vitest";
import { lazyWorker } from "./lazy-worker";

// Comlink's wrap, seen from outside: one remote per worker it's given.
vi.mock("comlink", () => ({ wrap: vi.fn((worker: unknown) => ({ remoteFor: worker })) }));

describe("lazyWorker", () => {
  it("starts no worker until the first call: islands also render at build, where Worker doesn't exist", () => {
    const spawn = vi.fn(() => ({}) as Worker);
    lazyWorker(spawn);
    expect(spawn).not.toHaveBeenCalled();
  });

  it("starts one worker on the first call, and hands back the same remote after", () => {
    const worker = {} as Worker;
    const spawn = vi.fn(() => worker);
    const remote = lazyWorker(spawn);
    const first = remote();
    expect(remote()).toBe(first);
    expect(spawn).toHaveBeenCalledTimes(1);
    expect(first).toEqual({ remoteFor: worker });
  });

  it("gives each page's module its own worker", () => {
    const a = lazyWorker(() => ({ name: "a" }) as unknown as Worker);
    const b = lazyWorker(() => ({ name: "b" }) as unknown as Worker);
    expect(a()).not.toBe(b());
  });
});
