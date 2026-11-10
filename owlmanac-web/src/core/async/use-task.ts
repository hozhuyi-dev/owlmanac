import { ref, shallowRef } from "vue";

/**
 * Runs async work where only the latest call counts. An earlier call finishing late is ignored, and
 * a failure leaves the last good result in place.
 */
export function useTask<Args extends unknown[], Result>(run: (...args: Args) => Promise<Result>) {
  const result = shallowRef<Result>();
  const error = shallowRef<unknown>();
  const pending = ref(false);
  let latest = 0;

  async function start(...args: Args) {
    const id = ++latest;
    pending.value = true;
    try {
      const value = await run(...args);
      if (id === latest) {
        result.value = value;
        error.value = undefined;
      }
    } catch (failure) {
      if (id === latest) error.value = failure;
    } finally {
      if (id === latest) pending.value = false;
    }
  }

  return { result, error, pending, start };
}
