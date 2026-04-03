<script setup lang="ts">
import { computed } from "vue";
import { useSoftwareList, useUpdateCheck } from "../../composables/useInvoke";
import { useOperations } from "../../composables/useOperations";

defineEmits<{
  toggleLog: [];
}>();

const { data: software } = useSoftwareList(() => "all");
const { data: updates } = useUpdateCheck();
const { operation, isRunning } = useOperations();

const catalogCount = computed(() => software.value?.length ?? 0);
const installedCount = computed(() => {
  if (!software.value) return 0;
  return (software.value as Array<{ detection?: { type: string } }>).filter(
    (p) => p.detection?.type === "Installed" || p.detection?.type === "InstalledUnknownVersion",
  ).length;
});
const updateCount = computed(() => updates.value?.length ?? 0);

// TODO: persist scan timestamp to show real "Last scan: 14:30" (#TBD)
const lastSync = "Never";
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
        Last scan: {{ lastSync }}
      </span>
      <template v-if="isRunning">
        <span class="status-separator" />
        <span class="status-item has-op">
          <i class="pi pi-spinner pi-spin" />
          {{ operation?.label }}
        </span>
      </template>
    </div>

    <div class="status-actions">
      <button
        class="log-toggle"
        title="Toggle log panel (Ctrl+L)"
        @click="$emit('toggleLog')"
      >
        <i class="pi pi-list" />
        Log
      </button>
    </div>
  </div>
</template>

<style scoped>
.app-status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 26px;
  padding: 0 16px;
  background: var(--p-surface-900);
  border-top: 1px solid var(--p-surface-700);
  font-size: 11px;
  color: var(--p-surface-500);
  flex-shrink: 0;
}

.status-items {
  display: flex;
  align-items: center;
  gap: 16px;
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

.status-item.has-op {
  color: var(--p-blue-400);
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

.log-toggle {
  background: transparent;
  border: 1px solid var(--p-surface-600);
  border-radius: 4px;
  color: var(--p-surface-400);
  font-size: 11px;
  padding: 1px 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
}

.log-toggle:hover {
  border-color: var(--p-primary-400);
  color: var(--p-primary-400);
}
</style>
