// The 16×16 and 25×25 came from sudoku::generate(box, 0, 3), blanked until no further cell could go.
// A later fix to how seeds are scrambled changed what that call returns, so these clues are the record.
// Generating the 25×25 takes minutes, so these are kept here rather than made in the browser.

export type SudokuPuzzle = {
  name: string;
  /** What this puzzle shows. */
  lesson: string;
  boxSize: number;
  extended: boolean;
  clues: string;
};

// The 9×9 was posted by Rico Alan on Flickr in 2008 (https://www.flickr.com/photos/npcomplete/2361922699)
// and appears in Wikipedia's "Sudoku solving algorithms" article, in an image by Auguel under CC BY-SA 4.0
// (https://commons.wikimedia.org/wiki/File:Sudoku_puzzle_hard_for_brute_force.svg). Only its clues are used.
export const PUZZLES: SudokuPuzzle[] = [
  {
    name: "9×9, built against brute force",
    lesson: "Propagation alone solves it, with no guessing at all.",
    boxSize: 3,
    extended: true,
    clues: "..............3.85..1.2.......5.7.....4...1...9.......5......73..2.1........4...9",
  },
  {
    name: "16×16, minimal encoding",
    lesson: "With weaker propagation, learning from conflicts is what makes it quick.",
    boxSize: 4,
    extended: false,
    clues: "1G.D8....3.2.9.F..6F...A5.B..G..E..8...9.14D....3.C...1.....B5.....9..D.BFE.1....B.........G..D..C3AG.84..79E...84.G5E..........A72...G....B8...5..4....3.D....69...6...15..D3.C.....8...............A.28B..GD.3.2...G4..6........51..6.D..3..C7...315..2.A..F6.",
  },
  {
    name: "25×25, as sparse as it gets",
    lesson: "Learning alone isn't enough here. Choosing what to branch on matters.",
    boxSize: 5,
    extended: true,
    clues: "....9..N...O.8K4..G.M52.D..J......C.....5.D72.O8.1.F.C.5.72M.I..N.6.....H..7..M5.1K.6C.FA.9...H...NPK.8.O9B...M.D...EP...4..F2K6O.138..5..MA..NH..P..G....B..J....3.8F.7..O.6..83...B..E.......4G........G..P...M5I.NE.D..2.9.L..A..5...2..4..C.1..8..BE.ND..2.....8.C.7....B.J........EC.F.A.LIN....DK...19..7.CM..K..E4...89...LNB..INH...P.J....1.A....M.DO..386....H.......4PGA...59..3.HE..N.261..G..F7A....E..H.C4...8..9A7...K.1O64.F.......N..P.2.6.138B9.5.D7.2..1.GJC..8......P.E.6...8.9B.7.MD5.NE.PG..4C...1K..L...7.OMNPJ.4...C..A...72MO....4EK1..9B.IL.EJ4...AC..B3....D2...K.6.LH.B3...4P..89....C5D.O..M2.D7K.6.1...5.3........J",
  },
];

/** Reads clues written in base 36, with `.` for a blank. */
export function parseClues(clues: string): Uint8Array<ArrayBuffer> {
  return Uint8Array.from(clues, (character) => (character === "." ? 0 : parseInt(character, 36)));
}
