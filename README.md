# Owlmanac 🦉

A night owl's almanac of interesting problems, worked through in code. An owl works best at night; an almanac is a book of tables to look things up in. Each post is a problem worked through late, with its data, code and charts there for readers to check. Live at <https://hozhuyi-dev.github.io/owlmanac/>.

This README is the memory, for me and for AI assistants. Keep it short; record lasting decisions in the right section. Which pipelines each post uses, and how to regenerate them, is in [POSTS.md](POSTS.md).

## Design principles

1. **Two jobs: find a post, and find out about me.** Everything on the site serves one of them, or goes. The home page is the index (search, topics, series); About is a summary and the resume.
2. **Elegance is restraint.** One ink, one accent, thin lines, room around everything. The fun is in the words and the drawings of the quiet moments (loading, empty, not found, errors), never in decoration; a quip sits above the plain explanation, never instead of it.
3. **Readers can check the work.** A post ships its data, and the computation runs in the reader's browser (WASM, DuckDB, Pyodide). So the site is static: no server runs Python or Rust, and a backend arrives only with the first post that needs one (see Later).
4. **A post is a folder; producers make, posts consume.** Its prose, components and data live together in `owlmanac-web/src/content/posts/<slug>/`. Two folders produce what posts need (`owlmanac-notebooks`, `owlmanac-wasm`); each generated thing has one owner and one command to regenerate it, listed in [POSTS.md](POSTS.md).
5. **Nothing jumps.** Everything takes its final size on first paint, before data, WASM or fonts arrive, at any width: a chart sets `--chart-height`, a table fixes its columns, a figure's one status line (`.figure-status`) is always there, a current label is sized for its bold self. Nothing appears, disappears or changes width as results come in, and nothing overflows a phone.
6. **Content is a file.** A post, its series and its topics are frontmatter; the About summary is `src/content/about.md`; the resume is `public/resume.pdf`. Changing any of them needs no code.

## Brand

- **The mark** is a horned owl seen from behind, perched on a branch (`OWL` in `core/site/owl-art.ts`): filled in the header and the favicon, and the same owl as one continuous line that draws itself while something loads.
- **Drawings** for quiet moments are line art on a 32-unit grid, 1.5 stroke, round ends, in `currentColor` (`ART` in `owl-art.ts`): the owl in three poses (from behind, in flight, a barn owl facing you), and the night's work (pencil, book, candle, mug, hourglass, lens, feather, branch; a snapped pencil, spilt ink and a knot for errors). Muted, except an error's, in the danger colour.
- **Quips** come in pools (`QUIPS`: busy, empty, lost, broken), deadpan and owlish, about working late on a problem; one is picked per visit, with the drawing it names. Not about the sky: the site isn't.
- **Type:** Newsreader for the wordmark, titles and headings; Inter for body text; JetBrains Mono for code.
- **Colour:** charcoal dark theme (the default) and white light theme, one teal accent. Covers on the home page are redrawn in the theme's colours (a duotone: their charcoal becomes the page, their lines the accent; inverted in light theme) and take their real colours when looked at: hovered, focused, or on a touch screen while crossing the middle of the screen.

## Tech stack

