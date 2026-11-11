import type { MarkdownHeading } from "astro";

// A notebook post's sections and subsections, read from its export (notebook/index.html) at build time, so
// its contents are in the page from the first paint. The export runs the notebook (--execute) and embeds
// each cell's rendered output, Markdown headings and their ids included, as escaped JSON in the page; the
// same ids appear when the notebook renders in the browser.

const HEADING = /<h([23])\b[^>]*\bid="([^"]+)"[^>]*>([\s\S]*?)<\/h\1>/g;
const ENTITIES: Record<string, string> = { amp: "&", lt: "<", gt: ">", quot: '"', apos: "'", "#39": "'", "#x27": "'" };

/** The h2 and h3 headings in a notebook export, in order, each once. */
export function notebookHeadings(html: string): MarkdownHeading[] {
  // Undo the JSON string escapes the outputs are embedded with: < for <, \" for ", and so on.
  const text = html.replace(/\\u([0-9a-fA-F]{4})/g, (_, hex) => String.fromCharCode(parseInt(hex, 16))).replace(/\\(["\\/])/g, "$1");
  const seen = new Set<string>();
  const headings: MarkdownHeading[] = [];
  for (const [, depth, slug, inner] of text.matchAll(HEADING)) {
    if (seen.has(slug)) continue;
    seen.add(slug);
    const label = inner
      .replace(/<[^>]*>/g, "")
      .replace(/&(amp|lt|gt|quot|apos|#39|#x27);/g, (_, name: string) => ENTITIES[name])
      .replace(/\s+/g, " ")
      .trim();
    headings.push({ depth: Number(depth), slug, text: label });
  }
  return headings;
}
