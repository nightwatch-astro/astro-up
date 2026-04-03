<script setup lang="ts">
import { ref } from "vue";
import Button from "primevue/button";

const version = __APP_VERSION__;
const checking = ref(false);

async function checkForUpdates() {
  checking.value = true;
  try {
    const { check } = await import("@tauri-apps/plugin-updater");
    const update = await check();
    if (update) {
      alert(`Update available: v${update.version}`);
    } else {
      alert("You are on the latest version.");
    }
  } catch {
    alert("Failed to check for updates.");
  } finally {
    checking.value = false;
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
      <a
        href="https://github.com/nightwatch-astro/astro-up"
        target="_blank"
        class="about-link"
      >
        <i class="pi pi-github" /> GitHub
      </a>
    </div>
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
}
.about-link:hover { text-decoration: underline; }
</style>
