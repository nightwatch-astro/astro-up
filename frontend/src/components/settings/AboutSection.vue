<script setup lang="ts">
import { ref } from "vue";
import Button from "primevue/button";
import { useConfigSnapshots } from "../../stores/configSnapshots";

const version = __APP_VERSION__;
const checking = ref(false);
const { snapshots, restore, remove } = useConfigSnapshots();

defineEmits<{
  restoreSnapshot: [config: Record<string, unknown>];
}>();

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

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
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

    <!-- Config Snapshots (FR-040) -->
    <section
      v-if="snapshots.length > 0"
      class="snapshots-section"
    >
      <h4 class="section-title">
        Config Snapshots
      </h4>
      <div
        v-for="snap in snapshots"
        :key="snap.id"
        class="snapshot-item"
      >
        <span class="snapshot-date">{{ formatDate(snap.timestamp) }}</span>
        <div class="snapshot-actions">
          <Button
            label="Restore"
            size="small"
            severity="secondary"
            outlined
            @click="$emit('restoreSnapshot', restore(snap.id) as unknown as Record<string, unknown>)"
          />
          <Button
            icon="pi pi-trash"
            size="small"
            severity="danger"
            text
            rounded
            @click="remove(snap.id)"
          />
        </div>
      </div>
    </section>
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

.snapshots-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
  border-top: 1px solid var(--p-surface-700);
  padding-top: 16px;
}

.section-title {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--p-surface-300);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.snapshot-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 0;
}

.snapshot-date {
  font-size: 13px;
  color: var(--p-surface-300);
}

.snapshot-actions {
  display: flex;
  gap: 4px;
}
</style>
