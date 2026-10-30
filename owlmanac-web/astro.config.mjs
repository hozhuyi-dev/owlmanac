// @ts-check
import { readFileSync } from "node:fs";
import { defineConfig } from "astro/config";

import vue from "@astrojs/vue";
import mdx from "@astrojs/mdx";
import sitemap from "@astrojs/sitemap";

// https://astro.build/config
export default defineConfig({
  // Served at hozhuyi-dev.github.io/owlmanac/. An org site or custom domain changes only these two.
  site: "https://hozhuyi-dev.github.io",
  base: "/owlmanac",
  trailingSlash: "always",
  integrations: [vue(), mdx(), sitemap()],
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