| Folder | What | Stack |
| --- | --- | --- |
| `owlmanac-web` | The site | Astro, MDX, Vue islands, Observable Plot, DuckDB-WASM, pdf.js |
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
  src/content/about.md        the About page's summary
  src/wasm/<crate>/           a generated WASM package two or more posts share (not committed)
  src/content.config.ts       post frontmatter: title, description, pubDate, updatedDate, tags, tools, series,
                              next, cover, coverAlt, coverCredit, draft, notebook
  scripts/covers.mjs          draws posts' covers from their own data (npm run covers)
  scripts/notebooks.mjs       exports notebook posts into their notebook/ (npm run notebooks)
  integrations/notebooks.mjs  serves each notebook/ in dev, and copies it into the build for published posts
  src/pages/                  routes only: home (the index), posts/[slug], about, rss.xml, 404
  src/layouts/                BaseLayout (<html>, header, footer, duotone filters, the visit's quips),
                              PostLayout (title, meta, series, contents, body, next read),
                              NotebookLayout (the same header, then the notebook full width, grown to its height)
  src/components/site/        the site's own: PostRow, PostCover, PostMeta, SeriesNav, NextRead,
                              TableOfContents, Quiet (drawing + quip + explanation), Resume, ThemeSwitch, Icon
  src/components/post/        what MDX uses across posts: Figure (numbered captions)
  src/core/site/              build time and browser: posts.ts (series, next read, topics), owl-art.ts (the mark,
                              drawings, quips), url.ts (base-aware links)
  src/core/pdf/               browser: pdf-render.ts (pdf.js, loaded only by the resume)
  src/core/duckdb/            browser: duckdb.ts (one database per page, opened by the first query)
  src/core/plot/              browser: use-plot.ts (draw at the container's width), palette.ts (series colours)
  src/core/worker/            browser: lazy-worker.ts (one Comlink worker per page, started by the first call)
  src/core/async/             browser: use-task.ts (pending, error, latest call wins)
  src/styles/global.css       theme tokens, prose, charts, covers, drawings
owlmanac-notebooks/           uv project (marimo pinned); its README holds the notebook conventions
  <slug>/notebook.py          a post's marimo notebook: makes its data, or is the post itself
  <slug>/data/                cached query results, so notebooks run offline (and in the browser)
  notebook.css                the site's fonts and theme tokens, mapped onto marimo's
owlmanac-wasm/                Cargo workspace; its README and each crate's hold their design notes
scripts/changed.mjs           which parts (web, wasm, notebooks) changed since a commit, for CI
.github/workflows/ci.yml      checks each part a pull request changes; deploys run it in full first
.github/workflows/deploy.yml  on push to main: every check, then build and deploy to GitHub Pages
```

## Writing a post

1. Make the post's `data/` with whichever producer fits: a marimo notebook in `owlmanac-notebooks/<slug>/notebook.py`, or a Rust binary in `owlmanac-wasm`. Write only what the charts read. Add the post's row to [POSTS.md](POSTS.md). Or make the notebook the post: give the post folder an `index.md` with frontmatter only and `notebook: <slug>`, and the page is that notebook (see the notebooks [README](owlmanac-notebooks/README.md)); steps 2–4 don't apply.
2. Write `index.mdx`, and put each chart or interactive figure beside it as a Vue component.
3. Small data (up to a few thousand rows) is JSON, imported into the component: `import strategies from "./data/strategies.json"`. Big or SQL-shaped data is parquet, read through DuckDB: `import stars from "./data/stars.parquet?url"`, passed to a chart that runs `query(\`... FROM ${await parquet(src)}\`)` from `@/core/duckdb/duckdb`. DuckDB-WASM is tens of megabytes, so a post pays for it only when it needs it.
4. A WASM engine runs in a worker beside the post (`engine/`), built from its crate by an `npm run wasm:<name>` script. Use figures as `<Figure caption="…"><Chart client:visible /></Figure>`; `client:visible` means nothing heavy downloads until a figure scrolls into view.
5. `tags` are topics, the home page's filters: a small set, reused (Algorithms, Puzzles, Astronomy, Data). What it's built with goes in `tools` (Rust, WebAssembly, Python). Posts sharing a `series` name are its parts, in `pubDate` order; the next part is the post's next read, else the post sharing most topics outside the series. `next: <slug>` overrides it.
6. `draft: true` shows the post only in `npm run dev`.
7. Posts are listed newest first by `pubDate`. Give it a time when posts share a day, in UTC and around midday so the shown date is the same everywhere: `pubDate: 2026-11-10T09:00:00Z`.
8. A cover is 1600 × 900 (cards crop to 16:9), with `coverAlt`. A photo needs `coverCredit`; a cover drawn from the post's own data, by `scripts/covers.mjs`, doesn't. Drawn covers use the dark theme's charcoal (`#2b2b2b`) as their background, which the home page's duotone turns into the page itself.

## Commands

```sh
cd owlmanac-web && npm run dev                          # http://localhost:4321/owlmanac/
cd owlmanac-web && npm run build && npm run preview     # the production build
cd owlmanac-notebooks && uv run marimo edit <slug>/notebook.py  # write a notebook
cd owlmanac-web && npm run notebooks                    # export notebook posts (dev and build run it first)
```

Each post's regenerate commands are in [POSTS.md](POSTS.md).

## Checks

Every pull request into `main` runs CI ([ci.yml](.github/workflows/ci.yml)) on the parts it changes ([scripts/changed.mjs](scripts/changed.mjs)); a change to the WASM crates or the notebooks checks the site too, as it builds from them. A push to `main` runs every check before it builds and deploys ([deploy.yml](.github/workflows/deploy.yml)): nothing deploys past a failing check. The same checks, by hand:

```sh
cd owlmanac-web && npm run typecheck && npm run lint && npm test   # astro check, ESLint, Vitest
cd owlmanac-wasm && cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
cd owlmanac-notebooks && uv run ruff check . && uv run ruff format --check . && uv run marimo check .
```

- **Web:** ESLint takes the recommended rules (TypeScript, Astro, Vue's essential ones) and enforces that `core/` and `components/` never import a post. Vitest tests sit beside the file they test (`x.ts` → `x.test.ts`) and cover what runs without a browser: everything in `core/` that doesn't need one.
- **Rust:** rustfmt, and clippy with warnings as errors.
- **Python:** Ruff with its defaults, but for two marimo conventions it reads as mistakes (a cell's closing `return`, and its last bare expression, which marimo displays); `marimo check` checks each notebook is a valid app.
- To make a check required before merging, add it under the repository's branch protection for `main`.

## Conventions

- **One chart, both themes.** Theme colours and fonts are tokens on `:root` in `global.css`, each colour a `light-dark(light, dark)` pair. The site is dark by default; the header's theme switch sets `data-theme` on `<html>` to `light`, or to `auto` to follow the reader's system. Where one theme must compromise, dark wins. Charts take axes and text from `currentColor` and tooltips from `--plot-background`, never a light and a dark copy. Continuous colour scales (turbo, viridis) are chosen per chart; series take muted colours in order from `core/plot/palette.ts`, checked against both themes' backgrounds.
- **Quiet moments use the drawings and quips in `owl-art.ts`.** A page's own is `<Quiet pool="…">` with the plain explanation as its slot. CSS shows two without any script in the island: a chart's empty placeholder (`.chart:empty`) draws the owl over a busy quip, and a failed `.figure-status` opens with a broken quip; BaseLayout picks both per visit.
- A post's column is `--measure` (44rem, about 75 characters of body text). On wide screens its contents sit a fixed 4rem beside it and the pair is centred, so spare width falls to the margins, never between them.
- The header sticks on wide screens only. On a phone it scrolls away and a post's contents become a bar stuck to the top (sections only, sideways, with a progress line); `scroll-padding-top` follows.
- Fonts load through Astro's Fonts API (`fonts` in `astro.config.mjs`, from the Fontsource packages, latin only): BaseLayout's `<Font>` tags inline the `@font-face` rules, preload body and heading fonts, and add fallbacks sized to match, with `font-display: fallback`, so text never visibly swaps font or reflows. The notebook export copies the same files (`scripts/notebooks.mjs`, `notebook.css`).
- PDFs are drawn into the page with pdf.js (`core/pdf`, as in bizzy), never a browser's own viewer.
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
- Full-text search (Pagefind) once titles, descriptions and topics aren't enough to find a post.
- A notebook post's contents bar, read from the notebook's headings (same origin).

## Now

The redesign is in: the owl mark, the home page as an index with search and topics, series and next read, the phone contents bar, quips and drawings, and the About page drawing the resume with pdf.js.
