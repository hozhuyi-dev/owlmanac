import { defineCollection } from "astro:content";
import { glob } from "astro/loaders";
import { z } from "astro/zod";

// A post is a folder: src/content/posts/<slug>/index.mdx, with its components and data beside it.
const posts = defineCollection({
  loader: glob({
    pattern: "*/index.{md,mdx}",
    base: "./src/content/posts",
    generateId: ({ entry }) => entry.split("/")[0],
  }),
  schema: ({ image }) =>
    z.object({
      title: z.string(),
      description: z.string(),
      pubDate: z.coerce.date(),
      updatedDate: z.coerce.date().optional(),
      // Topics, filtered by on the home page. A small set: say what the post is about, not what it's built with.
      tags: z.array(z.string()).default([]),
      // What it's built with (languages, libraries), shown on the post.
      tools: z.array(z.string()).default([]),
      // Posts with the same series name are its parts, in pubDate order.
      series: z.string().optional(),
      // The post to read next, when the usual choice (next part, then shared topics) isn't the right one.
      next: z.string().optional(),
      cover: image().optional(),
      coverAlt: z.string().optional(),
      coverCredit: z.string().optional(),
      draft: z.boolean().default(false),
      // The post is owlmanac-notebooks/<notebook>/notebook.py, exported to the post's notebook/ and shown full width.
      notebook: z.string().optional(),
    }),
});

export const collections = { posts };
