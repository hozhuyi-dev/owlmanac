// What the star-battle WASM package returns, mirroring the Rust types. Hand-written, beside the
// generated package in star-battle/, and shared by every post that uses it.

export type Cell = { row: number; col: number };
export type Mark = "star" | "empty";
export type Placed = { cell: Cell; mark: Mark };
export type Rule = { kind: "row" | "column" | "region"; at: number } | { kind: "noTouching"; at: Cell };

export type Blame = { marks: Placed[]; rules: Rule[] };
export type Step = { placed: Placed; rule: Rule; because: Placed[] };
export type Clash = { rule: Rule; marks: Placed[] };
export type Chain = { steps: Step[]; clash: Clash };
export type WhyNot = { kind: "possible" } | ({ kind: "deduced" } & Chain) | ({ kind: "needsCases" } & Blame);

/** `found` holds each solution's stars; `complete` says whether those are all of them. */
export type Solutions = { found: Cell[][]; complete: boolean };
// Rust's `None` arrives as `undefined` (serde-wasm-bindgen's default), never `null`.

/** `depth` is undefined when a cell needs deeper reasoning than the grader tries. */
export type Grade = { depth?: number; length: number; deepestSteps: number };
export type LearnedRule = { cannotAllHold: Placed[]; lbd: number };
/** A cell the player can work out next, and how many "what if"s deep the reasoning goes. */
export type Hint = { placed: Placed; depth: number };
/** One try at making a puzzle by choosing the stars first. `other` is undefined if they're the only solution. */
export type Attempt = { stars: Cell[]; regions: number[]; other?: Cell[] };

export type Puzzle = { size: number; stars: number; regions: number[] };

/** A board's marks, one per cell row by row: 1 a star, -1 ruled out, 0 unmarked. */
export type Marks = Int8Array<ArrayBuffer>;
