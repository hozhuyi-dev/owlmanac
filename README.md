# Owlmanac 🦉

An almanac kept by a night owl: interactive posts on data, code and the sky. An owl sees in the dark; an almanac is the book of tables that says where the stars will be. Each post is a question looked into at night, with the data and charts there for readers to check. Live at <https://hozhuyi-dev.github.io/owlmanac/>.

This README is the memory, for me and for AI assistants. Keep it short; record lasting decisions in the right section.

## Design principles

1. **A post is a folder.** Its prose, components and data live together in `owlmanac-web/src/content/posts/<slug>/`. Nothing a post needs goes in `public/`.
2. **Explore in Python, publish in the browser.** A marimo notebook per post queries and shapes the data and writes it into the post's `data/` as parquet. The site never runs Python: readers' browsers query the data (DuckDB-WASM) and draw the charts (Observable Plot).
3. **One chart, both themes.** Theme colours and fonts are tokens on `:root` in `global.css`, each colour a `light-dark(light, dark)` pair. The header's theme switch sets `data-theme` on `<html>`; without it the reader's system decides. Charts take axes and text from `currentColor` and tooltips from `--plot-background`, never a light and a dark copy. Colour scales that encode data (turbo, viridis) are chosen per chart.
4. **Static until it can't be.** GitHub Pages serves everything. A folder for Rust, infra or a backend arrives only with the first post that needs it (see Later).

## Tech stack

| Folder | What | Stack |
| --- | --- | --- |
| `owlmanac-web` | The site | Astro, MDX, Vue islands, Observable Plot, DuckDB-WASM |
| `owlmanac-explore` | Exploring and preparing each post's data | marimo, uv, Polars, Astropy |

## Layout

```
README.md   this file: the memory
CLAUDE.md   points here
owlmanac-web/
  src/content/posts/<slug>/   index.mdx, its Vue chart components, data/*.parquet, cover image
  src/content.config.ts       post frontmatter: title, description, pubDate, updatedDate, tags, cover, coverAlt, coverCredit, draft
  src/lib/                    duckdb.ts (one database per page, opened by the first query), plot.ts (draw at the
                              container's width), url.ts (base-aware links), posts.ts
  src/components/, src/layouts/, src/pages/   Figure (numbered captions), PostCard, TableOfContents, ThemeSwitch, Icon,
                              BaseLayout; home, posts/[slug], about, rss.xml, 404
  src/styles/global.css       theme tokens, prose, charts
owlmanac-explore/
  <slug>.py                   marimo notebook: queries, then writes the post's data/
  data/<slug>/                cached query results, so notebooks run offline
.github/workflows/deploy.yml  builds owlmanac-web and deploys to GitHub Pages on push to main
```

## Writing a post

1. Explore in `owlmanac-explore/<slug>.py`. Write only the columns the charts read, as parquet, to the post's `data/`.
2. Write `index.mdx`. Import data as `import stars from "./data/stars.parquet?url"` and pass it to a chart.
3. A chart is a Vue component beside the post: `query(\`... FROM ${await parquet(src)}\`)` from `@/lib/duckdb`, drawn by `usePlot`. Use it as `<Figure caption="…"><Chart client:visible src={stars} /></Figure>`; `client:visible` means DuckDB downloads only when a chart scrolls into view.
4. `draft: true` shows the post only in `npm run dev`.

## Commands

```sh
cd owlmanac-web && npm run dev                          # http://localhost:4321/owlmanac/
cd owlmanac-web && npm run build && npm run preview     # the production build
uv run marimo edit --sandbox owlmanac-explore/<slug>.py # explore
uv run owlmanac-explore/<slug>.py                       # regenerate a post's data
```

## Conventions

- The look follows the old code-and-conundrums site: Inter and JetBrains Mono (Fontsource), a teal accent (`#0784b5`, `#9bd4e4` in dark), a charcoal dark mode, a hero and card grid on the home page, a sticky table of contents on posts.
- Links go through `url()`: the site is served under `/owlmanac/`. Moving to an org site or a custom domain changes only `site` and `base` in `astro.config.mjs`.
- Cast int64 columns in SQL (`::DOUBLE`): DuckDB returns them as BigInt.
- Parquet files are never inlined as data URIs (`assetsInlineLimit`), so DuckDB always fetches a URL.
- One environment: local dev and GitHub Pages. No secrets, so no `.env`.

## Later, only when needed

- **`owlmanac-wasm/`**: a Cargo workspace, one crate per heavy computation, built with wasm-bindgen (`--target web`), run in a Web Worker, typed arrays in and out, SIMD. Single-threaded: threads need COOP/COEP headers, which GitHub Pages can't send.
- **`owlmanac-infra/`** (CDK, TypeScript): S3 + CloudFront for datasets past GitHub's 100 MB file limit, read by DuckDB-WASM with range requests (CORS exposing `Content-Range`).
- **`owlmanac-lambda/`** (Go): only for what must hold a secret or state, e.g. a Claude-powered feature or a newsletter.
- Comments: giscus (GitHub Discussions), no backend.

## Now

- Port the remaining post from code-and-conundrums (GitLab): Gaia DR3 Primer. High-Performance SEO Blogging described the Quarto pipeline this repo replaces, so it stays behind.
- Point the old GitLab Pages URLs here.
