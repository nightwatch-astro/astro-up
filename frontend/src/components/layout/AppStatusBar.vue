<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useSoftwareList, useUpdateCheck, useCancelOperation } from "../../composables/useInvoke";
import { useOperations } from "../../composables/useOperations";

defineEmits<{
  toggleLog: [];
}>();

const { data: software } = useSoftwareList(() => "all");
const { data: updates } = useUpdateCheck();
const { operation, isRunning, cancelOperation } = useOperations();
const cancelMutation = useCancelOperation();

function handleCancel() {
  if (operation.value) {
    cancelMutation.mutate(operation.value.id, {
      onSuccess: () => cancelOperation(),
    });
  }
}

const catalogCount = computed(() => software.value?.length ?? 0);
const installedCount = computed(() => {
  if (!software.value) return 0;
  return (software.value as Array<{ detection?: { type: string } }>).filter(
    (p) => p.detection?.type === "Installed" || p.detection?.type === "InstalledUnknownVersion",
  ).length;
});
const updateCount = computed(() => updates.value?.length ?? 0);

const lastScanTime = ref<Date | null>(null);
let unlistenScan: UnlistenFn | null = null;
let unlistenCatalog: UnlistenFn | null = null;

onMounted(async () => {
  try {
    unlistenScan = await listen("core-event", (event) => {
      const payload = event.payload as { type?: string };
      if (payload.type === "scan_complete") {
        lastScanTime.value = new Date();
      }
    });
    unlistenCatalog = await listen("catalog-status", (event) => {
      if (event.payload === "ready") {
        lastScanTime.value = new Date();
      }
    });
  } catch {
    // Not running inside Tauri
  }
});

onUnmounted(() => {
  unlistenScan?.();
  unlistenCatalog?.();
});

const lastSync = computed(() => {
  if (!lastScanTime.value) return "Never";
  return lastScanTime.value.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", hour12: false });
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
        Last scan: {{ lastSync }}
      </span>
      <template v-if="isRunning">
        <span class="status-separator" />
        <span class="status-item has-op">
          <i class="pi pi-spinner pi-spin" />
          {{ operation?.label }}
          <button
            class="cancel-btn"
            title="Cancel operation"
            @click="handleCancel"
          >
            <i class="pi pi-times" />
          </button>
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

.cancel-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  margin-left: 4px;
  padding: 0;
  background: transparent;
  border: none;
  border-radius: 3px;
  color: var(--p-surface-400);
  cursor: pointer;
  line-height: 1;
}

.cancel-btn:hover {
  background: var(--p-surface-700);
  color: var(--p-red-400);
}

.cancel-btn i {
  font-size: 10px;
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
