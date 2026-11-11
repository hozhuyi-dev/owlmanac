import { describe, expect, it } from "vitest";
import {
  artFor,
  artSvg,
  artUrl,
  firstQuip,
  HOURS,
  owlMarkSvg,
  POOLS,
  QUIPS,
  quipFor,
  TIMES,
  type ArtName,
  type Pool,
} from "./owl-art";

const pools = Object.keys(POOLS.night) as Pool[];
const cases = TIMES.flatMap((time) => pools.map((pool) => [time, pool] as const));

describe("quips and their drawings", () => {
  it.each(cases)("pairs every %s %s quip with a drawing from its own pool", (time, pool) => {
    for (const quip of QUIPS[time][pool]) {
      expect(POOLS[time][pool]).toContain(artFor(quip, pool, POOLS[time][pool][0], time));
    }
  });

  it.each([
    ["night", "broken", "Snapped the pencil.", "snapped"],
    ["night", "busy", "Sharpening the pencil…", "pencil"],
    ["night", "busy", "Thinking it over…", "owl"],
    ["night", "busy", "On the wing…", "flying"],
    ["night", "busy", "Up past bedtime. Nearly there…", "candle"],
    ["night", "lost", "This page flew off.", "flying"],
    ["night", "lost", "Hoo? Not in the almanac.", "barn"],
    ["night", "empty", "The owl looked twice. Nothing.", "barn"],
    ["night", "empty", "Searched every branch.", "lens"],
    ["day", "busy", "Rubbing its eyes…", "asleep"],
    ["day", "busy", "Making the first coffee…", "mug"],
    ["day", "lost", "This page is still in bed.", "asleep"],
    ["day", "broken", "Spilt the coffee.", "mug"],
    ["day", "broken", "Woke up in a knot.", "knot"],
  ] as const)("draws what a %s %s quip names: %s", (time, pool, quip, art) => {
    expect(artFor(quip, pool, "feather", time)).toBe(art);
  });

  it("falls back for a quip that names nothing drawn in its pool", () => {
    // "almanac" names the book, which isn't an empty-search drawing.
    expect(artFor("Not in this almanac. Yet.", "empty", "feather")).toBe("feather");
    // The owl dozing isn't an error's drawing.
    expect(artFor("Dozed off mid-sum.", "broken", "ink", "day")).toBe("ink");
  });

  it.each(cases)("picks a %s %s quip and drawing from that pool", (time, pool) => {
    for (let i = 0; i < 50; i++) {
      const { quip, art } = quipFor(pool, time);
      expect(QUIPS[time][pool]).toContain(quip);
      expect(POOLS[time][pool]).toContain(art);
    }
    expect(firstQuip(pool, time).quip).toBe(QUIPS[time][pool][0]);
  });
});

describe("the header's hours", () => {
  it.each(TIMES)("has a fresh %s line for many visits: twenty or more, all different, at most 46 characters", (time) => {
    const lines = HOURS[time];
    expect(lines.length).toBeGreaterThanOrEqual(20);
    expect(new Set(lines).size).toBe(lines.length);
    for (const line of lines) {
      expect(line.length).toBeLessThanOrEqual(46);
      expect(line).toMatch(/[.?]$/);
    }
  });

  it("never says the same line by night and by day", () => {
    const night = new Set<string>(HOURS.night);
    expect(HOURS.day.filter((line) => night.has(line))).toEqual([]);
  });
});

describe("drawings", () => {
  const names = [...new Set(TIMES.flatMap((time) => Object.values(POOLS[time]).flat()))] as ArtName[];

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

  it("gives the header's mark a z z that only day shows", () => {
    expect(owlMarkSvg(36)).not.toContain("by-day");
    expect(owlMarkSvg(36, { asleep: true })).toContain('class="by-day"');
  });
});
