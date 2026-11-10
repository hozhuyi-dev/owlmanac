// Exports each notebook post as a WASM marimo app into the post's notebook/. A post's frontmatter names
// its notebook (`notebook: <name>`, for owlmanac-notebooks/<name>/notebook.py); the export gets that
// folder's data/ and the site's fonts beside it. Generated, not committed.
// Run by `npm run notebooks`, which `npm run dev` and `npm run build` run first. Needs uv.
import { spawnSync } from "node:child_process";
import { cpSync, existsSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const web = fileURLToPath(new URL("..", import.meta.url));
const posts = join(web, "src/content/posts");
const notebooks = join(web, "../owlmanac-notebooks");
const fonts = [
  "@fontsource-variable/inter/files/inter-latin-wght-normal.woff2",
  "@fontsource-variable/jetbrains-mono/files/jetbrains-mono-latin-wght-normal.woff2",
  "@fontsource-variable/newsreader/files/newsreader-latin-opsz-normal.woff2",
];

for (const slug of readdirSync(posts)) {
  const name = notebookOf(join(posts, slug));
  if (!name) continue;
  const source = join(notebooks, name);
  const out = join(posts, slug, "notebook");
  // Empty it rather than delete it: on Windows a folder that a dev server or shell has open can't be removed.
  if (existsSync(out)) for (const entry of readdirSync(out)) rmSync(join(out, entry), { recursive: true, force: true });

  console.log(`notebooks: exporting owlmanac-notebooks/${name} into ${slug}`);
  // Run from the notebook's folder: marimo resolves css_file against the working directory.
  // --execute runs the notebook here and embeds its outputs, so readers see them before Python loads.
  const args = ["run", "--project", notebooks, "marimo", "export", "html-wasm", "notebook.py", "-o", out, "--mode", "run", "--show-code", "--execute", "--no-sandbox", "-f"];
  const run = spawnSync("uv", args, { cwd: source, stdio: "inherit" });
  if (run.status !== 0) process.exit(run.status ?? 1);

  // In the browser the notebook reads mo.notebook_location() / "data", which is this folder's URL.
  if (existsSync(join(source, "data"))) cpSync(join(source, "data"), join(out, "data"), { recursive: true });
  for (const font of fonts) cpSync(join(web, "node_modules", font), join(out, "fonts", font.split("/").pop()));
  rmSync(join(out, "CLAUDE.md"), { force: true }); // marimo writes one into every export
  const index = join(out, "index.html");
  writeFileSync(index, readFileSync(index, "utf8").replace("<head>", `<head>\n${inPost()}`));
  writeFileSync(join(out, ".gitignore"), "*\n");
}

/** The notebook a post's frontmatter names, if any. */
function notebookOf(post) {
  const index = ["index.md", "index.mdx"].map((f) => join(post, f)).find(existsSync);
  const frontmatter = index && /^---\r?\n([\s\S]*?)\r?\n---/.exec(readFileSync(index, "utf8"))?.[1];
  return frontmatter && /^notebook:\s*"?([\w-]+)"?\s*$/m.exec(frontmatter)?.[1];
}

// What the notebook does everywhere: never asks for notification permission (see below).
// What it does inside a post's page (the frame shares the page's origin, so it can read it):
// - Marks <html> with .in-post, so notebook.css can hand scrolling to the page.
// - Marks <html> with .site-dark when the page is dark, from the first paint: marimo turns dark only
//   once its JavaScript has run, and until then the frame would flash white.
// - Follows the site's theme. Notebooks use marimo's "system" theme, which reads prefers-color-scheme;
//   inside a post the site's theme switch (data-theme on the page's <html>) should win, so the notebook's
//   dark-mode query answers with the page's theme, and tells marimo when it changes.
function inPost() {
  return `<script>
  (() => {
    // marimo asks for notification permission to say "Execution completed" when a run finishes in a
    // background tab, so a reader who switches tabs while Python loads is prompted. Posts never notify:
    // without the Notification API, marimo skips it.
    try { delete window.Notification; } catch {}
    let site;
    try { site = window.parent !== window && window.parent.document.documentElement; } catch {}
    if (!site) return;
    document.documentElement.classList.add("in-post");
    const QUERY = "(prefers-color-scheme: dark)";
    const system = window.parent.matchMedia(QUERY);
    // The site is dark unless set to light, or to auto (the system decides).
    const dark = () => (site.dataset.theme === "auto" ? system.matches : site.dataset.theme !== "light");
    const listeners = new Set();
    const query = {
      media: QUERY, onchange: null, get matches() { return dark(); },
      addEventListener: (_, f) => listeners.add(f), removeEventListener: (_, f) => listeners.delete(f),
      addListener: (f) => listeners.add(f), removeListener: (f) => listeners.delete(f), dispatchEvent: () => true,
    };
    const mark = () => document.documentElement.classList.toggle("site-dark", dark());
    mark();
    const notify = () => {
      mark();
      const event = { matches: dark(), media: QUERY };
      listeners.forEach((f) => f(event));
      query.onchange?.(event);
    };
    new MutationObserver(notify).observe(site, { attributes: true, attributeFilter: ["data-theme"] });
    system.addEventListener("change", notify);
    const matchMedia = window.matchMedia.bind(window);
    window.matchMedia = (q) => (q.replace(/\\s/g, "") === QUERY.replace(/\\s/g, "") ? query : matchMedia(q));
  })();
</script>`;
}
