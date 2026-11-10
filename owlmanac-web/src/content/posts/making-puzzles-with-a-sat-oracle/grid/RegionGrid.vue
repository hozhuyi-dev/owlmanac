<script setup lang="ts">
import { ref } from "vue";

// A Star Battle grid: thick lines between regions, optional region colours, and a symbol per cell.
// With `paintable`, pressing and dragging across cells emits each cell once.
const props = defineProps<{
  size: number;
  /** Each cell's region, row by row. */
  regions: number[];
  /** Colour per region; regions are left plain without it. */
  colours?: string[];
  /** Text to show in cells, by cell index. */
  symbols?: Record<number, string>;
  /** Cells to draw attention to, by cell index. */
  marked?: Set<number>;
  paintable?: boolean;
  label: string;
}>();

const emit = defineEmits<{ paint: [index: number] }>();

const indices = Array.from({ length: props.size * props.size }, (_, index) => index);
const painting = ref(false);

function edges(index: number) {
  const { size, regions } = props;
  const row = Math.floor(index / size);
  const col = index % size;
  return {
    "edge-top": row > 0 && regions[index - size] !== regions[index],
    "edge-left": col > 0 && regions[index - 1] !== regions[index],
  };
}

function start(index: number) {
  if (!props.paintable) return;
  painting.value = true;
  emit("paint", index);
}

function enter(index: number) {
  if (props.paintable && painting.value) emit("paint", index);
}
</script>

<template>
  <div
    class="grid"
    :class="{ paintable }"
    :style="{ '--size': size }"
    role="img"
    :aria-label="label"
    @pointerup="painting = false"
    @pointerleave="painting = false"
  >
    <span
      v-for="index in indices"
      :key="index"
      :class="[edges(index), { marked: marked?.has(index) }]"
      :style="colours ? { background: `color-mix(in srgb, ${colours[regions[index]]} 28%, var(--bg))` } : undefined"
      @pointerdown.prevent="start(index)"
      @pointerenter="enter(index)"
      >{{ symbols?.[index] ?? "" }}</span
    >
  </div>
</template>

<style scoped>
.grid {
  display: grid;
  grid-template-columns: repeat(var(--size), minmax(0, 1fr));
  width: 100%;
  max-width: 22rem;
  border: 2px solid var(--fg);
  touch-action: none;
  user-select: none;
}

.grid span {
  display: grid;
  aspect-ratio: 1;
  place-items: center;
  border-top: 1px solid var(--line);
  border-left: 1px solid var(--line);
  font-size: clamp(0.8rem, 3vw, 1.1rem);
  line-height: 1;
}

.paintable span {
  cursor: crosshair;
}

.grid span.edge-top {
  border-top: 2px solid var(--fg);
}

.grid span.edge-left {
  border-left: 2px solid var(--fg);
}

.marked {
  color: var(--danger);
  font-weight: 700;
}
</style>
