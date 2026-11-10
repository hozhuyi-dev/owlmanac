import { describe, expect, it, vi } from "vitest";
import type { PDFPageProxy } from "pdfjs-dist";

// pdf.js itself needs a browser; these tests give pageLinks a page that answers like one.
vi.mock("pdfjs-dist", () => ({
  GlobalWorkerOptions: {},
  AbortException: class {},
  RenderingCancelledException: class {},
  TextLayer: class {},
  getDocument: vi.fn(),
}));
vi.mock("pdfjs-dist/build/pdf.worker.min.mjs?url", () => ({ default: "pdf.worker.js" }));

const { pageLinks, pageSize } = await import("./pdf-render");

/** An A4-ish page 600 × 800 PDF units, whose space runs up from its foot, as a PDF's does. */
function page(annotations: unknown[]): PDFPageProxy {
  const viewport = { width: 600, height: 800, convertToViewportPoint: (x: number, y: number) => [x, 800 - y] };
  return {
    getViewport: () => viewport,
    getAnnotations: async () => annotations,
  } as unknown as PDFPageProxy;
}

describe("pageLinks", () => {
  it("places a link in % of the page, measured from its top-left", async () => {
    // 60 units in from the left, 40 up from the foot; 120 wide, 20 tall.
    const links = await pageLinks(page([{ subtype: "Link", url: "https://example.com/", rect: [60, 40, 180, 60] }]));
    expect(links).toHaveLength(1);
    const [link] = links;
    expect(link.url).toBe("https://example.com/");
    expect(link.left).toBeCloseTo(10);
    expect(link.top).toBeCloseTo(92.5);
    expect(link.width).toBeCloseTo(20);
    expect(link.height).toBeCloseTo(2.5);
  });

  it("keeps email links, and leaves out links within the PDF and other annotations", async () => {
    const links = await pageLinks(
      page([
        { subtype: "Link", url: "mailto:someone@example.com", rect: [0, 0, 10, 10] },
        { subtype: "Link", dest: ["page 2"], rect: [0, 0, 10, 10] },
        { subtype: "Widget", url: "https://example.com/form", rect: [0, 0, 10, 10] },
        { subtype: "Link", url: "https://example.com/no-rect" },
      ]),
    );
    expect(links.map((l) => l.url)).toEqual(["mailto:someone@example.com"]);
  });

  it("has no links on a page without any", async () => {
    expect(await pageLinks(page([]))).toEqual([]);
  });
});

describe("pageSize", () => {
  it("is the page at scale 1, in PDF units", () => {
    expect(pageSize(page([]))).toEqual({ width: 600, height: 800 });
  });
});
