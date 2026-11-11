<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import type { Cell } from "@/wasm/star-battle.types";
import { SIZE, useGame } from "../engine/use-game";
import type { Level } from "../engine/pool";

const game = useGame();
const askingWhy = ref(false);
// What a reader does to a cell: taps it on a touch screen, else clicks it. Set once the island is in the
// browser; the line it's in has a fixed height, so the word changing never moves anything.
const verb = ref("Click");
onMounted(() => {
  if (matchMedia("(hover: none)").matches) verb.value = "Tap";
});

const indices = Array.from({ length: SIZE * SIZE }, (_, index) => index);
const at = (cell: Cell) => cell.row * SIZE + cell.col;
// Before a puzzle loads, a plain grid with no region lines, so the board already has its size.
const regions = computed(() => game.puzzle.value?.regions ?? indices.map(() => 0));

function edges(index: number) {
  const here = regions.value[index];
  return {
    "edge-top": index >= SIZE && regions.value[index - SIZE] !== here,
    "edge-left": index % SIZE > 0 && regions.value[index - 1] !== here,
  };
}

const blamed = computed(() => new Set(game.verdict.value?.blame?.marks.map((mark) => at(mark.cell))));
const highlighted = computed(() => new Set(game.answer.value?.explanation.cells.map(at)));
const hintCell = computed(() => {
  const hint = game.answer.value?.hint;
  return hint ? at(hint.placed.cell) : undefined;
});

function press(index: number) {
  if (askingWhy.value) game.askWhyNot(index);
  else game.cycleMark(index);
}

function choose(level: Level) {
  askingWhy.value = false;
  game.newPuzzle(level);
}

// Nothing can be asked of a puzzle that's on its way out.
const ready = computed(() => game.puzzle.value !== undefined && !game.loading.value);
// Until the first puzzle is in, or while the next one comes, the owl works over the board.
const waiting = computed(() => !ready.value && !game.loadingError.value);

// In a narrow figure, a level's name is cut to `short` (the rest hides with `.long`).
const LEVELS: { level: Level; short: string; rest: string }[] = [
  { level: "easy", short: "Easy", rest: "" },
  { level: "medium", short: "Med", rest: "ium" },
  { level: "hard", short: "Hard", rest: "" },
];

const SYMBOLS: Record<number, string> = { 0: "", 1: "★", [-1]: "×" };
const LABELS: Record<number, string> = { 0: "unmarked", 1: "star", [-1]: "ruled out" };

const status = computed(() => {
  if (game.loadingError.value) return "The puzzles couldn't be loaded.";
  if (game.solverError.value) return "The solver stopped with an error.";
  if (!game.puzzle.value || game.loading.value) return "Choosing a puzzle…";
  if (game.solved.value) return "Solved! Every row, column and region has its two stars.";
  const verdict = game.verdict.value;
  if (verdict && !verdict.solvable) return "This board can't be finished: the marks in red can't all stand.";
  return "Two stars in every row, column and region, and no two touching.";
});

const lines = computed(() => {
  if (game.thinking.value) return ["Thinking…"];
  if (game.answer.value) return game.answer.value.explanation.lines;
  if (askingWhy.value) return [`${verb.value} a cell to ask why a star can't go there.`];
  return [`${verb.value} a cell to cycle it: star, ruled out, empty. Stuck? Ask for a hint.`];
});
</script>

<template>
  <div class="game">
    <div class="chart-controls toolbar">
      <div class="levels" role="radiogroup" aria-label="Level">
        <button
          v-for="{ level, short, rest } in LEVELS"
          :key="level"
          type="button"
          role="radio"
          :aria-checked="game.level.value === level"
          @click="choose(level)"
        >
          {{ short }}<span class="long">{{ rest }}</span>
        </button>
      </div>
      <button type="button" @click="choose(game.level.value)">New<span class="long"> puzzle</span></button>
      <button type="button" :disabled="!ready" @click="game.askHint">Hint</button>
      <button type="button" :aria-pressed="askingWhy" :disabled="!ready" @click="askingWhy = !askingWhy">
        <span class="long">Ask why</span><span class="short">Why?</span>
      </button>
      <button type="button" :disabled="!ready" @click="game.clear">Clear</button>
    </div>

    <div class="play">
      <div class="board-frame">
        <div class="board" :class="{ asking: askingWhy }">
          <button
            v-for="index in indices"
            :key="index"
            type="button"
            :class="[
              edges(index),
              {
                blamed: blamed.has(index),
                highlighted: highlighted.has(index),
                hinted: hintCell === index,
              },
            ]"
            :disabled="!ready"
            :aria-label="`Row ${Math.floor(index / SIZE) + 1}, column ${(index % SIZE) + 1}: ${LABELS[game.marks.value[index]]}`"
            @click="press(index)"
          >
            {{ SYMBOLS[game.marks.value[index]] }}
          </button>
        </div>
        <div v-if="waiting" class="busy waiting" aria-hidden="true" />
      </div>

      <div class="side">
        <p class="figure-status" :class="{ failed: game.loadingError.value || game.solverError.value }" role="status">
          {{ status }}
        </p>
        <ol class="explanation" aria-live="polite">
          <li v-for="(line, index) in lines" :key="index">{{ line }}</li>
        </ol>
      </div>
    </div>
  </div>
