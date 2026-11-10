<script setup lang="ts">
import { computed, ref } from "vue";

// Mirrors the Sudoku encoding in owlmanac-wasm/crates/sudoku/src/encode.rs, for a 9×9.
const SIDE = 9;
const BOX = 3;
const indices = Array.from({ length: SIDE }, (_, index) => index);
const digits = indices.map((index) => index + 1);

type Cell = { row: number; col: number };
type Rule = { title: string; extended: boolean; clauses: string[]; cells: Cell[] };

const row = ref(4);
const col = ref(4);
const digit = ref(5);
const chosenRule = ref(2);

const name = (cell: Cell, value: number) => `r${cell.row + 1}c${cell.col + 1}=${value}`;
const not = (text: string) => `not ${text}`;
const anyOf = (texts: string[]) => texts.join(" or ");

const variable = computed(() => name({ row: row.value, col: col.value }, digit.value));

const rules = computed<Rule[]>(() => {
  const here = { row: row.value, col: col.value };
  const me = variable.value;
  const rowCells = indices.map((c) => ({ row: here.row, col: c }));
  const colCells = indices.map((r) => ({ row: r, col: here.col }));
  const top = here.row - (here.row % BOX);
  const left = here.col - (here.col % BOX);
  const boxCells = indices.map((index) => ({ row: top + Math.floor(index / BOX), col: left + (index % BOX) }));
  const others = (cells: Cell[]) => cells.filter((cell) => cell.row !== here.row || cell.col !== here.col);
  const pairs = (cells: Cell[]) => others(cells).map((cell) => anyOf([not(me), not(name(cell, digit.value))]));
  const somewhere = (cells: Cell[]) => [anyOf(cells.map((cell) => name(cell, digit.value)))];
  const d = digit.value;

  return [
    { title: "The cell holds some digit", extended: false, clauses: [anyOf(digits.map((v) => name(here, v)))], cells: [here] },
    { title: `No other ${d} in its row`, extended: false, clauses: pairs(rowCells), cells: rowCells },
    { title: `No other ${d} in its column`, extended: false, clauses: pairs(colCells), cells: colCells },
    { title: `No other ${d} in its box`, extended: false, clauses: pairs(boxCells), cells: boxCells },
    {
      title: "The cell holds only one digit",
      extended: true,
      clauses: digits.filter((v) => v !== d).map((v) => anyOf([not(me), not(name(here, v))])),
      cells: [here],
    },
    { title: `Its row has a ${d} somewhere`, extended: true, clauses: somewhere(rowCells), cells: rowCells },
    { title: `Its column has a ${d} somewhere`, extended: true, clauses: somewhere(colCells), cells: colCells },
    { title: `Its box has a ${d} somewhere`, extended: true, clauses: somewhere(boxCells), cells: boxCells },
  ];
});

const groups = computed(() => [
  { heading: "Minimal encoding", rules: rules.value.map((rule, index) => ({ rule, index })).filter(({ rule }) => !rule.extended) },
  { heading: "Extras in the extended encoding", rules: rules.value.map((rule, index) => ({ rule, index })).filter(({ rule }) => rule.extended) },
]);
const clauseCount = (list: { rule: Rule }[]) => list.reduce((sum, { rule }) => sum + rule.clauses.length, 0);
const total = computed(() => rules.value.reduce((sum, rule) => sum + rule.clauses.length, 0));

const rule = computed(() => rules.value[chosenRule.value]);
const lit = computed(() => new Set(rule.value.cells.map((cell) => cell.row * SIDE + cell.col)));
const plural = (n: number) => `${n} ${n === 1 ? "clause" : "clauses"}`;
</script>

