import { ref, computed, readonly } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useQueryClient } from "@tanstack/vue-query";
import { logger } from "../utils/logger";
import type { OperationId } from "../types/commands";

export type QueueItemStatus = "pending" | "running" | "complete" | "failed" | "cancelled";

export interface QueueItem {
  packageId: string;
  packageName: string;
  status: QueueItemStatus;
  error?: string;
}

// Singleton state (same pattern as useOperations.ts)
const queue = ref<QueueItem[]>([]);
const currentIndex = ref(-1);
const _isActive = ref(false);
const isCancelling = ref(false);
const currentOperationId = ref<string | null>(null);

export function useUpdateQueue() {
  const isActive = computed(() => _isActive.value);

  const currentItem = computed(() =>
    currentIndex.value >= 0 ? queue.value[currentIndex.value] ?? null : null,
  );

  const progress = computed(() => ({
    current: currentIndex.value + 1,
    total: queue.value.length,
  }));

  const queueLabel = computed(() => {
    const item = currentItem.value;
    if (!item) return "";
    return `Updating ${progress.value.current}/${progress.value.total}: ${item.packageName}`;
  });

  const summary = computed(() => {
    const counts = { succeeded: 0, failed: 0, cancelled: 0 };
    for (const item of queue.value) {
      if (item.status === "complete") counts.succeeded++;
      else if (item.status === "failed") counts.failed++;
      else if (item.status === "cancelled") counts.cancelled++;
    }
    return counts;
  });

  async function enqueue(packages: Array<{ id: string; name: string }>) {
    if (_isActive.value || packages.length === 0) return;

    queue.value = packages.map((p) => ({
      packageId: p.id,
      packageName: p.name,
      status: "pending" as const,
    }));
    currentIndex.value = 0;
    _isActive.value = true;
    isCancelling.value = false;

    logger.debug("useUpdateQueue", `enqueued ${packages.length} packages`);

    for (let i = 0; i < queue.value.length; i++) {
      if (isCancelling.value) {
        for (let j = i; j < queue.value.length; j++) {
          queue.value[j].status = "cancelled";
        }
        break;
      }

      currentIndex.value = i;
      queue.value[i].status = "running";

      try {
        const result = await invoke<OperationId>("update_software", {
          id: queue.value[i].packageId,
        });
        currentOperationId.value = result.id;
        // By the time invoke resolves, package_complete event has already
        // updated the status via markCurrentComplete/Failed. Safety net:
        if (queue.value[i].status === "running") {
          queue.value[i].status = "complete";
        }
      } catch (err) {
        queue.value[i].status = "failed";
        queue.value[i].error =
          err instanceof Error ? err.message : String(err);
      }

      currentOperationId.value = null;
    }

    _isActive.value = false;
    currentIndex.value = -1;

    logger.debug(
      "useUpdateQueue",
      `queue complete: ${summary.value.succeeded} ok, ${summary.value.failed} failed`,
    );

    // Invalidate queries once at the end
    const queryClient = useQueryClient();
    queryClient.invalidateQueries({ queryKey: ["software"] });
    queryClient.invalidateQueries({ queryKey: ["updates"] });
    queryClient.invalidateQueries({ queryKey: ["activity"] });
  }

  async function cancelQueue() {
    isCancelling.value = true;
    if (currentOperationId.value) {
      try {
        await invoke("cancel_operation", {
          operationId: currentOperationId.value,
        });
      } catch {
        // ignore — operation may have already finished
      }
    }
  }

  function clearQueue() {
    queue.value = [];
    currentIndex.value = -1;
    _isActive.value = false;
    isCancelling.value = false;
    currentOperationId.value = null;
  }

  function markCurrentComplete() {
    if (!_isActive.value || currentIndex.value < 0) return;
    const item = queue.value[currentIndex.value];
    if (item?.status === "running") item.status = "complete";
  }

  function markCurrentFailed(error: string) {
    if (!_isActive.value || currentIndex.value < 0) return;
    const item = queue.value[currentIndex.value];
    if (item) {
      item.status = "failed";
      item.error = error;
    }
  }

  function markCurrentBlocked(processName: string, pid: number) {
    if (!_isActive.value || currentIndex.value < 0) return;
    const item = queue.value[currentIndex.value];
    if (item) {
      item.status = "failed";
      item.error = `${processName} is running (PID ${pid})`;
    }
  }

  return {
    isActive,
    currentItem,
    progress,
    queueLabel,
    items: readonly(queue),
    summary,
    enqueue,
    cancelQueue,
    clearQueue,
    markCurrentComplete,
    markCurrentFailed,
    markCurrentBlocked,
  };
}