</template>

<style scoped>
.levels {
  display: inline-flex;
}

.levels button {
  border-radius: 0;
}

.levels button:first-child {
  border-radius: 0.375rem 0 0 0.375rem;
}

.levels button + button {
  border-left: 0;
}

.levels button:last-child {
  border-radius: 0 0.375rem 0.375rem 0;
}

.levels button[aria-checked="true"],
.chart-controls button[aria-pressed="true"] {
  border-color: var(--accent);
  background: color-mix(in srgb, var(--accent) 18%, var(--bg));
}

/* Every control on one row, at any width. In a narrow figure (a phone) each is a thumb's height, they share
   the width, and the long labels are cut: "New puzzle" is "New", "Medium" is "Med", "Ask why" is "Why?".
   Measured on the figure, not the screen. */
.game {
  container-type: inline-size;
}

.toolbar {
  flex-wrap: nowrap;
  gap: 0.5rem;
}

.toolbar button {
  white-space: nowrap;
}

.short {
  display: none;
}

@container (max-width: 30rem) {
  .toolbar {
    gap: 0.375rem;
    font-size: 0.8125rem;
  }

  .toolbar button {
    flex: 1 1 auto;
    min-height: 2.5rem;
    padding: 0.3rem 0.2rem;
  }

  .toolbar .levels {
    flex: 2 1 auto;
  }

  .long {
    display: none;
  }

  .short {
    display: inline;
  }
}

.play {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr));
  gap: 0.75rem 1.5rem;
  align-items: start;
  margin-top: 0.75rem;
}

.board-frame {
  position: relative;
  max-width: 28rem;
}

.board {
  display: grid;
  grid-template-columns: repeat(10, minmax(0, 1fr));
  width: 100%;
  border: 2px solid var(--fg);
  /* A board is tapped, often quickly: no flash, no text selection or callout on a long press. */
  -webkit-tap-highlight-color: transparent;
  -webkit-touch-callout: none;
  user-select: none;
}

/* The owl at work over the board, faded in only if the wait is long enough to notice. */
.waiting {
  position: absolute;
  inset: 0;
  border-radius: 0;
  background: color-mix(in srgb, var(--surface) 94%, transparent);
  animation: waiting-in 0.2s ease-out 0.15s backwards;
}

@keyframes waiting-in {
  from {
    opacity: 0;
  }
}

.board button {
  aspect-ratio: 1;
  padding: 0;
  border: 0;
  border-top: 1px solid var(--line);
  border-left: 1px solid var(--line);
  background: var(--bg);
  color: var(--fg);
  font-size: clamp(0.9rem, 3.5vw, 1.3rem);
  line-height: 1;
  cursor: pointer;
}

.board button:disabled {
  cursor: default;
}

.board.asking button:not(:disabled) {
  cursor: help;
}

/* Only where there's a pointer to hover: on a touch screen, hover sticks to the last cell tapped. */
@media (hover: hover) {
  .board button:hover:not(:disabled) {
    background: var(--surface);
  }
}

.board button:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: -2px;
}

.board button.edge-top {
  border-top: 2px solid var(--fg);
}

.board button.edge-left {
  border-left: 2px solid var(--fg);
}

.board button.highlighted {
  background: color-mix(in srgb, var(--accent) 22%, var(--bg));
}

.board button.hinted {
  background: color-mix(in srgb, var(--accent) 45%, var(--bg));
}

.board button.blamed {
  color: var(--danger);
}

/* The explanation scrolls inside a fixed height, so a long chain never moves the page. Shorter on a
   phone, so the board and what's said about it fit on one screen. */
.explanation {
  height: 14rem;
  margin: 0;
  padding-left: 1.25rem;
  overflow-y: auto;
  font-size: 0.9rem;
}

@container (max-width: 30rem) {
  .explanation {
    height: 9rem;
  }
}

.explanation li + li {
  margin-top: 0.3rem;
}

.figure-status {
  min-height: 3em;
  margin-top: 0;
}
</style>
