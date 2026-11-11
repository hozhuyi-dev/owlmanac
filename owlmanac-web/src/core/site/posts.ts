import { getCollection, type CollectionEntry } from "astro:content";

export type Post = CollectionEntry<"posts">;

/** Published posts, newest first; drafts show only in dev. */
export async function getPosts(): Promise<Post[]> {
  const posts = await getCollection("posts", ({ data }) => import.meta.env.DEV || !data.draft);
  return posts.sort((a, b) => b.data.pubDate.valueOf() - a.data.pubDate.valueOf());
}

export function formatDate(date: Date): string {
  return date.toLocaleDateString("en-GB", { day: "numeric", month: "long", year: "numeric" });
}

/** "10 Nov", for the home page's rows, which sit under their year. */
export function formatDay(date: Date): string {
  return date.toLocaleDateString("en-GB", { day: "numeric", month: "short", timeZone: "UTC" });
}

/** Minutes to read a post's source at 220 words a minute, imports and markup included. */
export function readingTime(body: string): string {
  const words = body.split(/\s+/).filter(Boolean).length;
  return `${Math.max(1, Math.round(words / 220))} min read`;
}

/** Every topic once, in alphabetical order, with how many posts it has: the home page's chips. */
export function topics(posts: Post[]): { name: string; count: number }[] {
  const counts = new Map<string, number>();
  for (const post of posts) for (const tag of post.data.tags) counts.set(tag, (counts.get(tag) ?? 0) + 1);
  return [...counts].map(([name, count]) => ({ name, count })).sort((a, b) => a.name.localeCompare(b.name));
}

export interface Series {
  name: string;
  /** Oldest first. */
  parts: Post[];
  /** This post's place in `parts`, from 0. */
  index: number;
}

/** The series a post belongs to, with its parts in order. */
export function seriesOf(post: Post, posts: Post[]): Series | undefined {
  const name = post.data.series;
  if (!name) return undefined;
  const parts = posts
    .filter((p) => p.data.series === name)
    .sort((a, b) => a.data.pubDate.valueOf() - b.data.pubDate.valueOf());
  return { name, parts, index: parts.findIndex((p) => p.id === post.id) };
}

/**
 * What to read after a post: the one its `next` names; else the series' next part; else the post sharing
 * the most topics (outside its own series, whose earlier parts the reader has likely read), newest first;
 * else the newest other post.
 */
export function nextRead(post: Post, posts: Post[]): { post: Post; inSeries: boolean } | undefined {
  if (post.data.next) {
    const named = posts.find((p) => p.id === post.data.next);
    if (named) return { post: named, inSeries: false };
  }
  const series = seriesOf(post, posts);
  const following = series?.parts[series.index + 1];
  if (following) return { post: following, inSeries: true };
  const others = posts.filter((p) => p.id !== post.id && (!series || p.data.series !== series.name));
  const shared = (p: Post) => p.data.tags.filter((tag) => post.data.tags.includes(tag)).length;
  const best = [...others].sort((a, b) => shared(b) - shared(a))[0];
  return best ? { post: best, inSeries: false } : undefined;
}

export interface Haunt {
  post: Post;
  /** The element on the post's page it scrolls to. */
  id: string;
  /** Its name in the menu. */
  label: string;
}

/** The Haunts menu: every post's haunts by group, groups and the haunts in each in alphabetical order. */
export function haunts(posts: Post[]): { group: string; haunts: Haunt[] }[] {
  const groups = new Map<string, Haunt[]>();
  for (const post of posts) {
    for (const { group, id, label = post.data.title } of post.data.haunts ?? []) {
      groups.set(group, [...(groups.get(group) ?? []), { post, id, label }]);
    }
  }
  return [...groups]
    .map(([group, list]) => ({ group, haunts: list.sort((a, b) => a.label.localeCompare(b.label)) }))
    .sort((a, b) => a.group.localeCompare(b.group));
}
