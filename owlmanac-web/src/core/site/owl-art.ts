/**
 * Owlmanac's drawings: the owl mark, and small line drawings for the site's quiet moments (loading, empty,
 * not found, errors), each shown with a quip. Everything is drawn in currentColor. The drawings sit on a
 * 32-unit grid with a 1.5 stroke and round ends: the weight of the owl's own line at that size.
 *
 * Plain strings, so the same drawing serves an .astro page (set:html), a Vue island (v-html) and CSS (a
 * data: URI, see artUrl).
 */

/**
 * The mark: a horned owl seen from behind, perched on a branch, on a 64-unit grid. BODY and BRANCH make
 * the filled silhouette (the header, the favicon); LINE is the same owl as one continuous stroke, in along
 * the branch, round the owl and out along it, so it can draw itself while something loads.
 */
export const OWL = {
  body: "M24 51.1L22.6 48.6L21.4 49.2L21.2 46.8L19.6 46.4L20 44.2C18.2 38 17.4 30 17.8 24C18.1 19.6 18.4 15.6 18.8 12.6C18.8 9.4 18.4 5.8 18.4 2.2C20.4 3.8 22.4 6 25.6 6.6C28.6 6.8 31 5.4 33.2 1.8C33.8 5 34.4 8.4 34.6 10.8C38.4 15 41.2 21.4 42.6 28C44 34.4 46.2 40 45.8 44.6C45.6 47.4 43.6 49.6 41.6 51C41.4 55 40.8 58.4 39.8 61.4C39 58.4 38.4 55 38.2 51.8Z",
  branch: "M4 50.4L60 52.6M9.4 50.6L6.6 47.6",
  line: "M4 50.4L9.4 50.6L6.6 47.6L9.4 50.6L24 51.1L22.6 48.6L21.4 49.2L21.2 46.8L19.6 46.4L20 44.2C18.2 38 17.4 30 17.8 24C18.1 19.6 18.4 15.6 18.8 12.6C18.8 9.4 18.4 5.8 18.4 2.2C20.4 3.8 22.4 6 25.6 6.6C28.6 6.8 31 5.4 33.2 1.8C33.8 5 34.4 8.4 34.6 10.8C38.4 15 41.2 21.4 42.6 28C44 34.4 46.2 40 45.8 44.6C45.6 47.4 43.6 49.6 41.6 51C41.4 55 40.8 58.4 39.8 61.4C39 58.4 38.4 55 38.2 51.8L60 52.6",
};

/** The "z z" an asleep owl gives off, in the mark's 64 grid: rising from beside its head, in the space right of it. */
const ZZ = "M49.2 19.6H53.4L49.2 24.6H53.4M55.2 7.6H61L55.2 14.6H61";

/**
 * The filled mark as a whole <svg>, sized by its box. `asleep` adds the "z z" as a .by-day path, which
 * the theme shows by day only (the header's mark sleeps in light theme).
 */
