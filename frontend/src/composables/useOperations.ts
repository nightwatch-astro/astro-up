import { ref, computed, readonly } from "vue";
import { useToast } from "primevue/usetoast";
import { logger } from "../utils/logger";
import type { Operation, OperationStep } from "../types/operations";

const activeOperation = ref<Operation | null>(null);

export function useOperations() {
  const toast = useToast();
  const isRunning = computed(() => activeOperation.value?.status === "running");

  function startOperation(id: string, label: string): boolean {
    // Clear finished operations so they don't block new ones
    if (activeOperation.value && activeOperation.value.status !== "running") {
      activeOperation.value = null;
    }

    // Safety valve: force-clear stale running operations (>60s)
    if (activeOperation.value && activeOperation.value.status === "running") {
      const started = activeOperation.value.steps[0]?.timestamp;
      if (started && Date.now() - new Date(started).getTime() > 60_000) {
        activeOperation.value = null;
      }
    }

    if (isRunning.value) {
      toast.add({
        severity: "warn",
        summary: "An operation is already in progress.",
        life: 3000,
      });
      return false;
    }

    activeOperation.value = {
      id,
      label,
      progress: 0,
      status: "running",
      steps: [],
    };
    logger.debug("useOperations", `started: ${id} (${label})`);
    return true;
  }

  function updateProgress(progress: number, message?: string) {
    if (!activeOperation.value) return;
    activeOperation.value.progress = Math.min(100, Math.max(0, progress));
    if (message) {
      addStep("info", message);
    }
  }

  function addStep(level: OperationStep["level"], message: string) {
    if (!activeOperation.value) return;
    activeOperation.value.steps.push({
      timestamp: new Date().toISOString(),
      message,
      level,
    });
  }

  function completeOperation() {
    if (!activeOperation.value) return;
    logger.debug("useOperations", `completed: ${activeOperation.value.id}`);
    activeOperation.value.status = "complete";
    activeOperation.value.progress = 100;
  }

  function failOperation(error: string) {
    if (!activeOperation.value) return;
    logger.debug("useOperations", `failed: ${activeOperation.value.id} — ${error}`);
    activeOperation.value.status = "failed";
    addStep("error", error);
  }

  function cancelOperation() {
    if (!activeOperation.value) return;
    logger.debug("useOperations", `cancelled: ${activeOperation.value.id}`);
    activeOperation.value.status = "cancelled";
  }

  function dismissOperation() {
    activeOperation.value = null;
  }

  return {
    operation: readonly(activeOperation),
    isRunning,
    startOperation,
    updateProgress,
    addStep,
    completeOperation,
    failOperation,
    cancelOperation,
    dismissOperation,
  };
}
