import { describe, expect, it } from "vitest";
import { SERIES } from "./palette";

// The theme backgrounds (global.css): --bg's dark and light values.
const DARK = "#2b2b2b";
const LIGHT = "#f6f4ef";

/** WCAG contrast ratio between two hex colours. */
function contrast(a: string, b: string): number {
  const luminance = (hex: string) => {
    const [r, g, bl] = [1, 3, 5]
      .map((i) => parseInt(hex.slice(i, i + 2), 16) / 255)
      .map((v) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
    return 0.2126 * r + 0.7152 * g + 0.0722 * bl;
  };
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

describe("SERIES", () => {
  it("has six distinct colours, taken in order and never cycled", () => {
    expect(SERIES).toHaveLength(6);
    expect(new Set(SERIES).size).toBe(6);
    for (const colour of SERIES) expect(colour).toMatch(/^#[0-9a-f]{6}$/);
  });

  it("stands out from the light background, every colour", () => {
    for (const colour of SERIES) expect(contrast(colour, LIGHT)).toBeGreaterThanOrEqual(3);
  });

  it("has exactly the colours its note names below 3:1 on the dark background (indigo, terracotta, plum)", () => {
    const faint = SERIES.filter((colour) => contrast(colour, DARK) < 3);
    expect(faint).toEqual(["#3464a0", "#af5742", "#855290"]);
  });
});
