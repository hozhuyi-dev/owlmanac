# owlmanac-notebooks

Owlmanac's marimo notebooks, one folder per post: `<slug>/notebook.py`, with its cached data in `<slug>/data/`. A notebook either makes a post's data, or is the post itself (a notebook post). A folder that isn't a post's slug is exploration only. Pure functions shared by notebooks would go in a `src/` package here, once there are any.

```sh
uv run marimo edit <slug>/notebook.py   # write or explore
uv run <slug>/notebook.py               # run it top to bottom: refresh caches, write data
```

It is one uv project, with marimo pinned: a notebook post's export runs this marimo's frontend, which loads the same marimo in the browser.

## Notebook posts

A post in `owlmanac-web` whose `index.md` has `notebook: <slug>` is this folder's notebook. `npm run notebooks` exports it with `marimo export html-wasm --mode run --show-code --execute` into the post's `notebook/`, and copies in `<slug>/data/` and the site's fonts; marimo's frontend goes to a runtime shared by every notebook on this marimo version (see the site's README). It re-exports only when the notebook's folder, `notebook.css`, `pyproject.toml` or `uv.lock` changed. The page shows the site's header and the post's title, then the notebook at full width.

- **Outputs come first.** `--execute` runs the notebook at export and embeds its outputs, so readers see prose, code and charts in a few seconds; inputs work once Python has loaded in the browser and rerun the cells (about half a minute on a first visit, then cached). Until then a toast from the post's layout says so. It reads marimo's own markers (pinned version): `data-connection-state="OPEN"`, then `data-testid="loading-indicator"` and "This output is stale" while the cells rerun, gone when they finish.
- **One notebook runs in two places.** Locally it queries the source (Gaia) or reads the cache; in the browser (Pyodide, `sys.platform == "emscripten"`) it can't reach the source, so `cached(name, query)` fetches `data/<name>.json` from beside the page instead. Cache what the charts read, not raw query results, so the browser does no heavy work.
- **Every package costs the reader.** Before running anything, marimo downloads each package the notebook names in an `import`, even one inside a function the browser never calls. So:
  - Packages only local runs need (Astropy, astroquery) are imported by name through `local_only("astropy.units")`, which marimo can't see.
  - No `mo.sql`: one SQL cell makes marimo load DuckDB, pandas, sqlglot and pyarrow (about 23 MB). Use Polars.
  - Caches are JSON, read with Python's `json` module: in the browser, every Polars reader (parquet, CSV, JSON) goes through pyarrow, another 10 MB.
  - Mapping the Milky Way downloads about 30 MB of Python packages on a first visit, Polars (18 MB) the largest; the export's own page and assets add about 15 MB.
- **The theme is the site's.** `notebook.css` (`css_file="../notebook.css"`) maps the site's tokens onto marimo's (`--background`, `--primary`, …) and its fonts. Notebooks use marimo's `system` theme (`[tool.marimo.display]` in `pyproject.toml`; not a script header, which would make `uv run` treat the notebook as a standalone script without the project's packages); the export adds a script that makes its dark-mode query follow the site's theme switch. Charts take the site's look from an Altair theme registered in the setup cell (`alt.theme.register("owlmanac", enable=True)`): the site's font, a transparent background, and colours checked for contrast and colour-blind separation. It sets no axis or text colours, because marimo sets those for the page's theme and a theme's would override them. Python can't tell which theme the page is in (`mo.app_meta().theme` reports "light" under `system`), so each colour serves both, and dark wins where one must give:
  - **Series** (`range.category`): indigo, brick, sage teal, mustard, plum, moss; each passes on both pages.
  - **Magnitude** (`range.ramp`, `range.heatmap`): "dusk", a muted turbo from deep indigo through plum, brick and ochre to unbleached cream. Its lightness only rises, so more always reads brighter and dense regions glow on the dark page; turbo's lightness rises and falls, drawing edges that aren't in the data.
  - **Counts over many orders of magnitude** take `COUNTS`, a log scale blended through dusk in Lab (`interpolate="lab"`), which keeps the in-between shades muted. If you ever want steps instead of a gradient, colour by a labelled step column: Vega-Lite's threshold scales draw a legend whose labels collide.
- **No notification prompts.** marimo asks for notification permission so it can say "Execution completed" when a run finishes in a background tab, which on a post means every reader who switches tabs while Python loads. The export's injected script removes `Notification` from the notebook's page, so marimo skips it. The "made with marimo" badge stays, restyled as a muted pill centred at the notebook's end.
- **The page scrolls, not the notebook.** Inside a post the export marks the notebook's `<html>` with `.in-post`, and `notebook.css` stops marimo's own scroller (`#App`) in both directions, so the page never scrolls sideways. Wide code, tables and charts scroll in their own box. Size charts to the container (`width="container"`) where you can; a chart that needs a fixed size, like a map whose x and y share a scale, scrolls sideways on a phone. marimo draws charts inside a shadow root, so `height="container"` gets no height.
- **Prose cells hide their code** (`@app.cell(hide_code=True)`); every other cell shows it. Inputs are `mo.ui` elements, defined in one cell and read in another.
