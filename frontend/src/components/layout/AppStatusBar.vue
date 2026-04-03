<script setup lang="ts">
import { computed } from "vue";
import { useSoftwareList, useUpdateCheck } from "../../composables/useInvoke";

defineEmits<{
  toggleLog: [];
}>();

const { data: software } = useSoftwareList(() => "all");
const { data: updates } = useUpdateCheck();

const catalogCount = computed(() => software.value?.length ?? 0);
const installedCount = computed(() => {
  if (!software.value) return 0;
  return (software.value as Array<{ detection?: { type: string } }>).filter(
    (p) => p.detection?.type === "Installed" || p.detection?.type === "InstalledUnknownVersion",
  ).length;
});
const updateCount = computed(() => updates.value?.length ?? 0);

const lastSync = computed(() => {
  if (!software.value) return "Never";
  return new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
});
</script>

<template>
  <div class="app-status-bar">
    <div class="status-items">
      <span class="status-item">
        <i class="pi pi-th-large" />
        {{ catalogCount }} packages
      </span>
      <span class="status-separator" />
      <span class="status-item">
        <i class="pi pi-check-circle" />
        {{ installedCount }} installed
      </span>
      <span class="status-separator" />
      <span
        class="status-item"
        :class="{ 'has-updates': updateCount > 0 }"
      >
        <i class="pi pi-arrow-up" />
        {{ updateCount }} updates
      </span>
      <span class="status-separator" />
      <span class="status-item">
        <i class="pi pi-sync" />
        {{ lastSync }}
      </span>
    </div>

    <div class="status-actions">
      <button
        class="status-action"
        title="Toggle log panel"
        @click="$emit('toggleLog')"
      >
        <i class="pi pi-list" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.app-status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 28px;
  padding: 0 12px;
  background: var(--p-surface-900);
  border-top: 1px solid var(--p-surface-700);
  font-size: 12px;
  color: var(--p-surface-400);
  flex-shrink: 0;
}

.status-items {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.status-item i {
  font-size: 11px;
}

.status-item.has-updates {
  color: var(--p-yellow-400);
}

.status-separator {
  width: 1px;
  height: 12px;
  background: var(--p-surface-700);
}

.status-actions {
  display: flex;
  align-items: center;
}

.status-action {
  background: none;
  border: none;
  color: var(--p-surface-400);
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 12px;
}

.status-action:hover {
  background: var(--p-surface-800);
  color: var(--p-surface-0);
}
</style>
