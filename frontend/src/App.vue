<script setup lang="ts">
import { onMounted } from "vue";
import Toast from "primevue/toast";
import { useToast } from "primevue/usetoast";
import { useTheme } from "./composables/useTheme";
import { useCoreEvents } from "./composables/useCoreEvents";
import { useErrorLog } from "./stores/errorLog";
import type { CoreEvent } from "./types/commands";

const version = __APP_VERSION__;
const toast = useToast();
const { init: initTheme } = useTheme();
const { addEntry } = useErrorLog();

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

onMounted(() => {
  initTheme();
});
</script>

<template>
  <Toast position="bottom-right" />
  <main class="p-4">
    <h1 class="text-2xl font-bold">Astro-Up</h1>
    <p class="text-sm opacity-60">v{{ version }}</p>
    <p class="mt-4">Astrophotography software manager for Windows.</p>
  </main>
</template>
