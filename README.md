# Owlmanac 🦉

An almanac kept by a night owl: interactive posts on data, code and the sky. An owl sees in the dark; an almanac is the book of tables that says where the stars will be. Each post is a question looked into at night, with the data and charts there for readers to check. Live at <https://hozhuyi-dev.github.io/owlmanac/>.

This README is the memory, for me and for AI assistants. Keep it short; record lasting decisions in the right section. Which pipelines each post uses, and how to regenerate them, is in [POSTS.md](POSTS.md).

## Design principles

1. **A post is a folder.** Its prose, components and data live together in `owlmanac-web/src/content/posts/<slug>/`. Nothing a post needs goes in `public/`.
2. **Producers make, posts consume.** Two folders produce what posts need, each with its own README: `owlmanac-notebooks` (marimo: a notebook per post that makes its data or is the post itself, and notebooks that only explore) and `owlmanac-wasm` (Rust: WASM engines, and native tools that write data). A post mixes whatever it needs; each generated thing has one owner and one command to regenerate it, listed in [POSTS.md](POSTS.md). No server runs Python or Rust: readers' browsers run the WASM, query the data, and draw the charts (Observable Plot), and run a notebook post's Python (Pyodide).
3. **One chart, both themes.** Theme colours and fonts are tokens on `:root` in `global.css`, each colour a `light-dark(light, dark)` pair. The site is dark by default; the header's theme switch sets `data-theme` on `<html>` to `light`, or to `auto` to follow the reader's system. Where one theme must compromise, dark wins. Charts take axes and text from `currentColor` and tooltips from `--plot-background`, never a light and a dark copy. Continuous colour scales (turbo, viridis) are chosen per chart; series take muted colours in order from `core/plot/palette.ts`, checked against both themes' backgrounds.
4. **Figures hold still.** An interactive figure takes its final size on first paint, before data or WASM arrives, at any width. A chart sets `--chart-height` to the height it draws at and shows its axes at once; a table renders every row with fixed column widths and tabular figures; one status line (`.figure-status`), always present, carries loading, progress and errors. Nothing appears, disappears or changes width as results come in, and nothing overflows a phone's screen.
5. **Static until it can't be.** GitHub Pages serves everything. A folder for infra or a backend arrives only with the first post that needs it (see Later).

## Tech stack

| Folder | What | Stack |
| --- | --- | --- |
| `owlmanac-web` | The site | Astro, MDX, Vue islands, Observable Plot, DuckDB-WASM |
| `owlmanac-notebooks` | Producer: posts' data, and notebook posts; see its [README](owlmanac-notebooks/README.md) | marimo, uv, Polars, Altair, DuckDB, Astropy |
| `owlmanac-wasm` | Producer: WASM engines, and native tools that write data; see its [README](owlmanac-wasm/README.md) | Rust, wasm-bindgen, Web Workers |

## Layout

```
README.md   this file: the memory
POSTS.md    each post's producers and regenerate commands
CLAUDE.md   points here
owlmanac-web/
  src/content/posts/<slug>/   the post is the feature: index.mdx, its Vue components, cover image, and
                              data/      committed data files, from one documented command
                              wasm/      a generated WASM package used only by this post (not committed)
                              notebook/  a notebook post's generated marimo export (not committed)
  src/wasm/<crate>/           a generated WASM package two or more posts share (not committed)
  src/content.config.ts       post frontmatter: title, description, pubDate, updatedDate, tags, cover, coverAlt, coverCredit, draft, notebook
  scripts/covers.mjs          draws posts' covers from their own data (npm run covers)
  scripts/notebooks.mjs       exports notebook posts into their notebook/ (npm run notebooks)
  integrations/notebooks.mjs  serves each notebook/ in dev, and copies it into the build for published posts
  src/pages/                  routes only: home, posts/[slug], about, rss.xml, 404
  src/layouts/                BaseLayout (<html>, header, footer), PostLayout (article header, cover, contents),
                              NotebookLayout (title, then the notebook full width, grown to its height, and a
                              toast while Python loads)
  src/components/site/        the site's own: PostCard, TableOfContents, ThemeSwitch, Icon
  src/components/post/        what MDX uses across posts: Figure (numbered captions)
  src/core/site/              build time: posts.ts, url.ts (base-aware links)
  src/core/duckdb/            browser: duckdb.ts (one database per page, opened by the first query)
  src/core/plot/              browser: use-plot.ts (draw at the container's width), palette.ts (series colours)
  src/core/worker/            browser: lazy-worker.ts (one Comlink worker per page, started by the first call)
  src/core/async/             browser: use-task.ts (pending, error, latest call wins)
  src/styles/global.css       theme tokens, prose, charts
owlmanac-notebooks/           uv project (marimo pinned); its README holds the notebook conventions
  <slug>/notebook.py          a post's marimo notebook: makes its data, or is the post itself
  <slug>/data/                cached query results, so notebooks run offline (and in the browser)
  notebook.css                the site's fonts and theme tokens, mapped onto marimo's
owlmanac-wasm/                Cargo workspace; its README and each crate's hold their design notes
.github/workflows/deploy.yml  builds owlmanac-wasm and owlmanac-web, deploys to GitHub Pages on push to main
```

