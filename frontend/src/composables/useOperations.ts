import { ref, computed, readonly } from "vue";
import { useToast } from "primevue/usetoast";
import type { Operation, OperationStep } from "../types/operations";

const activeOperation = ref<Operation | null>(null);

export function useOperations() {
  const toast = useToast();
  const isRunning = computed(() => activeOperation.value?.status === "running");

  function startOperation(id: string, label: string): boolean {
    // Single-op guard (FR-052)
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
    activeOperation.value.status = "complete";
    activeOperation.value.progress = 100;
  }

  function failOperation(error: string) {
    if (!activeOperation.value) return;
    activeOperation.value.status = "failed";
    addStep("error", error);
  }

  function cancelOperation() {
    if (!activeOperation.value) return;
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
