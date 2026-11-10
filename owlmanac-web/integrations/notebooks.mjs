// Serves each post's generated notebook/ (see scripts/notebooks.mjs) at /<base>/posts/<slug>/notebook/,
// and marimo's shared runtime at /<base>/notebook-runtime/<version>/: from where they're generated in dev,
// copied into the build, so nothing a post needs lives in public/.
import { cpSync, createReadStream, existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { extname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";

const posts = fileURLToPath(new URL("../src/content/posts/", import.meta.url));
const runtimes = fileURLToPath(new URL("../notebook-runtime/", import.meta.url));
// The generator's own files, never published.
const PRIVATE = new Set([".gitignore", ".stamp", ".runtime", ".worker"]);
const published = (src) => !PRIVATE.has(src.split(/[\\/]/).pop());

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
  let notebookRoute = /$^/;
  let runtimeRoute = /$^/;
  return {
    name: "owlmanac-notebooks",
    hooks: {
      "astro:config:setup": ({ config }) => {
        // Astro's dev server strips the base before middleware sees the URL, so match with or without it.
        const base = config.base.replace(/\/$/, "");
        notebookRoute = new RegExp(`^(?:${base})?/posts/([^/]+)/notebook/(.*)$`);
        runtimeRoute = new RegExp(`^(?:${base})?/notebook-runtime/(.*)$`);
      },
      "astro:server:setup": ({ server }) => {
        server.middlewares.use((req, res, next) => {
          const path = decodeURIComponent((req.url ?? "").split("?")[0]);
          const notebook = notebookRoute.exec(path);
          const runtime = runtimeRoute.exec(path);
          const [root, rest] = notebook ? [join(posts, notebook[1], "notebook"), notebook[2]] : runtime ? [runtimes, runtime[1]] : [];
          if (!root) return next();
          let file = normalize(join(root, rest || "index.html"));
          if (!file.startsWith(root)) return next();
          if (existsSync(file) && statSync(file).isDirectory()) file = join(file, "index.html");
          if (!existsSync(file)) return next();
          res.setHeader("Content-Type", TYPES[extname(file)] ?? "application/octet-stream");
          createReadStream(file).pipe(res);
        });
      },
      // Copies a notebook only for posts that were built, so a draft's notebook never ships, and the marimo
      // runtimes those notebooks use, so an old version no post needs any more is left behind.
      "astro:build:done": ({ dir, logger }) => {
        const out = fileURLToPath(dir);
        const used = new Set();
        for (const slug of readdirSync(posts)) {
          const notebook = join(posts, slug, "notebook");
          const page = join(out, "posts", slug);
          if (!existsSync(notebook) || !existsSync(join(page, "index.html"))) continue;
          cpSync(notebook, join(page, "notebook"), { recursive: true, filter: published });
          logger.info(`copied ${slug}/notebook`);
          const runtime = join(notebook, ".runtime");
          if (existsSync(runtime)) used.add(readFileSync(runtime, "utf8").trim());
        }
        for (const version of used) {
          const runtime = join(runtimes, version);
          if (!existsSync(runtime)) throw new Error(`notebook-runtime/${version} is missing: run npm run notebooks -- --force`);
          cpSync(runtime, join(out, "notebook-runtime", version), { recursive: true, filter: published });
          logger.info(`copied notebook-runtime/${version}`);
        }
      },
    },
  };
}
