<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import Toast from "primevue/toast";
import { useToast } from "primevue/usetoast";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useTheme } from "./composables/useTheme";
import { useCoreEvents } from "./composables/useCoreEvents";
import { useKeyboard } from "./composables/useKeyboard";
import { useOperations } from "./composables/useOperations";
import { useErrorLog } from "./stores/errorLog";
import AppSidebar from "./components/layout/AppSidebar.vue";
import AppStatusBar from "./components/layout/AppStatusBar.vue";
import OperationsDock from "./components/layout/OperationsDock.vue";
import LogPanel from "./components/layout/LogPanel.vue";
import type { CoreEvent } from "./types/commands";

const toast = useToast();
const { init: initTheme } = useTheme();
const { addEntry } = useErrorLog();
const { updateProgress, completeOperation, failOperation, addStep } = useOperations();
const logVisible = ref(false);
const logPanel = ref<InstanceType<typeof LogPanel> | null>(null);

useKeyboard({
  onToggleLog: () => { logVisible.value = !logVisible.value; },
  onEscape: () => { logVisible.value = false; },
  onFocusSearch: () => {
    const searchInput = document.querySelector<HTMLInputElement>(".app-main input[type='text'], .app-main .p-inputtext");
    searchInput?.focus();
  },
});

const updateVersion = ref<string | null>(null);
let unlistenUpdate: UnlistenFn | null = null;
let unlistenBackendLog: UnlistenFn | null = null;

// Forward backend tracing logs to the log panel
async function setupBackendLogListener() {
  try {
    unlistenBackendLog = await listen<{
      timestamp: string;
      level: string;
      target: string;
      message: string;
    }>("backend-log", (event) => {
      logPanel.value?.addEntry({
        timestamp: event.payload.timestamp,
        level: event.payload.level as "error" | "warn" | "info" | "debug" | "trace",
      target: event.payload.target,
      message: event.payload.message,
    });
  });
  } catch {
    // Not running inside Tauri
  }
}

// Wire core events to operations dock, log panel, and error toasts (T036/T037)
useCoreEvents((event: CoreEvent) => {
  // Forward all events to log panel
  logPanel.value?.addEntry({
    timestamp: new Date().toISOString(),
    level: event.type.includes("error") || event.type.includes("failed") ? "error" : "info",
    target: "core",
    message: `[${event.type}] ${JSON.stringify(event.data)}`,
  });

  // Map events to operations progress
  if (event.type === "download_progress") {
    updateProgress(event.data.progress, `Downloading: ${event.data.progress}%`);
  } else if (event.type === "scan_progress") {
    updateProgress(event.data.progress, `Scanning: ${event.data.current_id}`);
  } else if (event.type === "backup_progress") {
    const pct = Math.round((event.data.files_processed / event.data.total_files) * 100);
    updateProgress(pct, `Backing up: ${event.data.files_processed}/${event.data.total_files}`);
  } else if (
    event.type === "install_complete" ||
    event.type === "download_complete" ||
    event.type === "scan_complete" ||
    event.type === "backup_complete" ||
    event.type === "restore_complete" ||
    event.type === "orchestration_complete"
  ) {
    completeOperation();
    addStep("info", `${event.type}`);
  } else if (event.type === "install_started" || event.type === "download_started" || event.type === "backup_started" || event.type === "restore_started" || event.type === "scan_started") {
    addStep("info", `${event.type}`);
  }

  // Error handling -> toasts + error log (FR-028)
  if (event.type === "error" || event.type === "install_failed") {
    const detail = "error" in event.data ? event.data.error : "Unknown error";
    failOperation(detail);
    addEntry("error", `Operation failed: ${event.data.id}`, detail);
    toast.add({
      severity: "error",
      summary: `Error: ${event.data.id}`,
      detail,
      life: 5000,
    });
  }
});

// Listen for self-update availability
async function setupUpdateListener() {
  try {
    unlistenUpdate = await listen<{ version: string; body: string | null }>(
    "update-available",
    (event) => {
      updateVersion.value = event.payload.version;
      toast.add({
        severity: "info",
        summary: `Update available: v${event.payload.version}`,
        detail: event.payload.body ?? "A new version is available.",
        life: 0,
        closable: true,
        group: "update",
      });
    },
  );
  } catch {
    // Not running inside Tauri
  }
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
  setupBackendLogListener();
});

onUnmounted(() => {
  if (unlistenUpdate) unlistenUpdate();
  if (unlistenBackendLog) unlistenBackendLog();
});
</script>

<template>
  <Toast position="bottom-right" />

  <!-- Update notification banner -->
  <div
    v-if="updateVersion"
    class="update-banner"
  >
    <span>Update available: v{{ updateVersion }}</span>
    <div class="update-actions">
      <button
        class="update-btn update-btn-primary"
        @click="installUpdate"
      >
        Install
      </button>
      <button
        class="update-btn update-btn-secondary"
        @click="dismissUpdate"
      >
        Dismiss
      </button>
    </div>
  </div>

  <!-- App layout: sidebar + main + status bar -->
  <div class="app-layout">
    <AppSidebar />

    <div class="app-content">
      <div class="app-main">
        <router-view />
      </div>
      <OperationsDock />
      <LogPanel
        ref="logPanel"
        :visible="logVisible"
        @close="logVisible = false"
      />
      <AppStatusBar @toggle-log="logVisible = !logVisible" />
    </div>
  </div>
</template>

<style>
.app-layout {
  display: flex;
  width: 100%;
  height: 100%;
}

.app-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

.app-main {
  flex: 1;
  overflow-y: auto;
}

.update-banner {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 50;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  background: var(--p-primary-500);
  padding: 8px 16px;
  color: var(--p-surface-0);
  font-size: 14px;
}

.update-actions {
  display: flex;
  gap: 8px;
}

.update-btn {
  padding: 4px 12px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: none;
}

.update-btn-primary {
  background: var(--p-surface-0);
  color: var(--p-primary-500);
}

.update-btn-secondary {
  background: transparent;
  border: 1px solid var(--p-surface-0);
  color: var(--p-surface-0);
}
</style>
