import { onMounted, onUnmounted, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { logger } from "../utils/logger";
import type { CoreEvent } from "../types/commands";

/**
 * Subscribe to the "core-event" channel from astro-up-core.
 * Auto-unsubscribes when the component is unmounted.
 */
export function useCoreEvents(callback: (event: CoreEvent) => void) {
  const listening = ref(false);
  let unlisten: UnlistenFn | null = null;

  onMounted(async () => {
    try {
      unlisten = await listen<CoreEvent>("core-event", (event) => {
        callback(event.payload);
      });
      listening.value = true;
      logger.debug("useCoreEvents", "core-event listener established");
    } catch {
      logger.warn("useCoreEvents", "core-event listener unavailable (not in Tauri)");
    }
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
