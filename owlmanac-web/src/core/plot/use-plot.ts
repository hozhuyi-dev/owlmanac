import { onBeforeUnmount, onMounted, watch, type Ref, type WatchSource } from "vue";

type Render = (width: number) => Element | undefined | Promise<Element | undefined>;

/**
 * Draws a chart into `el` at the container's width, again when it resizes or `deps` change.
 * `el` holds only the chart: Vue must not render children into it.
 */
export function usePlot(el: Ref<HTMLElement | null>, render: Render, deps: WatchSource[] = []) {
  let width = 0;
  let latest = 0;
  let observer: ResizeObserver | undefined;

  async function draw() {
    if (!el.value || !width) return;
    const id = ++latest;
    const chart = await render(width);
    // A later draw started while this one waited for data: let it win.
    if (id === latest && el.value && chart) el.value.replaceChildren(chart);
  }

  onMounted(() => {
    observer = new ResizeObserver(([entry]) => {
      const next = Math.floor(entry.contentRect.width);
      if (next !== width) {
        width = next;
        draw();
      }
    });
    observer.observe(el.value!);
  });
  onBeforeUnmount(() => observer?.disconnect());
  watch(deps, draw);
}