## Writing a post

1. Make the post's `data/` with whichever producer fits: a marimo notebook in `owlmanac-notebooks/<slug>/notebook.py`, or a Rust binary in `owlmanac-wasm`. Write only what the charts read. Add the post's row to [POSTS.md](POSTS.md). Or make the notebook the post: give the post folder an `index.md` with frontmatter only and `notebook: <slug>`, and the page is that notebook (see the notebooks [README](owlmanac-notebooks/README.md)); steps 2–4 don't apply.
2. Write `index.mdx`, and put each chart or interactive figure beside it as a Vue component.
3. Small data (up to a few thousand rows) is JSON, imported into the component: `import strategies from "./data/strategies.json"`. Big or SQL-shaped data is parquet, read through DuckDB: `import stars from "./data/stars.parquet?url"`, passed to a chart that runs `query(\`... FROM ${await parquet(src)}\`)` from `@/core/duckdb/duckdb`. DuckDB-WASM is tens of megabytes, so a post pays for it only when it needs it.
4. A WASM engine runs in a worker beside the post (`engine/`), built from its crate by an `npm run wasm:<name>` script. Use figures as `<Figure caption="…"><Chart client:visible /></Figure>`; `client:visible` means nothing heavy downloads until a figure scrolls into view.
5. `draft: true` shows the post only in `npm run dev`.
6. Posts are listed newest first by `pubDate`. Give it a time when posts share a day, in UTC and around midday so the shown date is the same everywhere: `pubDate: 2026-11-10T09:00:00Z`.
7. A cover is 1600 × 900 (cards crop to 16:9), with `coverAlt`. A photo needs `coverCredit`; a cover drawn from the post's own data, by `scripts/covers.mjs`, doesn't.

## Commands

```sh
cd owlmanac-web && npm run dev                          # http://localhost:4321/owlmanac/
cd owlmanac-web && npm run build && npm run preview     # the production build
cd owlmanac-notebooks && uv run marimo edit <slug>/notebook.py  # write a notebook
cd owlmanac-web && npm run notebooks                    # export notebook posts (dev and build run it first)
cd owlmanac-wasm && cargo test --workspace              # the Rust crates
```

Each post's regenerate commands are in [POSTS.md](POSTS.md).

## Conventions

- The look follows the old code-and-conundrums site: Inter and JetBrains Mono (Fontsource), a teal accent (`#0784b5`, `#9bd4e4` in dark), a charcoal dark mode, a hero and card grid on the home page, a sticky table of contents on posts.
- Links go through `url()`: the site is served under `/owlmanac/`. Moving to an org site or a custom domain changes only `site` and `base` in `astro.config.mjs`.
- Cast int64 columns in SQL (`::DOUBLE`): DuckDB returns them as BigInt.
- Parquet files are never inlined as data URIs (`assetsInlineLimit`), so DuckDB always fetches a URL.
- One environment: local dev and GitHub Pages. No secrets, so no `.env`.
- `core/` and `components/` never import a post. Everything starts in its post, and moves out when a second post needs it. The same goes for WASM: a package starts in its post's `wasm/` and moves to `src/wasm/<crate>/` when a second post uses it.
- `.astro` unless it needs JS in the browser; then `.vue`, as an island.
- Each island is its own Vue app, so islands share state through module-level singletons, not `provide/inject`. Islands also render at build, where `Worker` doesn't exist: create workers on first use, never at import.
- No TanStack Query. Results are deterministic and sessions stateful, so posts use a small latest-wins task helper; Comlink carries worker calls.
- WASM packages are generated by `npm run wasm`, which runs every `wasm:<name>` script and which `npm run dev` and `npm run build` run first, so both need Rust and wasm-pack. They aren't committed.
- Notebook posts are exported by `npm run notebooks`, which `npm run dev` and `npm run build` also run first, so both need uv. A draft's export is never copied into the build.

## Later, only when needed

- **`owlmanac-infra/`** (CDK, TypeScript): S3 + CloudFront for datasets past GitHub's 100 MB file limit, read by DuckDB-WASM with range requests (CORS exposing `Content-Range`).
- **`owlmanac-lambda/`** (Go): only for what must hold a secret or state, e.g. a Claude-powered feature or a newsletter.
- Comments: giscus (GitHub Discussions), no backend.

## Now

Nothing in progress. Mapping the Milky Way is now a notebook post, the first; it replaced the MDX version (still in git history).
