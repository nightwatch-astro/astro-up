<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import Toast from "primevue/toast";
import { useToast } from "primevue/usetoast";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useTheme } from "./composables/useTheme";
import { useCoreEvents } from "./composables/useCoreEvents";
import { useErrorLog } from "./stores/errorLog";
import type { CoreEvent } from "./types/commands";

const version = __APP_VERSION__;
const toast = useToast();
const { init: initTheme } = useTheme();
const { addEntry } = useErrorLog();

const updateVersion = ref<string | null>(null);
let unlistenUpdate: UnlistenFn | null = null;

// Listen for core events (errors, progress)
useCoreEvents((event: CoreEvent) => {
  if (event.type === "error" || event.type === "install_failed") {
    const detail = "error" in event.data ? event.data.error : "Unknown error";
    addEntry("error", `Operation failed: ${event.data.id}`, detail);
    toast.add({
      severity: "error",
      summary: `Error: ${event.data.id}`,
      detail,
      life: 5000,
    });
  }
});

// Listen for self-update availability (T031)
async function setupUpdateListener() {
  unlistenUpdate = await listen<{ version: string; body: string | null }>(
    "update-available",
    (event) => {
      updateVersion.value = event.payload.version;
      toast.add({
        severity: "info",
        summary: `Update available: v${event.payload.version}`,
        detail: event.payload.body ?? "A new version is available.",
        life: 0, // Persist until dismissed
        closable: true,
        group: "update",
      });
    },
  );
}

async function installUpdate() {
  try {
    const { check } = await import("@tauri-apps/plugin-updater");
    const update = await check();
    if (update) {
      toast.add({
        severity: "info",
        summary: "Downloading update...",
        life: 3000,
      });
      await update.downloadAndInstall();
      // App will restart automatically after install
    }
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    addEntry("error", "Update failed", msg);
    toast.add({
      severity: "error",
      summary: "Update failed",
      detail: msg,
      life: 5000,
    });
  }
}

function dismissUpdate() {
  updateVersion.value = null;
}

onMounted(() => {
  initTheme();
  setupUpdateListener();
});

onUnmounted(() => {
  if (unlistenUpdate) {
    unlistenUpdate();
  }
});
</script>

<template>
  <Toast position="bottom-right" />

  <!-- Update notification banner (T031) -->
  <div
    v-if="updateVersion"
    class="fixed top-0 left-0 right-0 z-50 flex items-center justify-between gap-4 bg-blue-600 px-4 py-2 text-white"
  >
    <span>
      Update available: v{{ updateVersion }}
    </span>
    <div class="flex gap-2">
      <button
        class="rounded bg-white px-3 py-1 text-sm font-medium text-blue-600"
        @click="installUpdate"
      >
        Install
      </button>
      <button
        class="rounded border border-white px-3 py-1 text-sm"
        @click="dismissUpdate"
      >
        Dismiss
      </button>
    </div>
  </div>

  <main class="p-4">
    <h1 class="text-2xl font-bold">
      Astro-Up
    </h1>
    <p class="text-sm opacity-60">
      v{{ version }}
    </p>
    <p class="mt-4">
      Astrophotography software manager for Windows.
    </p>
  </main>
</template>
