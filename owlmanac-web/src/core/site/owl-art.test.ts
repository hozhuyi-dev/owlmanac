import { describe, expect, it } from "vitest";
import { artFor, artSvg, artUrl, firstQuip, owlMarkSvg, POOLS, QUIPS, quipFor, type ArtName, type Pool } from "./owl-art";

const pools = Object.keys(POOLS) as Pool[];

describe("quips and their drawings", () => {
  it.each(pools)("pairs every %s quip with a drawing from its own pool", (pool) => {
    for (const quip of QUIPS[pool]) {
      expect(POOLS[pool]).toContain(artFor(quip, pool, POOLS[pool][0]));
    }
  });

  it.each([
    ["broken", "Snapped the pencil.", "snapped"],
    ["busy", "Sharpening the pencil…", "pencil"],
    ["busy", "Thinking it over…", "owl"],
    ["busy", "On the wing…", "flying"],
    ["lost", "This page flew off.", "flying"],
    ["lost", "Hoo? Not in the almanac.", "barn"],
    ["empty", "The owl looked twice. Nothing.", "barn"],
    ["empty", "Searched every branch.", "lens"],
  ] as const)("draws what a %s quip names: %s", (pool, quip, art) => {
    expect(artFor(quip, pool, "feather")).toBe(art);
  });

  it("falls back for a quip that names nothing drawn in its pool", () => {
    // "almanac" names the book, which isn't an empty-search drawing.
    expect(artFor("Not in this almanac. Yet.", "empty", "feather")).toBe("feather");
  });

  it.each(pools)("picks a %s quip and drawing from that pool", (pool) => {
    for (let i = 0; i < 50; i++) {
      const { quip, art } = quipFor(pool);
      expect(QUIPS[pool]).toContain(quip);
      expect(POOLS[pool]).toContain(art);
    }
    expect(firstQuip(pool).quip).toBe(QUIPS[pool][0]);
  });
});

describe("drawings", () => {
  const names = [...new Set(Object.values(POOLS).flat())] as ArtName[];

  it.each(names)("draws %s on the 32-unit grid in the text colour", (name) => {
    const svg = artSvg(name, 48);
    expect(svg).toMatch(/^<svg [^>]*viewBox="0 0 32 32"[^>]*stroke="currentColor"/);
    expect(svg).toMatch(/<path /);
  });

  it("marks a drawing that draws itself", () => {
    expect(artSvg("owl", 48, { draw: true })).toContain('class="art drawing"');
    expect(artSvg("owl", 48)).toContain('class="art"');
  });

  it("makes a CSS mask of the owl that animates itself", () => {
    const url = artUrl("owl");
    expect(url).toMatch(/^url\("data:image\/svg\+xml,/);
    expect(decodeURIComponent(url)).toContain("@keyframes");
  });

  it("draws the mark filled, at the size asked", () => {
    expect(owlMarkSvg(36)).toMatch(/width="36" height="36"/);
    expect(owlMarkSvg(36)).toContain('fill="currentColor"');
  });
});
