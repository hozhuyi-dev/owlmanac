// Serves each post's generated notebook/ (see scripts/notebooks.mjs) at /<base>/posts/<slug>/notebook/,
// from the post folder in dev and copied into the build, so nothing a post needs lives in public/.
import { cpSync, createReadStream, existsSync, readdirSync, statSync } from "node:fs";
import { extname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";

const posts = fileURLToPath(new URL("../src/content/posts/", import.meta.url));

const TYPES = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript",
  ".mjs": "text/javascript",
  ".css": "text/css",
  ".json": "application/json",
  ".webmanifest": "application/manifest+json",
  ".wasm": "application/wasm",
  ".woff2": "font/woff2",
  ".ttf": "font/ttf",
  ".png": "image/png",
  ".ico": "image/x-icon",
  ".svg": "image/svg+xml",
  ".parquet": "application/octet-stream",
};

/** @returns {import("astro").AstroIntegration} */
export default function notebooks() {
  let route = /$^/;
  return {
    name: "owlmanac-notebooks",
    hooks: {
      "astro:config:setup": ({ config }) => {
        // Astro's dev server strips the base before middleware sees the URL, so match with or without it.
        const base = config.base.replace(/\/$/, "");
        route = new RegExp(`^(?:${base})?/posts/([^/]+)/notebook/(.*)$`);
      },
      "astro:server:setup": ({ server }) => {
        server.middlewares.use((req, res, next) => {
          const match = route.exec(decodeURIComponent((req.url ?? "").split("?")[0]));          if (!match) return next();
          const root = join(posts, match[1], "notebook");
          let file = normalize(join(root, match[2] || "index.html"));
          if (!file.startsWith(root)) return next();
          if (existsSync(file) && statSync(file).isDirectory()) file = join(file, "index.html");
          if (!existsSync(file)) return next();
          res.setHeader("Content-Type", TYPES[extname(file)] ?? "application/octet-stream");
          createReadStream(file).pipe(res);
        });
      },
      // Copies a notebook only for posts that were built, so a draft's notebook never ships.
      "astro:build:done": ({ dir, logger }) => {
        const out = fileURLToPath(dir);
        for (const slug of readdirSync(posts)) {
          const notebook = join(posts, slug, "notebook");
          const page = join(out, "posts", slug);
          if (!existsSync(notebook) || !existsSync(join(page, "index.html"))) continue;
          cpSync(notebook, join(page, "notebook"), { recursive: true, filter: (src) => !src.endsWith(".gitignore") });
          logger.info(`copied ${slug}/notebook`);
        }
      },
    },
  };
}
