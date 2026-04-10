<script setup lang="ts">
import { ref, watch } from "vue";
import ProgressBar from "primevue/progressbar";
import Button from "primevue/button";
import { useOperations } from "../../composables/useOperations";
import { useUpdateQueue } from "../../composables/useUpdateQueue";
import { useCancelOperation } from "../../composables/useInvoke";

const DOWNLOAD_PATH_PREFIX = "Downloaded to: ";

function extractDownloadPath(message: string): string | null {
  if (message.startsWith(DOWNLOAD_PATH_PREFIX)) {
    return message.slice(DOWNLOAD_PATH_PREFIX.length);
  }
  return null;
}

async function openDownloadFolder(path: string) {
  try {
    const { open: openPath } = await import("@tauri-apps/plugin-shell");
    await openPath(path);
  } catch {
    // Not running inside Tauri or path invalid
  }
}

const { operation, dismissOperation, cancelOperation } = useOperations();
const { isActive: queueActive, progress: queueProgress, queueLabel, summary: queueSummary, items: queueItems, cancelQueue, clearQueue } = useUpdateQueue();
const cancelMutation = useCancelOperation();

const expanded = ref(false);
let autoDismissTimer: ReturnType<typeof setTimeout> | null = null;

function handleCancel() {
  if (queueActive.value) {
    cancelQueue();
  } else if (operation.value) {
    cancelMutation.mutate(operation.value.id, {
      onSuccess: () => cancelOperation(),
    });
  }
}

function handleDismiss() {
  if (queueItems.value.length > 0) clearQueue();
  dismissOperation();
  expanded.value = false;
}

// Auto-dismiss 3s after complete (suppress during queue — queue manages lifecycle)
watch(
  () => operation.value?.status,
  (status) => {
    if (autoDismissTimer) {
      clearTimeout(autoDismissTimer);
      autoDismissTimer = null;
    }

    if (status === "complete" && !queueActive.value) {
      autoDismissTimer = setTimeout(() => {
        dismissOperation();
        expanded.value = false;
      }, 3000);
    }
  },
);
</script>

<template>
  <div
    v-if="operation || (!queueActive && queueItems.length > 0)"
    class="ops-dock"
    :class="{ expanded }"
  >
    <!-- Queue progress header -->
    <div
      v-if="queueActive"
      class="ops-queue-bar"
    >
      <i class="pi pi-list" />
      <span>{{ queueLabel }}</span>
      <span class="ops-queue-counter">{{ queueProgress.current }}/{{ queueProgress.total }}</span>
    </div>

    <!-- Queue summary (shown after queue completes) -->
    <div
      v-else-if="queueItems.length > 0"
      class="ops-queue-bar ops-queue-summary"
    >
      <i class="pi pi-check-circle" />
      <span>{{ queueSummary.succeeded }} succeeded, {{ queueSummary.failed }} failed</span>
      <Button
        icon="pi pi-times"
        text
        rounded
        size="small"
        severity="secondary"
        title="Dismiss"
        @click.stop="handleDismiss"
      />
    </div>

    <div
      v-if="operation"
      class="ops-header"
      @click="expanded = !expanded"
    >
      <div class="ops-summary">
        <i
          class="pi"
          :class="{
            'pi-spinner pi-spin': operation.status === 'running',
            'pi-check-circle': operation.status === 'complete',
            'pi-times-circle': operation.status === 'failed',
            'pi-ban': operation.status === 'cancelled',
          }"
        />
        <span class="ops-label">{{ operation.label }}</span>
        <span
          v-if="operation.status === 'running'"
          class="ops-progress-text"
        >
          {{ operation.progress }}%
        </span>
        <span
          v-else
          class="ops-status"
          :class="operation.status"
        >
          {{ operation.status }}
        </span>
      </div>

      <div class="ops-actions">
        <Button
          v-if="operation.status === 'running'"
          icon="pi pi-times"
          text
          rounded
          size="small"
          severity="secondary"
          title="Cancel"
          @click.stop="handleCancel"
        />
        <Button
          icon="pi pi-chevron-down"
          text
          rounded
          size="small"
          severity="secondary"
          :class="{ 'rotate-180': expanded }"
          @click.stop="expanded = !expanded"
        />
        <Button
          v-if="operation.status !== 'running' && !queueActive"
          icon="pi pi-times"
          text
          rounded
          size="small"
          severity="secondary"
          title="Dismiss"
          @click.stop="handleDismiss"
        />
      </div>
    </div>

    <template v-if="operation">
      <ProgressBar
        v-if="operation.status === 'running'"
        :value="operation.progress"
        :show-value="false"
        class="ops-progress"
      />

      <div
        v-if="expanded && operation.steps.length > 0"
        class="ops-steps"
      >
        <div
          v-for="(step, i) in operation.steps"
          :key="i"
          class="ops-step"
          :class="step.level"
        >
          <span class="step-time">{{ step.timestamp.slice(11, 19) }}</span>
          <span
            v-if="extractDownloadPath(step.message)"
            class="step-message step-link"
            title="Open folder"
            @click.stop="openDownloadFolder(extractDownloadPath(step.message)!)"
          >
            <i class="pi pi-folder-open" /> {{ step.message }}
          </span>
          <span
            v-else
            class="step-message"
          >{{ step.message }}</span>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.ops-dock {
  background: var(--p-surface-900);
  border-top: 1px solid var(--p-surface-700);
  flex-shrink: 0;
}

.ops-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  cursor: pointer;
  user-select: none;
}

.ops-summary {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--p-surface-200);
}

.ops-label {
  font-weight: 500;
}

.ops-progress-text {
  color: var(--p-surface-400);
  font-size: 12px;
}

.ops-status {
  font-size: 12px;
  font-weight: 500;
  text-transform: capitalize;
}

.ops-status.complete { color: var(--p-green-400); }
.ops-status.failed { color: var(--p-red-400); }
.ops-status.cancelled { color: var(--p-surface-400); }

.ops-actions {
  display: flex;
  align-items: center;
  gap: 2px;
}

.rotate-180 {
  transform: rotate(180deg);
}

.ops-progress {
  height: 3px;
  border-radius: 0;
}

:deep(.ops-progress .p-progressbar-value) {
  border-radius: 0;
}

.ops-steps {
  max-height: 150px;
  overflow-y: auto;
  padding: 4px 12px 8px;
  font-family: "JetBrains Mono", "Fira Code", monospace;
  font-size: 11px;
}

.ops-step {
  display: flex;
  gap: 8px;
  line-height: 1.6;
}

.ops-step.error { color: var(--p-red-400); }
.ops-step.warn { color: var(--p-yellow-400); }
.ops-step.info { color: var(--p-surface-400); }

.step-time {
  color: var(--p-surface-500);
  flex-shrink: 0;
}

.ops-queue-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  font-size: 12px;
  color: var(--p-primary-400);
  background: color-mix(in srgb, var(--p-primary-500) 8%, transparent);
  border-bottom: 1px solid var(--p-surface-700);
}

.ops-queue-counter {
  margin-left: auto;
  font-weight: 600;
}

.ops-queue-summary {
  color: var(--p-surface-300);
  background: transparent;
}

.step-link {
  cursor: pointer;
  color: var(--p-primary-400);
  text-decoration: underline;
  text-decoration-style: dotted;
}

.step-link:hover {
  color: var(--p-primary-300);
}

.step-link .pi {
  font-size: 10px;
  margin-right: 2px;
}
</style>
