import { useStorage } from "@vueuse/core";
import type { AppConfig } from "../types/config";
import type { ConfigSnapshot } from "../types/operations";

const MAX_SNAPSHOTS = 10;

const snapshots = useStorage<ConfigSnapshot[]>("astro-up-config-snapshots", []);

export function useConfigSnapshots() {
  function save(config: AppConfig) {
    const snapshot: ConfigSnapshot = {
      id: crypto.randomUUID(),
      timestamp: new Date().toISOString(),
      config: config as unknown as Record<string, unknown>,
    };
    snapshots.value.unshift(snapshot);
    if (snapshots.value.length > MAX_SNAPSHOTS) {
      snapshots.value.length = MAX_SNAPSHOTS;
    }
  }

  function restore(id: string): AppConfig | null {
    const snapshot = snapshots.value.find((s) => s.id === id);
    if (!snapshot) return null;
    return snapshot.config as unknown as AppConfig;
  }

  function remove(id: string) {
    snapshots.value = snapshots.value.filter((s) => s.id !== id);
  }

  return {
    snapshots,
    save,
    restore,
    remove,
  };
}
