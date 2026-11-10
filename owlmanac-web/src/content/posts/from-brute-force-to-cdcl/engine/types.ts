// What a run of the ladder reports, mirroring the WASM crate.

export type Outcome = "waiting" | "running" | "solved" | "noSolution" | "gaveUp" | "cancelled";

export type Work = { guesses: number; deadEnds: number; propagations: number };

/** `grid` is the puzzle as the search has it now, row by row with 0 for a blank. SAT only has one once solved. */
export type RungResult = Work & { outcome: Outcome; ms: number; grid?: Uint8Array };
