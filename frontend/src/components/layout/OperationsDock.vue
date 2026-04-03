<script setup lang="ts">
import { ref, watch } from "vue";
import ProgressBar from "primevue/progressbar";
import Button from "primevue/button";
import { useOperations } from "../../composables/useOperations";
import { useCancelOperation } from "../../composables/useInvoke";

const { operation, dismissOperation, cancelOperation } = useOperations();
const cancelMutation = useCancelOperation();

const expanded = ref(false);
let autoDismissTimer: ReturnType<typeof setTimeout> | null = null;

function handleCancel() {
  if (operation.value) {
    cancelMutation.mutate(operation.value.id, {
      onSuccess: () => cancelOperation(),
    });
  }
}

// Auto-dismiss 3s after complete
watch(
  () => operation.value?.status,
  (status) => {
    if (autoDismissTimer) {
      clearTimeout(autoDismissTimer);
      autoDismissTimer = null;
    }

    if (status === "complete") {
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
    v-if="operation"
    class="ops-dock"
    :class="{ expanded }"
  >
    <div
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
          v-if="operation.status !== 'running'"
          icon="pi pi-times"
          text
          rounded
          size="small"
          severity="secondary"
          title="Dismiss"
          @click.stop="dismissOperation()"
        />
      </div>
    </div>

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
        <span class="step-message">{{ step.message }}</span>
      </div>
    </div>
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
</style>
