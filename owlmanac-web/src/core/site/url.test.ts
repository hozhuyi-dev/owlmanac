import { afterEach, describe, expect, it, vi } from "vitest";
import { url } from "./url";

afterEach(() => vi.unstubAllEnvs());

describe("url", () => {
  it("puts a path under the site's base", () => {
    vi.stubEnv("BASE_URL", "/owlmanac");
    expect(url("posts/star-battle/")).toBe("/owlmanac/posts/star-battle/");
  });

  it("is the base itself, with its slash, for the home page", () => {
    vi.stubEnv("BASE_URL", "/owlmanac");
    expect(url()).toBe("/owlmanac/");
  });

  it("joins with one slash, whether the base or the path brings one", () => {
    vi.stubEnv("BASE_URL", "/owlmanac/");
    expect(url("/about/")).toBe("/owlmanac/about/");
    expect(url("?tag=Puzzles")).toBe("/owlmanac/?tag=Puzzles");
  });

  it("works at the root of a domain", () => {
    vi.stubEnv("BASE_URL", "/");
    expect(url("rss.xml")).toBe("/rss.xml");
  });
});
