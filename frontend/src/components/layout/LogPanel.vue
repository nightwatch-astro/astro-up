<script setup lang="ts">
import { ref, computed, nextTick, watch } from "vue";
import Dropdown from "primevue/dropdown";
import type { LogEntry } from "../../types/operations";

const props = defineProps<{
  visible: boolean;
}>();

defineEmits<{
  close: [];
}>();

const MAX_ENTRIES = 1000;
const entries = ref<LogEntry[]>([]);
const logFilter = ref<LogEntry["level"] | "all">("info");
const logContainer = ref<HTMLElement | null>(null);
const panelHeight = ref(200);
const isResizing = ref(false);

const filterOptions = [
  { label: "All", value: "all" },
  { label: "Error", value: "error" },
  { label: "Warn", value: "warn" },
  { label: "Info", value: "info" },
  { label: "Debug", value: "debug" },
  { label: "Trace", value: "trace" },
];

const filteredEntries = computed(() => {
  if (logFilter.value === "all") return entries.value;
  return entries.value.filter((e) => e.level === logFilter.value);
});

function addEntry(entry: LogEntry) {
  entries.value.push(entry);
  if (entries.value.length > MAX_ENTRIES) {
    entries.value.splice(0, entries.value.length - MAX_ENTRIES);
  }
  nextTick(() => {
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight;
    }
  });
}

function clearLog() {
  entries.value = [];
}

function startResize(event: MouseEvent) {
  isResizing.value = true;
  const startY = event.clientY;
  const startHeight = panelHeight.value;
  const maxHeight = window.innerHeight * 0.6;
  const minHeight = 100;

  function onMouseMove(e: MouseEvent) {
    const delta = startY - e.clientY;
    panelHeight.value = Math.min(maxHeight, Math.max(minHeight, startHeight + delta));
  }

  function onMouseUp() {
    isResizing.value = false;
    document.removeEventListener("mousemove", onMouseMove);
    document.removeEventListener("mouseup", onMouseUp);
  }

  document.addEventListener("mousemove", onMouseMove);
  document.addEventListener("mouseup", onMouseUp);
}

function levelColor(level: LogEntry["level"]): string {
  switch (level) {
    case "error": return "var(--p-red-400)";
    case "warn": return "var(--p-yellow-400)";
    case "info": return "var(--p-blue-400)";
    case "debug": return "var(--p-surface-400)";
    case "trace": return "var(--p-surface-500)";
  }
}

// Auto-scroll on new entries when panel is visible
watch(() => props.visible, (v) => {
  if (v) {
    nextTick(() => {
      if (logContainer.value) {
        logContainer.value.scrollTop = logContainer.value.scrollHeight;
      }
    });
  }
});

defineExpose({ addEntry, clearLog });
</script>

<template>
  <div
    v-if="visible"
    class="log-panel"
    :style="{ height: panelHeight + 'px' }"
  >
    <div
      class="log-resize-handle"
      @mousedown="startResize"
    />
    <div class="log-header">
      <span class="log-title">Log</span>
      <Dropdown
        v-model="logFilter"
        :options="filterOptions"
        option-label="label"
        option-value="value"
        class="log-filter"
      />
      <button
        class="log-action"
        title="Clear log"
        @click="clearLog"
      >
        <i class="pi pi-trash" />
      </button>
      <button
        class="log-action"
        title="Close log panel"
        @click="$emit('close')"
      >
        <i class="pi pi-times" />
      </button>
    </div>
    <div
      ref="logContainer"
      class="log-entries"
    >
      <div
        v-for="(entry, i) in filteredEntries"
        :key="i"
        class="log-entry"
      >
        <span class="log-time">{{ entry.timestamp.slice(11, 23) }}</span>
        <span
          class="log-level"
          :style="{ color: levelColor(entry.level) }"
        >
          {{ entry.level.toUpperCase().padEnd(5) }}
        </span>
        <span class="log-target">{{ entry.target }}</span>
        <span class="log-message">{{ entry.message }}</span>
      </div>
      <div
        v-if="filteredEntries.length === 0"
        class="log-empty"
      >
        No log entries{{ logFilter !== 'all' ? ` at ${logFilter} level` : '' }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.log-panel {
  background: var(--p-surface-950);
  border-top: 1px solid var(--p-surface-700);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  position: relative;
}

.log-resize-handle {
  position: absolute;
  top: -3px;
  left: 0;
  right: 0;
  height: 6px;
  cursor: ns-resize;
  z-index: 10;
}

.log-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 12px;
  border-bottom: 1px solid var(--p-surface-800);
  flex-shrink: 0;
}

.log-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--p-surface-300);
  margin-right: auto;
}

.log-filter {
  font-size: 11px;
}

:deep(.log-filter .p-dropdown-label) {
  padding: 2px 8px;
  font-size: 11px;
}

.log-action {
  background: none;
  border: none;
  color: var(--p-surface-400);
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
  font-size: 11px;
}

.log-action:hover {
  background: var(--p-surface-800);
  color: var(--p-surface-0);
}

.log-entries {
  flex: 1;
  overflow-y: auto;
  padding: 4px 12px;
  font-family: "JetBrains Mono", "Fira Code", "Cascadia Code", monospace;
  font-size: 11px;
  line-height: 1.6;
}

.log-entry {
  display: flex;
  gap: 8px;
  white-space: nowrap;
}

.log-time {
  color: var(--p-surface-500);
  flex-shrink: 0;
}

.log-level {
  flex-shrink: 0;
  font-weight: 600;
}

.log-target {
  color: var(--p-surface-500);
  flex-shrink: 0;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.log-message {
  color: var(--p-surface-300);
  overflow: hidden;
  text-overflow: ellipsis;
}

.log-empty {
  color: var(--p-surface-500);
  padding: 8px 0;
  font-style: italic;
}
</style>
