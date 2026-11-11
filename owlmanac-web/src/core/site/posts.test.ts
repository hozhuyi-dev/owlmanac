import { describe, expect, it, vi } from "vitest";
import { formatDay, haunts, nextRead, readingTime, seriesOf, topics, type Post } from "./posts";

// posts.ts reads the collection only in getPosts, which these tests don't call.
vi.mock("astro:content", () => ({ getCollection: vi.fn() }));

/** A post with just what the helpers read. */
function post(id: string, date: string, data: Partial<Post["data"]> = {}): Post {
  return { id, data: { title: id, tags: [], tools: [], pubDate: new Date(date), ...data } } as unknown as Post;
}

// Newest first, as getPosts returns them.
const third = post("third", "2026-11-10T10:00:00Z", { series: "SAT", tags: ["Algorithms", "Puzzles"] });
const second = post("second", "2026-11-10T09:00:00Z", { series: "SAT", tags: ["Algorithms", "Puzzles"] });
const first = post("first", "2026-11-10T08:00:00Z", { series: "SAT", tags: ["Algorithms", "Puzzles"] });
const graphs = post("graphs", "2026-06-01T12:00:00Z", { tags: ["Algorithms"] });
const sky = post("sky", "2025-11-05T12:00:00Z", { tags: ["Astronomy", "Data"] });
const posts = [third, second, first, graphs, sky];

describe("seriesOf", () => {
  it("lists a series' parts oldest first, whatever order the posts come in", () => {
    const series = seriesOf(second, posts);
    expect(series?.parts.map((p) => p.id)).toEqual(["first", "second", "third"]);
    expect(series?.index).toBe(1);
  });

  it("is undefined for a post in no series", () => {
    expect(seriesOf(sky, posts)).toBeUndefined();
  });
});

describe("nextRead", () => {
  it("is the series' next part", () => {
    expect(nextRead(first, posts)).toEqual({ post: second, inSeries: true });
  });

  it("after the last part, is the post sharing most topics outside the series", () => {
    expect(nextRead(third, posts)).toEqual({ post: graphs, inSeries: false });
  });

  it("with no topics shared, is the newest other post", () => {
    expect(nextRead(sky, posts)?.post).toBe(third);
  });

  it("follows `next` over everything else", () => {
    const pointed = post("pointed", "2026-01-01T12:00:00Z", { series: "SAT", next: "sky" });
    expect(nextRead(pointed, [...posts, pointed])).toEqual({ post: sky, inSeries: false });
  });

  it("ignores a `next` naming no post", () => {
    const lost = post("lost", "2026-01-01T12:00:00Z", { next: "nowhere", tags: ["Astronomy"] });
    expect(nextRead(lost, [...posts, lost])?.post).toBe(sky);
  });

  it("is undefined when there's nothing else to read", () => {
    expect(nextRead(sky, [sky])).toBeUndefined();
  });
});

describe("topics", () => {
  it("lists every topic once, in alphabetical order, with its number of posts", () => {
    expect(topics(posts)).toEqual([
      { name: "Algorithms", count: 4 },
      { name: "Astronomy", count: 1 },
      { name: "Data", count: 1 },
      { name: "Puzzles", count: 3 },
    ]);
  });
});

describe("formatDay", () => {
  it("shows the UTC day, so a post reads the same date everywhere", () => {
    expect(formatDay(new Date("2026-11-10T23:30:00Z"))).toBe("10 Nov");
  });
});

describe("readingTime", () => {
  it("counts 220 words a minute, and never less than a minute", () => {
    expect(readingTime("word ".repeat(660))).toBe("3 min read");
    expect(readingTime("short")).toBe("1 min read");
  });
});

describe("haunts", () => {
  const game = post("game", "2026-01-01T12:00:00Z", {
    title: "Star Battle",
    haunts: [
      { group: "Puzzles", id: "play", action: "Play" },
      { group: "Tools", id: "make", label: "Puzzle maker", action: "Make" },
    ],
  });
  const other = post("other", "2026-02-01T12:00:00Z", {
    haunts: [{ group: "Puzzles", id: "play", label: "Nonograms", action: "Play" }],
  });

  it("groups every post's haunts, both in alphabetical order, labelled by the post's title by default", () => {
    expect(haunts([other, sky, game])).toEqual([
      {
        group: "Puzzles",
        haunts: [
          { post: other, id: "play", label: "Nonograms" },
          { post: game, id: "play", label: "Star Battle" },
        ],
      },
      { group: "Tools", haunts: [{ post: game, id: "make", label: "Puzzle maker" }] },
    ]);
  });

  it("is empty when no post has one", () => {
    expect(haunts(posts)).toEqual([]);
  });
});