export function owlMarkSvg(size: number, { asleep = false } = {}): string {
  const zz = asleep
    ? `<path class="by-day" d="${ZZ}" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>`
    : "";
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="${size}" height="${size}" aria-hidden="true"><path d="${OWL.branch}" fill="none" stroke="currentColor" stroke-width="3.6" stroke-linecap="round"/><path d="${OWL.body}" fill="currentColor" stroke="currentColor" stroke-width="1" stroke-linejoin="round"/>${zz}</svg>`;
}

/** A dotted path: round caps on near-zero dashes draw dots. */
const DOTTED = 'stroke-dasharray="0.01 3"';

const ART = {
  /** The owl's one line, halved onto the grid. Drawn on (pathLength 1) when it shows something loading. */
  owl: `<path pathLength="1" transform="scale(0.5)" stroke-width="3" d="${OWL.line}"/>`,
  /** The owl in flight, from the front, wings raised (64 grid, halved). */
  flying:
    '<g transform="scale(0.5) translate(0 2.6)" stroke-width="3"><path pathLength="1" d="M32 19.6C35.4 19.6 37 21.4 36.8 24.4C37 25.6 37.6 26.6 38.4 27C45 23.6 52.6 17.4 59 10.4L57.6 16.6L59.6 18.4L55.4 21.8L57.4 24.2L52 26.6L53.2 29.2C47 30.4 41.4 31.6 37.8 34.4C37.6 38.2 36.6 41.2 34.8 43.4L36 48.6L32 47.2L28 48.6L29.2 43.4C27.4 41.2 26.4 38.2 26.2 34.4C22.6 31.6 17 30.4 10.8 29.2L12 26.6L6.6 24.2L8.6 21.8L4.4 18.4L6.4 16.6L5 10.4C11.4 17.4 19 23.6 25.6 27C26.4 26.6 27 25.6 27.2 24.4C27 21.4 28.6 19.6 32 19.6Z"/>' +
    '<path d="M29.2 25.6Q32 28.6 34.8 25.6M30.2 24.2h0M33.8 24.2h0"/></g>',
  /** A barn owl perched, turned to face you, wide awake: one line, its heart-shaped face included (64 grid,
   *  halved), and two eyes. */
  barn:
    '<path pathLength="1" transform="scale(0.5)" stroke-width="3" d="M4 51L24 51.4C19 47 15.6 40 15.8 33C16 28 17.4 24.6 19 22.4C15.4 19 15 13 17.8 9.4C20 6.6 22.6 5.8 25 5.8V7.2C21.6 4.8 18.2 8 18.8 12.6C19.4 16.8 22.4 19.6 25 21.8C27.6 19.6 30.6 16.8 31.2 12.6C31.8 8 28.4 4.8 25 7.2V5.8C28.4 5.8 32.4 8.2 33.6 12.4C34.4 15 34.2 17 33.4 18.6C38 22 41.4 28 43.4 35C45.2 41.4 46.4 48.6 48.8 57.8L45.6 55.6L43.4 51.6L60 52.4"/>' +
    '<path stroke-width="1.8" d="M11 6.4h0M14 6.4h0"/>',
  /** The owl from behind, asleep on its branch: the mark's line, and a "z z" rising beside its head. */
  asleep:
    `<path pathLength="1" transform="scale(0.5)" stroke-width="3" d="${OWL.line}"/>` +
    '<path stroke-width="1.2" d="M24.6 9.4H26.8L24.6 12H26.8M27.6 3.4H30.6L27.6 7H30.6"/>',
  /** A pencil, writing. */
  pencil:
    '<g transform="rotate(-40 16 16)"><path d="M11 13.6H25.6V18.4H11Z"/><path d="M11 13.6L5.6 16L11 18.4"/><path d="M22.6 13.6V18.4"/></g>' +
    '<path d="M3.6 27.4C5.6 25.6 6.6 28.6 8.6 27C10 25.8 10.6 25.2 12.2 26.4"/>',
  /** An open book, its page turning. */
  book:
    '<path d="M4 10Q10 8 16 11Q22 8 28 10V25Q22 23 16 26Q10 23 4 25Z"/><path d="M16 11V26"/>' +
    '<path d="M7.4 14Q10.4 13.2 13 14.2M7.4 17.4Q10.4 16.6 13 17.6M7.4 20.8Q10.4 20 13 21"/>' +
    '<path d="M16 11Q20 4.6 26.6 5.4"/>',
  /** A candle, burning late. */
  candle:
    '<path d="M16 4.6C14 7.8 14 10.2 16 11.8C18 10.2 18 7.8 16 4.6Z"/><path d="M16 11.8V14"/>' +
    '<path d="M12.6 14H19.4V27H12.6Z"/><path d="M12.6 17.4Q14 19.4 15.2 17.4"/><path d="M8.6 27H23.4"/>',
  /** A mug, steaming. */
  mug:
    '<path d="M7.6 13H20.6V22.4A3.6 3.6 0 0 1 17 26H11.2A3.6 3.6 0 0 1 7.6 22.4Z"/>' +
    '<path d="M20.6 15.4H22.6A2.6 2.6 0 0 1 22.6 20.6H20.6"/>' +
    '<path d="M11.6 10Q10.6 8.2 12 6.6T11.8 3.2M16.4 10Q15.4 8.2 16.8 6.6T16.6 3.2"/>',
  /** An hourglass, half run. */
  hourglass:
    '<path d="M9 4H23M9 28H23"/>' +
    '<path d="M10.6 4C10.6 11 16 13 16 16C16 19 10.6 21 10.6 28M21.4 4C21.4 11 16 13 16 16C16 19 21.4 21 21.4 28"/>' +
    '<path d="M13.4 25.6Q16 22.6 18.6 25.6Z"/><path d="M14.4 10.6H17.6"/><path d="M16 17.6V22" ' + DOTTED + "/>",
  /** A lens over a blank page: looked, found nothing. */
  lens:
    '<path d="M8 4H19.6L24.4 8.8V28H8Z"/><path d="M19.6 4V8.8H24.4"/>' +
    '<circle cx="14.6" cy="16.6" r="4.8"/><path d="M18.2 20.2L22.6 24.6"/>',
  /** A bare branch, a feather falling: the owl has flown. */
  branch:
    '<path d="M3 13.6Q16 12.4 29 14.4"/><path d="M8.4 13.4L5.6 10.2M21 13.6L23.6 10.8"/>' +
    '<path d="M14.6 27.6L19.6 20.4"/><path d="M19.6 20.4C16.6 20.6 14.4 22.6 14.2 26L15.6 25.2L15.8 26.4C18.4 25.6 19.8 23.2 19.6 20.4Z"/>' +
    '<path d="M22 11Q25.4 7.4 29.4 6.4" ' + DOTTED + "/>",
  /** A feather, drifting. */
  feather:
    '<path d="M7 27.4L19.4 13"/>' +
    '<path d="M25 5C19 5.6 12.6 11 11.2 21.4L13.2 19.6L13.6 21.8C19.2 19.6 23.6 13 25 5Z"/>' +
    '<path d="M4 13Q6.6 10.6 9.2 12" ' + DOTTED + "/>",
  /** A pencil, snapped. */
  snapped:
    '<g transform="rotate(-30 16 16)"><path d="M4.6 16L9.6 13.6V18.4Z"/><path d="M9.6 13.6H14.4L15.4 15L14.6 16.2L15.4 18.4H9.6"/>' +
    '<path d="M18.2 13.6L17.4 14.8L18.2 16L17.4 18.4H27V13.6Z"/><path d="M24 13.6V18.4"/></g>' +
    '<path d="M15.6 8.6V6.6M19.2 9.6L20.6 8.2M12.2 9.4L10.8 8"/>',
  /** An ink bottle knocked over. */
  ink:
    '<path d="M4.8 12.6H12.6L16.6 15V20.6L12.6 23H4.8A1.6 1.6 0 0 1 3.2 21.4V14.2A1.6 1.6 0 0 1 4.8 12.6Z"/><path d="M16.6 16.2H19.4V19.4H16.6"/><path d="M7.4 15.6V20"/>' +
    '<path d="M20.2 21.4C22.6 19.4 28.6 20 28.2 23.4C27.8 26.4 21.4 27 19.4 25C18.2 23.8 18.8 22.6 20.2 21.4Z"/>',
  /** A line that got itself in a knot. */
  knot: '<path d="M3 18C8 18 9 10 14 10C19 10 19.4 20 14.4 20C9.6 20 10.4 12.4 16 12.4C21.6 12.4 22 22 17.2 22C13.4 22 14 16 19.6 16C24 16 25 18 29 18"/>',
} as const;

export type ArtName = keyof typeof ART;

/**
 * Night is the dark theme, when the owl is up and working; day is the light theme, when it sleeps. Every
 * quiet moment has both, and the theme shows one (.by-night, .by-day in global.css).
 */
export type Time = "night" | "day";
export const TIMES: readonly Time[] = ["night", "day"];

/**
 * Which drawings each moment picks from: `busy` while something loads, `empty` when a search finds
 * nothing, `lost` for a page that isn't there, `broken` for something that failed.
 */
export const POOLS = {
  night: {
    busy: ["owl", "owl", "flying", "pencil", "book", "candle", "mug", "hourglass"],
    empty: ["barn", "lens", "branch", "feather"],
    lost: ["flying", "branch", "feather", "barn"],
    broken: ["snapped", "ink", "knot"],
  },
  day: {
    busy: ["asleep", "asleep", "mug", "book", "hourglass"],
    empty: ["asleep", "lens", "branch", "feather"],
    lost: ["asleep", "branch", "feather", "barn"],
    broken: ["snapped", "ink", "knot", "mug"],
  },
} as const satisfies Record<Time, Record<string, readonly ArtName[]>>;

export type Pool = keyof (typeof POOLS)["night"];

/**
 * One short line above a plain explanation, never instead of it. Deadpan, owlish: by night about working
 * late on a problem, by day about an owl woken too early. Kept out of anything the reader needs to act on.
 */
const NIGHT_QUIPS = {
  busy: [
    "Thinking it over…",
    "Sharpening the pencil…",
    "Checking the working…",
    "Turning the pages…",
    "Burning the midnight oil…",
    "Brewing another pot…",
    "One more test…",
    "On the wing…",
    "Up past bedtime. Nearly there…",
  ],
  empty: [
    "Searched every branch.",
    "The owl looked twice. Nothing.",
    "Not in this almanac. Yet.",
    "No such problem. Lucky you.",
  ],
  lost: [
    "Hoo? Not in the almanac.",
    "This page flew off.",
    "Only a feather left here.",
    "Wrong branch.",
  ],
  broken: ["Snapped the pencil.", "Spilt the ink.", "Got in a knot.", "That didn't add up."],
} as const satisfies Record<Pool, readonly string[]>;

const DAY_QUIPS = {
  busy: [
    "Waking the owl…",
    "Rubbing its eyes…",
    "Yawning. Nearly there…",
    "Finding its spectacles…",
    "Making the first coffee…",
    "Reading last night's notes…",
  ],
  empty: [
    "Slept through it. Nothing here.",
    "Looked, half asleep. Nothing.",
    "Not in this almanac. Yet.",
    "Nothing, and too early to mind.",
  ],
  lost: [
    "Sleepwalked off the page.",
    "This page is still in bed.",
    "Hoo? Ask again tonight.",
    "Wrong branch. Too early.",
  ],
  broken: ["Dozed off mid-sum.", "Snapped the pencil before coffee.", "Spilt the coffee.", "Woke up in a knot."],
} as const satisfies Record<Pool, readonly string[]>;

/**
 * The header's line under the site's name, on every page: what the owl is up to, by the theme (up at
 * night, asleep by day, beside the mark's "z z"). One is picked per visit, so the site stays fresh; the
 * first is what pages are built with. Same voice as the quips, and not about the sky. It never wraps: a
 * visit picks only from the lines that fit beside the nav (BaseLayout), so each pool keeps short ones (18
 * characters or fewer) for the narrowest phones. At most 46 characters, one line on a wide screen.
 */
export const HOURS = {
  night: [
    "The owl is up. Pull up a chair.",
    "The owl is up, and the kettle's on.",
    "Up and working. Mind the papers.",
    "The owl is at its desk. Come in.",
    "Wide awake, and halfway through a proof.",
    "The owl is up. So, it seems, are you.",
    "Night shift. The problems are open.",
    "Burning the oil. There's plenty for two.",
    "The owl is in. Something's being solved.",
    "Awake, alert, and slightly over-caffeinated.",
    "The owl is up, sharpening pencils.",
    "Working hours: now.",
    "The owl is up and the almanac is open.",
    "Prime time for problems.",
    "Eyes open, pencil out. Come in.",
    "The owl is up, checking the working.",
    "Another late one. Good company welcome.",
    "Up past everyone's bedtime but yours.",
    "The night's work has started. Join in.",
    "Hoo's there? Come in, come in.",
    "The owl is up. Mind the ink.",
    "Fully nocturnal, fully caffeinated.",
    "The candle's lit and the problems are waiting.",
    "Quiet hours. Good for thinking.",
    // Short ones, for the narrowest phones, where the line has about 18 characters beside the nav.
    "Up past midnight.",
    "Proof in progress.",
    "Still at the desk.",
    "Wide awake.",
    "Kettle's on.",
    "Pencil out.",
    "Night shift.",
    "Mid-proof.",
    "Hoo's up? Me.",
    "Eyes wide open.",
    "Lamp's still on.",
    "Ink still wet.",
    "Thinking. Come in.",
    "Late, and working.",
    "On the night shift.",
    "Burning the oil.",
    // A little longer, for the next size of phone (about 26 characters).
    "Up late, chasing a proof.",
    "The kettle's on. Come in.",
    "Working while you sleep.",
    "Still up. Pull up a chair.",
  ],
  day: [
    "The owl is asleep. Read quietly.",
    "The owl is asleep. Help yourself.",
    "Asleep on the branch. Do read on.",
    "The owl sleeps. The almanac doesn't.",
    "Do not disturb. Reading allowed.",
    "Asleep. Back after dark.",
    "The owl is dreaming of tidy proofs.",
    "Asleep mid-thought. Read on, softly.",
    "Napping. The working is all on the page.",
    "Off duty until dusk.",
    "The owl is asleep. The kettle is cold.",
    "Shh. Owl asleep, posts awake.",
    "Sleeping off last night's problem.",
    "The owl worked late. Let it sleep.",
    "Asleep. The pencils are sharpened for later.",
    "The owl keeps odd hours. The posts don't.",
    "Resting its eyes. Read on.",
    "The owl is asleep. Tiptoe through the posts.",
    "Snoozing. Leave a problem by the door.",
    "Asleep at the desk again.",
    "Gone to roost. The posts stay up.",
    "The owl is out cold. The code still runs.",
    "Asleep. Everything here works without it.",
    "Fast asleep, one ear on the search box.",
    // Short ones, for the narrowest phones.
    "Asleep. Shh.",
    "Back at dusk.",
    "Do not disturb.",
    "Roosting.",
    "Fast asleep.",
    "Napping. Read on.",
    "Out like a candle.",
    "Zzz. Read on.",
    "Asleep. Read on.",
    "Dreaming of proofs.",
    "Snoring softly.",
    "Lights out.",
    "Do read quietly.",
    "Asleep till dusk.",
    "Head under a wing.",
    "Napping. Shh.",
    // A little longer, for the next size of phone.
    "Asleep. The posts aren't.",
    "Back after dark. Read on.",
    "Asleep at the desk. Shh.",
    "Owl asleep. Tiptoe in.",
  ],
} as const satisfies Record<Time, readonly string[]>;

export const QUIPS = { night: NIGHT_QUIPS, day: DAY_QUIPS } as const satisfies Record<
  Time,
  Record<Pool, readonly string[]>
>;

/** What a quip names, and the drawing that goes with it: first match wins. A new quip pairs itself by its words. */
const ART_WORDS: ReadonlyArray<readonly [RegExp, ArtName]> = [
  [/owl looked|hoo\?/i, "barn"],
  [/sleep|doz|yawn|\bbed\b|waking|eyes|spectacles/i, "asleep"],
  [/fl(y|ew|ight)|wing/i, "flying"],
  [/search|looked|nothing\b|no such/i, "lens"],
  [/pencil|working|snapped/i, "pencil"],
  [/page|almanac|book/i, "book"],
  [/oil|candle|late|bedtime/i, "candle"],
  [/brew|pot|coffee/i, "mug"],
  [/\bink\b|spilt/i, "ink"],
  [/knot/i, "knot"],
  [/feather/i, "feather"],
  [/branch/i, "branch"],
  [/think/i, "owl"],
];

const pick = <T>(list: readonly T[]): T => list[Math.floor(Math.random() * list.length)];

/** The drawing a quip names, else `fallback`. A snapped pencil is the error's pencil; the plain one is for working. */
export function artFor(quip: string, pool: Pool, fallback: ArtName, time: Time = "night"): ArtName {
  const named = ART_WORDS.find(([words]) => words.test(quip))?.[1];
  if (pool === "broken" && named === "pencil") return "snapped";
  return named && (POOLS[time][pool] as readonly ArtName[]).includes(named) ? named : fallback;
}

/** A quip and its drawing for a moment, picked at random: one per visit, so the site stays fresh. */
export function quipFor(pool: Pool, time: Time = "night"): { quip: string; art: ArtName } {
  const quip = pick(QUIPS[time][pool]);
  return { quip, art: artFor(quip, pool, pick(POOLS[time][pool]), time) };
}

/** The same, always the first: what the page is built with, before a script picks one at random. */
export function firstQuip(pool: Pool, time: Time = "night"): { quip: string; art: ArtName } {
  const quip = QUIPS[time][pool][0];
  return { quip, art: artFor(quip, pool, POOLS[time][pool][0], time) };
}

/** A drawing as a whole <svg>. `draw` has its lines draw themselves on, over and over (.art.drawing). */
export function artSvg(name: ArtName, size = 48, { draw = false } = {}): string {
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" width="${size}" height="${size}" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" class="art${draw ? " drawing" : ""}">${ART[name]}</svg>`;
}

/**
 * A drawing as a CSS url(), for a mask (it then takes the colour behind it, currentColor). The owl
 * carries its own animation, so a chart's placeholder can show it drawing itself with no script.
 */
export function artUrl(name: ArtName): string {
  const style =
    name === "owl"
      ? "<style>path{stroke-dasharray:1;stroke-dashoffset:1;animation:d 3.2s ease-in-out infinite}@keyframes d{60%,88%{stroke-dashoffset:0;opacity:1}to{stroke-dashoffset:0;opacity:0}}@media (prefers-reduced-motion:reduce){path{animation:none;stroke-dashoffset:0}}</style>"
      : "";
  const svg = artSvg(name).replace("currentColor", "#000").replace(">", `>${style}`);
  return `url("data:image/svg+xml,${encodeURIComponent(svg)}")`;
}
