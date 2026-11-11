import { describe, expect, it } from "vitest";
import { notebookHeadings } from "./notebook-headings";

// As marimo embeds a cell's output in an export: HTML inside a JSON string.
const embedded = (html: string) => JSON.stringify(html).replace(/</g, "\\u003C").replace(/>/g, "\\u003E");

describe("notebookHeadings", () => {
  it("reads sections and subsections from the outputs embedded in an export", () => {
    const html = `<script>{"outputs":[${embedded('<span class="markdown"><h2 id="introduction">Introduction</h2><p>Text</p><h3 id="first-glimpse">First Glimpse</h3></span>')}]}</script>`;
    expect(notebookHeadings(html)).toEqual([
      { depth: 2, slug: "introduction", text: "Introduction" },
      { depth: 3, slug: "first-glimpse", text: "First Glimpse" },
    ]);
  });

  it("keeps a heading's plain text, without its markup or entities", () => {
    const html = embedded('<h2 id="stars-dust">Stars &amp; <em>dust</em></h2>');
    expect(notebookHeadings(html)).toEqual([{ depth: 2, slug: "stars-dust", text: "Stars & dust" }]);
  });

  it("lists each heading once, and leaves out titles and deeper headings", () => {
    const html = embedded('<h1 id="title">Title</h1><h2 id="a">A</h2><h4 id="detail">Detail</h4>') + embedded('<h2 id="a">A</h2>');
    expect(notebookHeadings(html)).toEqual([{ depth: 2, slug: "a", text: "A" }]);
  });

  it("finds nothing in a page without headings", () => {
    expect(notebookHeadings("<html><body></body></html>")).toEqual([]);
  });
});
