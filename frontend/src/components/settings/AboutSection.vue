<script setup lang="ts">
import { ref } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import { useToast } from "primevue/usetoast";
import { open as openUrl } from "@tauri-apps/plugin-shell";

const toast = useToast();

const version = __APP_VERSION__;
const checking = ref(false);
const updateAvailable = ref(false);
const updateVersion = ref("");
const updateNotes = ref("");
const showNotes = ref(false);

async function checkForUpdates() {
  checking.value = true;
  try {
    const { check } = await import("@tauri-apps/plugin-updater");
    const update = await check();
    if (update) {
      updateAvailable.value = true;
      updateVersion.value = update.version;
      updateNotes.value = update.body ?? "";
      showNotes.value = true;
    } else {
      updateAvailable.value = false;
      updateVersion.value = "";
      updateNotes.value = "";
      showNotes.value = false;
      toast.add({ severity: "info", summary: "Up to date", detail: "You are on the latest version.", life: 3000 });
    }
  } catch {
    toast.add({ severity: "error", summary: "Update check failed", detail: "Failed to check for updates.", life: 5000 });
  } finally {
    checking.value = false;
  }
}

async function installUpdate() {
  try {
    const { check } = await import("@tauri-apps/plugin-updater");
    const update = await check();
    if (update) {
      showNotes.value = false;
      await update.downloadAndInstall();
      const { relaunch } = await import("@tauri-apps/plugin-process");
      await relaunch();
    }
  } catch {
    toast.add({ severity: "error", summary: "Update failed", detail: "Failed to install update.", life: 5000 });
  }
}
</script>

<template>
  <div class="settings-section">
    <div class="about-info">
      <h3 class="about-name">
        Astro-Up
      </h3>
      <span class="about-version">Version {{ version }}</span>
    </div>

    <Button
      label="Check for App Updates"
      icon="pi pi-refresh"
      severity="secondary"
      outlined
      :loading="checking"
      @click="checkForUpdates"
    />

    <div class="about-links">
      <button
        class="about-link"
        @click="openUrl('https://github.com/nightwatch-astro/astro-up')"
      >
        <i class="pi pi-github" /> GitHub
      </button>
    </div>

    <Dialog
      v-model:visible="showNotes"
      :header="`Update available: v${updateVersion}`"
      modal
      :style="{ width: '500px', maxHeight: '80vh' }"
    >
      <pre
        v-if="updateNotes"
        class="release-notes"
      >{{ updateNotes }}</pre>
      <p v-else>
        A new version is available.
      </p>

      <template #footer>
        <Button
          label="Later"
          severity="secondary"
          text
          @click="showNotes = false"
        />
        <Button
          label="Install & Restart"
          icon="pi pi-download"
          @click="installUpdate"
        />
      </template>
    </Dialog>
  </div>
</template>

<style scoped>
.settings-section { display: flex; flex-direction: column; gap: 16px; }
.about-info { display: flex; flex-direction: column; gap: 2px; }
.about-name { margin: 0; font-size: 18px; font-weight: 700; color: var(--p-surface-0); }
.about-version { font-size: 13px; color: var(--p-surface-400); }
.about-links { display: flex; gap: 16px; }
.about-link {
  font-size: 13px;
  color: var(--p-primary-400);
  text-decoration: none;
  display: flex;
  align-items: center;
  gap: 4px;
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
}
.about-link:hover { text-decoration: underline; }
.release-notes {
  font-family: inherit;
  font-size: 13px;
  line-height: 1.6;
  color: var(--p-surface-200);
  white-space: pre-wrap;
  word-wrap: break-word;
  margin: 0;
  max-height: 400px;
  overflow-y: auto;
}
</style>