<template>
  <div class="explorer">
    <div class="picker">
      <p class="step">Pick a cell</p>
      <div class="grid">
        <template v-for="r in indices" :key="r">
          <button
            v-for="c in indices"
            :key="c"
            type="button"
            :class="{ chosen: r === row && c === col, lit: lit.has(r * SIDE + c) }"
            :aria-label="`Row ${r + 1}, column ${c + 1}`"
            :aria-pressed="r === row && c === col"
            @click="(row = r), (col = c)"
          />
        </template>
      </div>
      <p class="step">Pick a digit</p>
      <div class="digits" role="radiogroup" aria-label="Digit">
        <button
          v-for="value in digits"
          :key="value"
          type="button"
          role="radio"
          :aria-checked="value === digit"
          @click="digit = value"
        >
          {{ value }}
        </button>
      </div>
    </div>

    <div class="rules">
      <p class="variable">
        <code>{{ variable }}</code> is true when row {{ row + 1 }}, column {{ col + 1 }} holds a {{ digit }}.
        <strong>{{ total }}</strong> clauses mention it.
      </p>
      <section v-for="group in groups" :key="group.heading">
        <h4>{{ group.heading }} <span>· {{ plural(clauseCount(group.rules)) }}</span></h4>
        <ul>
          <li v-for="{ rule: item, index } in group.rules" :key="index">
            <button type="button" :aria-pressed="chosenRule === index" @click="chosenRule = index">
              <span>{{ item.title }}</span>
              <span class="count">{{ plural(item.clauses.length) }}</span>
            </button>
          </li>
        </ul>
      </section>
      <div class="example" aria-live="polite">
        <span class="label">One of its clauses</span>
        <div class="clause">
          <!-- The longest clause, invisible, keeps the box the same height whichever rule is chosen. -->
          <code class="sizer" aria-hidden="true">{{ rules[0].clauses[0] }}</code>
          <code>{{ rule.clauses[0] }}</code>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.explorer {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr));
  gap: 1.5rem 2rem;
}

.step {
  margin: 0 0 0.4rem;
  color: var(--muted);
  font-size: 0.875rem;
}

.grid {
  display: grid;
  grid-template-columns: repeat(9, minmax(0, 1fr));
  width: 100%;
  max-width: 20rem;
  margin-bottom: 1rem;
  border: 2px solid var(--fg);
}

.grid button {
  aspect-ratio: 1;
  padding: 0;
  border: 0;
  border-right: 1px solid var(--line);
  border-bottom: 1px solid var(--line);
  background: var(--bg);
  cursor: pointer;
}

.grid button:nth-child(9n) {
  border-right: 0;
}

.grid button:nth-child(3n):not(:nth-child(9n)) {
  border-right: 2px solid var(--fg);
}

.grid button:nth-child(n + 19):nth-child(-n + 27),
.grid button:nth-child(n + 46):nth-child(-n + 54) {
  border-bottom: 2px solid var(--fg);
}

.grid button:nth-child(n + 73) {
  border-bottom: 0;
}

.grid button.lit {
  background: color-mix(in srgb, var(--accent) 25%, var(--bg));
}

.grid button.chosen {
  background: var(--accent);
}

.grid button:focus-visible,
.digits button:focus-visible {
  outline: 2px solid var(--fg);
  outline-offset: -2px;
}

.digits {
  display: grid;
  grid-template-columns: repeat(9, minmax(0, 1fr));
  max-width: 20rem;
  border: 1px solid var(--line);
  border-radius: 0.375rem;
  overflow: hidden;
}

.digits button {
  padding: 0.4rem 0;
  border: 0;
  border-left: 1px solid var(--line);
  background: var(--surface);
  color: var(--fg);
  font: inherit;
  cursor: pointer;
}

.digits button:first-child {
  border-left: 0;
}

.digits button[aria-checked="true"] {
  background: var(--accent);
  color: var(--bg);
  font-weight: 600;
}

.variable {
  margin: 0 0 1rem;
}

h4 {
  margin: 0.75rem 0 0.25rem;
  font-size: 0.875rem;
}

h4 span {
  color: var(--muted);
  font-weight: 400;
}

ul {
  margin: 0;
  padding: 0;
  list-style: none;
}

li button {
  display: flex;
  gap: 0.75rem;
  justify-content: space-between;
  width: 100%;
  padding: 0.3rem 0.5rem;
  border: 0;
  border-left: 3px solid transparent;
  background: none;
  color: var(--fg);
  font: inherit;
  font-size: 0.95rem;
  text-align: left;
  cursor: pointer;
}

li button:hover {
  background: var(--surface);
}

li button[aria-pressed="true"] {
  border-left-color: var(--accent);
  background: var(--surface);
}

.count {
  flex-shrink: 0;
  color: var(--muted);
  font-size: 0.875rem;
}

.example {
  margin-top: 1rem;
  padding: 0.6rem 0.75rem;
  border-radius: 0.375rem;
  background: var(--surface);
}

.example .label {
  display: block;
  margin-bottom: 0.25rem;
  color: var(--muted);
  font-size: 0.8rem;
}

.clause {
  display: grid;
}

.clause code {
  grid-area: 1 / 1;
  font-size: 0.8rem;
  overflow-wrap: anywhere;
}

.sizer {
  visibility: hidden;
}
</style>
