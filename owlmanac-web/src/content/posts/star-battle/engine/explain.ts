import type { Cell, Placed, Rule, Step, WhyNot } from "@/wasm/star-battle.types";

// Turns the solver's answers into sentences a player can follow.

export const cellName = (cell: Cell) => `row ${cell.row + 1}, column ${cell.col + 1}`;

const capitalised = (text: string) => text[0].toUpperCase() + text.slice(1);

/** Why a rule forced a cell: a full unit has no room left, an emptying unit needs this cell. */
function becauseOf(rule: Rule, star: boolean): string {
  switch (rule.kind) {
    case "row":
      return star ? `row ${rule.at + 1} has nowhere else left for its stars` : `row ${rule.at + 1} already has its stars`;
    case "column":
      return star
        ? `column ${rule.at + 1} has nowhere else left for its stars`
        : `column ${rule.at + 1} already has its stars`;
    case "region":
      return star ? "its region has nowhere else left for its stars" : "its region already has its stars";
    case "noTouching":
      return "it touches a star";
  }
}

function ruleName(rule: Rule): string {
  switch (rule.kind) {
    case "row":
      return `row ${rule.at + 1} can't get the right number of stars`;
    case "column":
      return `column ${rule.at + 1} can't get the right number of stars`;
    case "region":
      return "a region can't get the right number of stars";
    case "noTouching":
      return "two stars would touch";
  }
}

export function stepSentence(step: Step): string {
  const star = step.placed.mark === "star";
  return `${capitalised(cellName(step.placed.cell))} ${star ? "must be a star" : "can't be a star"}: ${becauseOf(step.rule, star)}.`;
}

export const markText = (placed: Placed) => `${placed.mark === "star" ? "a star" : "no star"} at ${cellName(placed.cell)}`;

export type Explanation = { lines: string[]; cells: Cell[] };

/** An answer to "why can't this go here?", as numbered steps and the cells they involve. */
export function explain(answer: WhyNot, proposal: Placed): Explanation {
  const asked = markText(proposal);
  switch (answer.kind) {
    case "possible":
      return { lines: [`Nothing rules out ${asked} yet: some way of finishing the board has it.`], cells: [proposal.cell] };
    case "deduced":
      return {
        lines: [
          `Suppose ${asked}.`,
          ...answer.steps.map(stepSentence),
          `Then ${ruleName(answer.clash.rule)}. So that can't be right.`,
        ],
        cells: [proposal.cell, ...answer.steps.map((step) => step.placed.cell), ...answer.clash.marks.map((mark) => mark.cell)],
      };
    case "needsCases":
      return {
        lines: [
          `No short chain of forced moves rules out ${asked}; it takes trying cases.`,
          `These can't all hold: ${answer.marks.map(markText).join(", ")}.`,
        ],
        cells: answer.marks.map((mark) => mark.cell),
      };
  }
}

export function depthText(depth: number): string {
  if (depth === 0) return "It's forced: there's no other way.";
  if (depth === 1) return "Suppose the opposite, and the forced moves soon clash.";
  return "This one takes a “what if” inside a “what if”.";
}
