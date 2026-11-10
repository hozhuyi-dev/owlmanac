// @ts-check
import js from "@eslint/js";
import { defineConfig } from "eslint/config";
import astro from "eslint-plugin-astro";
import vue from "eslint-plugin-vue";
import globals from "globals";
import tseslint from "typescript-eslint";

// Lint for mistakes, not layout: the recommended rules of ESLint, typescript-eslint and Astro, and Vue's
// essential ones (its recommended set mostly formats templates). CI runs it (npm run lint).
export default defineConfig(
  // Generated: WASM packages (npm run wasm), notebook exports and marimo's runtime (npm run notebooks), the
  // build, Astro's types.
  {
    ignores: [
      "dist/**",
      ".astro/**",
      "src/wasm/*/**",
      "src/content/posts/*/wasm/**",
      "src/content/posts/*/notebook/**",
      "notebook-runtime/**",
    ],
  },
  js.configs.recommended,
  tseslint.configs.recommended,
  vue.configs["flat/essential"],
  {
    files: ["**/*.vue"],
    languageOptions: { parserOptions: { parser: tseslint.parser } },
    // Multi-word names keep components from clashing with HTML elements inside Vue templates. These are
    // Astro islands, each mounted on its own, so a figure can be called Game.
    rules: { "vue/multi-word-component-names": "off" },
  },
  astro.configs.recommended,
  { languageOptions: { globals: { ...globals.browser, ...globals.node } } },
  // README: core/ and components/ never import a post. Everything starts in its post, and moves out when a
  // second post needs it.
  {
    files: ["src/core/**", "src/components/**"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: ["**/content/posts/**"],
              message: "core/ and components/ never import a post: move what's shared out of the post first.",
            },
          ],
        },
      ],
    },
  },
);
