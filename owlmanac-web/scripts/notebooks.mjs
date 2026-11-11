// Exports each notebook post as a WASM marimo app into the post's notebook/. A post's frontmatter names
// its notebook (`notebook: <name>`, for owlmanac-notebooks/<name>/notebook.py); the export gets that
// folder's data/ and the site's fonts beside it. Generated, not committed.
// Run by `npm run notebooks`, which `npm run dev` and `npm run build` run first. Needs uv.
//
//   npm run notebooks                          # export what changed
//   npm run notebooks -- mapping-the-milky-way  # re-export these posts, changed or not
//   npm run notebooks -- --force               # re-export every notebook post
//
// An export is skipped when nothing that goes into it has changed (scripts/stamp.mjs): the notebook's
// folder, notebook.css, the uv project and lock, the fonts and this script. --execute runs the whole
// notebook, so this is what keeps `npm run dev` quick.
//
// marimo's frontend (assets/, 30 MB in 700-odd files) is the same for every notebook on one marimo version,
// so it lives once per version in notebook-runtime/<version>/, served at /<base>/notebook-runtime/<version>/,
// and each export's index.html points there. Upgrading marimo moves every notebook to the new version's
// folder; integrations/notebooks.mjs publishes only the versions built posts use.
//
// One file stays with each notebook: the Python worker's entry. In the browser, mo.notebook_location() is
// worked out from the worker's URL (<notebook>/assets/worker-*.js, one folder up), so the runtime starts
// the worker from the page's assets/, where a one-line module imports the real one from the runtime.
import { spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { hashInputs, isFresh, parseArgs, writeStamp } from "./stamp.mjs";

const web = fileURLToPath(new URL("..", import.meta.url));
const posts = join(web, "src/content/posts");
const notebooks = join(web, "../owlmanac-notebooks");
const runtimes = join(web, "notebook-runtime");
const fonts = [
  "@fontsource-variable/inter/files/inter-latin-wght-normal.woff2",
  "@fontsource-variable/jetbrains-mono/files/jetbrains-mono-latin-wght-normal.woff2",
  "@fontsource-variable/newsreader/files/newsreader-latin-opsz-normal.woff2",
];

const { force, names } = parseArgs();
const marimo = marimoVersion();
const runtime = join(runtimes, marimo);
// From a post's notebook/ (posts/<slug>/notebook/) up to the base, then into this version's runtime.
const runtimeUrl = `../../../notebook-runtime/${marimo}/`;

for (const slug of readdirSync(posts)) {
  if (names.length && !names.includes(slug)) continue;
  const name = notebookOf(join(posts, slug));
  if (!name) continue;
  const source = join(notebooks, name);
  const out = join(posts, slug, "notebook");
  const inputs = [
    source,
    join(notebooks, "notebook.css"),
    join(notebooks, "pyproject.toml"),
    join(notebooks, "uv.lock"),
    ...fonts.map((font) => join(web, "node_modules", font)),
    fileURLToPath(import.meta.url),
  ];
  const hash = hashInputs(inputs, [name]);
  if (!force && !names.length && isFresh(out, hash) && existsSync(join(runtime, ".worker"))) {
    console.log(`notebooks: ${slug} is up to date`);
    continue;
  }
  // Empty it rather than delete it: on Windows a folder that a dev server or shell has open can't be removed.
  if (existsSync(out)) for (const entry of readdirSync(out)) rmSync(join(out, entry), { recursive: true, force: true });

  console.log(`notebooks: exporting owlmanac-notebooks/${name} into ${slug}`);
  // Run from the notebook's folder: marimo resolves css_file against the working directory.
  // --execute runs the notebook here and embeds its outputs, so readers see them before Python loads.
  const args = ["run", "--project", notebooks, "marimo", "export", "html-wasm", "notebook.py", "-o", out, "--mode", "run", "--show-code", "--execute", "--no-sandbox", "-f"];
  const run = spawnSync("uv", args, { cwd: source, stdio: "inherit" });
  if (run.status !== 0) process.exit(run.status ?? 1);

  // marimo's frontend goes to this version's shared runtime, unless an earlier export already put it there
  // (.worker, written last, marks a finished one). The page keeps only the worker's entry.
  if (!existsSync(join(runtime, ".worker"))) makeRuntime(join(out, "assets"));
  rmSync(join(out, "assets"), { recursive: true, force: true });
  const worker = readFileSync(join(runtime, ".worker"), "utf8");
  mkdirSync(join(out, "assets"));
  writeFileSync(join(out, "assets", worker), `import "../${runtimeUrl}${worker}";\n`);
  // In the browser the notebook reads mo.notebook_location() / "data", which is this folder's URL.
  if (existsSync(join(source, "data"))) cpSync(join(source, "data"), join(out, "data"), { recursive: true });
  for (const font of fonts) cpSync(join(web, "node_modules", font), join(out, "fonts", font.split("/").pop()));
  rmSync(join(out, "CLAUDE.md"), { force: true }); // marimo writes one into every export
  const index = join(out, "index.html");
  const html = readFileSync(index, "utf8")
    .replace("<head>", `<head>\n${inPost()}`)
    // marimo preloads two background images (noise, gradient) that only its editor shows, when
    // disconnected; a notebook post never uses them, and the browser warns that they went unused.
    .replace(/<link rel="preload" href="[^"]*" as="image" \/>\r?\n?/g, "")
    .replace(/(["'(])\.\/assets\//g, `$1${runtimeUrl}`);
  writeFileSync(index, html);
  writeFileSync(join(out, ".runtime"), marimo); // which runtime it needs, for the build
  writeFileSync(join(out, ".gitignore"), "*\n");
  writeStamp(out, hash);
}

/**
 * Moves an export's assets/ into this version's runtime, and points the one URL that must be the page's
 * there: marimo starts its Python worker from `new URL("worker-*.js", import.meta.url)`, which would now be
 * the runtime; it becomes `new URL("assets/worker-*.js", document.baseURI)`, the page's. If a new marimo
 * builds that URL differently, this stops rather than ship notebooks that can't find their data.
 */
function makeRuntime(assets) {
  rmSync(runtime, { recursive: true, force: true });
  mkdirSync(runtimes, { recursive: true });
  writeFileSync(join(runtimes, ".gitignore"), "*\n");
  renameSync(assets, runtime);
  const WORKER = /new URL\(`(worker-[\w-]+\.js)`,import\.meta\.url\)/g;
  const found = readdirSync(runtime)
    .filter((file) => file.endsWith(".js"))
    .flatMap((file) => [...readFileSync(join(runtime, file), "utf8").matchAll(WORKER)].map((match) => ({ file, worker: match[1] })));
  if (found.length !== 1) {
    rmSync(runtime, { recursive: true, force: true });
    throw new Error(`notebooks: expected marimo ${marimo} to start its worker from one new URL(\`worker-*.js\`, import.meta.url), found ${found.length}; see makeRuntime in scripts/notebooks.mjs`);
  }
  const [{ file, worker }] = found;
  const chunk = join(runtime, file);
  writeFileSync(chunk, readFileSync(chunk, "utf8").replace(WORKER, `new URL(\`assets/${worker}\`,document.baseURI)`));
  writeFileSync(join(runtime, ".worker"), worker);
}

/** The marimo version uv.lock pins: the runtime every export uses. */
function marimoVersion() {
  const lock = readFileSync(join(notebooks, "uv.lock"), "utf8");
  const version = /\[\[package\]\]\r?\nname = "marimo"\r?\nversion = "([^"]+)"/.exec(lock)?.[1];
  if (!version) throw new Error("notebooks: no marimo version in owlmanac-notebooks/uv.lock");
  return version;
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
    // The post keeps the frame transparent until this page has painted once, in the site's theme
    // (NotebookLayout). Before that the frame shows its blank first document, whose light scheme makes the
    // browser paint a white backdrop on a dark page. Two frames: the first paints, the second follows it.
    requestAnimationFrame(() => requestAnimationFrame(() => window.frameElement?.setAttribute("data-painted", "")));
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
