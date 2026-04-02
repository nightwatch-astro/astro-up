import { onUnmounted, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { CoreEvent } from "../types/commands";

/**
 * Subscribe to the "core-event" channel from astro-up-core.
 * Auto-unsubscribes when the component is unmounted.
 */
export function useCoreEvents(callback: (event: CoreEvent) => void) {
  const listening = ref(false);
  let unlisten: UnlistenFn | null = null;

  listen<CoreEvent>("core-event", (event) => {
    callback(event.payload);
  }).then((fn) => {
    unlisten = fn;
    listening.value = true;
  });

  onUnmounted(() => {
    if (unlisten) {
      unlisten();
      unlisten = null;
      listening.value = false;
    }
  });

  return { listening };
}
