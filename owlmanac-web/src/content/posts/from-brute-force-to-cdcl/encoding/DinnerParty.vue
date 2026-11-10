<script setup lang="ts">
import { computed, ref } from "vue";

type Guest = "Alice" | "Bob" | "Carol";
type Literal = { guest: Guest; coming: boolean };

const GUESTS: Guest[] = ["Alice", "Bob", "Carol"];
const CLAUSES: { rule: string; literals: Literal[] }[] = [
  {
    rule: "Someone has to come.",
    literals: [
      { guest: "Alice", coming: true },
      { guest: "Bob", coming: true },
      { guest: "Carol", coming: true },
    ],
  },
  {
    rule: "Alice and Bob aren't speaking.",
    literals: [
      { guest: "Alice", coming: false },
      { guest: "Bob", coming: false },
    ],
  },
  {
    rule: "Carol only comes if Bob does.",
    literals: [
      { guest: "Carol", coming: false },
      { guest: "Bob", coming: true },
    ],
  },
];
const SOLUTION_COUNT = 3;

const coming = ref<Record<Guest, boolean>>({ Alice: false, Bob: false, Carol: false });
const found = ref<string[]>([]);

const holds = (literal: Literal) => coming.value[literal.guest] === literal.coming;
const clauseHolds = computed(() => CLAUSES.map((clause) => clause.literals.some(holds)));
const solved = computed(() => clauseHolds.value.every(Boolean));
const guestList = computed(() => GUESTS.filter((guest) => coming.value[guest]).join(" and ") || "nobody");

function toggle(guest: Guest) {
  coming.value = { ...coming.value, [guest]: !coming.value[guest] };
  if (solved.value && !found.value.includes(guestList.value)) found.value = [...found.value, guestList.value];
}

const status = computed(() => {
  if (solved.value) return `Every clause holds: inviting ${guestList.value} works.`;
  const broken = clauseHolds.value.filter((ok) => !ok).length;
  return `${broken} of ${CLAUSES.length} clauses ${broken === 1 ? "is" : "are"} broken.`;
});

const word = (literal: Literal) => (literal.coming ? literal.guest : `not ${literal.guest}`);
</script>

<template>
  <div class="party">
    <div class="guests" role="group" aria-label="Who's coming">
      <button
        v-for="guest in GUESTS"
        :key="guest"
        type="button"
        :aria-pressed="coming[guest]"
        @click="toggle(guest)"
      >
        <span class="name">{{ guest }}</span>
        <span class="answer">{{ coming[guest] ? "coming" : "not coming" }}</span>
      </button>
    </div>

    <ol class="clauses">
      <li v-for="(clause, index) in CLAUSES" :key="index" :class="{ broken: !clauseHolds[index] }">
        <span class="rule">{{ clause.rule }}</span>
        <span class="clause">
          <template v-for="(literal, position) in clause.literals" :key="position">
            <span v-if="position" class="or"> or </span>
            <span :class="holds(literal) ? 'true' : 'false'">{{ word(literal) }}</span>
          </template>
        </span>
        <span class="verdict">{{ clauseHolds[index] ? "holds" : "broken" }}</span>
      </li>
    </ol>

    <p class="figure-status" role="status">{{ status }}</p>
    <p class="found">Solutions found: {{ found.length }} of {{ SOLUTION_COUNT }}</p>
    <ul class="slots">
      <li v-for="slot in SOLUTION_COUNT" :key="slot" :class="{ empty: !found[slot - 1] }">
        {{ found[slot - 1] ?? "?" }}
      </li>
    </ul>
  </div>
</template>

<style scoped>
.guests {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.5rem;
  max-width: 28rem;
}

.guests button {
  display: grid;
  gap: 0.1rem;
  padding: 0.5rem;
  border: 1px solid var(--line);
  border-radius: 0.375rem;
  background: var(--surface);
  color: var(--fg);
  font: inherit;
  cursor: pointer;
}

.guests button[aria-pressed="true"] {
  border-color: var(--accent);
  background: color-mix(in srgb, var(--accent) 18%, var(--bg));
}

.name {
  font-weight: 600;
}

.answer {
  color: var(--muted);
  font-size: 0.8rem;
}

.clauses {
  margin: 1rem 0 0;
  padding: 0;
  list-style: none;
}

.clauses li {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 0 1rem;
  padding: 0.4rem 0;
  border-bottom: 1px solid var(--line);
}

.rule {
  color: var(--muted);
  font-size: 0.875rem;
}

.clause {
  grid-column: 1;
  font-family: var(--font-mono);
  font-size: 0.875rem;
}

.true {
  color: var(--accent);
  font-weight: 600;
}

.false {
  color: var(--muted);
  text-decoration: line-through;
}

.or {
  color: var(--muted);
}

.verdict {
  grid-row: 1 / 3;
  grid-column: 2;
  align-self: center;
  min-width: 4rem;
  font-size: 0.875rem;
  text-align: right;
}

.broken .verdict {
  color: var(--danger);
}

.found {
  margin: 0 0 0.4rem;
  font-size: 0.875rem;
}

/* A slot for every solution from the start, so finding one never moves anything. */
.slots {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.5rem;
  max-width: 28rem;
  margin: 0;
  padding: 0;
  list-style: none;
}

.slots li {
  overflow: hidden;
  padding: 0.2rem 0.5rem;
  border-radius: 1rem;
  background: var(--surface);
  font-size: 0.875rem;
  text-align: center;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.slots .empty {
  color: var(--muted);
}
</style>
