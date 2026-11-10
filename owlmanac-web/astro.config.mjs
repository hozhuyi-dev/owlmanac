// @ts-check
import { readFileSync } from "node:fs";
import { defineConfig, fontProviders } from "astro/config";

import vue from "@astrojs/vue";
import mdx from "@astrojs/mdx";
import sitemap from "@astrojs/sitemap";
import notebooks from "./integrations/notebooks.mjs";

// https://astro.build/config
export default defineConfig({
  // Served at hozhuyi-dev.github.io/owlmanac/. An org site or custom domain changes only these two.
  site: "https://hozhuyi-dev.github.io",
  base: "/owlmanac",
  trailingSlash: "always",
  integrations: [vue(), mdx(), sitemap(), notebooks()],
  // The site's fonts, from the Fontsource packages (latin only). Astro writes their @font-face rules and a
  // fallback sized to match each one, so nothing reflows if a font is late; BaseLayout preloads the two
  // every page uses. `fallback` display hides text for at most ~100ms while a font arrives (a preloaded font
  // usually does), instead of drawing it in the fallback and visibly swapping.
  fonts: [
    {
      provider: fontProviders.local(),
      name: "Inter",
      cssVariable: "--font-sans",
      fallbacks: ["system-ui", "sans-serif"],
      display: "fallback",
      options: {
        variants: [{ src: ["@fontsource-variable/inter/files/inter-latin-wght-normal.woff2"], weight: "100 900", style: "normal" }],
      },
    },
    {
      provider: fontProviders.local(),
      name: "Newsreader",
      cssVariable: "--font-display",
      fallbacks: ["Georgia", "serif"],
      display: "fallback",
      options: {
        variants: [
          { src: ["@fontsource-variable/newsreader/files/newsreader-latin-opsz-normal.woff2"], weight: "200 800", style: "normal" },
          { src: ["@fontsource-variable/newsreader/files/newsreader-latin-opsz-italic.woff2"], weight: "200 800", style: "italic" },
        ],
      },
    },
    {
      provider: fontProviders.local(),
      name: "JetBrains Mono",
      cssVariable: "--font-mono",
      fallbacks: ["ui-monospace", "Menlo", "Consolas", "monospace"],
      display: "fallback",
      options: {
        variants: [
          { src: ["@fontsource-variable/jetbrains-mono/files/jetbrains-mono-latin-wght-normal.woff2"], weight: "100 800", style: "normal" },
        ],
      },
    },
  ],
  markdown: {
    shikiConfig: { themes: { light: "github-light", dark: "github-dark" }, defaultColor: false },
  },
  vite: {
    // DuckDB fetches data by URL, so parquet files are never inlined as data: URIs.
    build: { assetsInlineLimit: (file) => (file.endsWith(".parquet") ? false : undefined) },
    optimizeDeps: { exclude: ["@duckdb/duckdb-wasm"] },
    plugins: [
      {
        // DuckDB-WASM's sourcemaps point at apache-arrow sources it doesn't ship, so the dev server
        // warns once per file. Load its bundles without them.
        name: "duckdb-without-sourcemaps",
        // Ids carry at most a `?v=` version hash; `?url` imports are left alone so they still resolve to a URL.
        enforce: "pre",
        apply: "serve",
        load(id) {
          const match = /^(.*[\\/]@duckdb[\\/]duckdb-wasm[\\/]dist[\\/][^\\/?]+\.(?:m?js|cjs))(?:\?v=\w+)?$/.exec(id);
          if (match) return readFileSync(match[1], "utf8").replace(/\/\/# sourceMappingURL=\S+\s*$/, "");
        },
      },
    ],
  },
});
