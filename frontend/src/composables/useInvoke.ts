import { useQuery, useMutation, useQueryClient } from "@tanstack/vue-query";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "primevue/usetoast";
import { useErrorLog } from "../stores/errorLog";
import type { OperationId } from "../types/commands";

function useMutationErrorHandler(operation: string) {
  const toast = useToast();
  const { addEntry } = useErrorLog();
  return (err: unknown) => {
    const message = err instanceof Error ? err.message : String(err);
    addEntry("error", `${operation} failed`, message);
    toast.add({
      severity: "error",
      summary: `${operation} failed`,
      detail: message,
      life: 5000,
    });
  };
}

// --- Read queries ---

export function useSoftwareList(filter: () => string) {
  return useQuery({
    queryKey: ["software", filter],
    queryFn: () => invoke<unknown[]>("list_software", { filter: filter() }),
  });
}

export function useCatalogSearch(query: () => string) {
  return useQuery({
    queryKey: ["catalog-search", query],
    queryFn: () => invoke<unknown[]>("search_catalog", { query: query() }),
    enabled: () => query().length > 0,
  });
}

export function useVersions(id: () => string) {
  return useQuery({
    queryKey: ["versions", id],
    queryFn: () => invoke<unknown[]>("get_versions", { id: id() }),
    enabled: () => id().length > 0,
  });
}

export function useUpdateCheck() {
  return useQuery({
    queryKey: ["updates"],
    queryFn: () => invoke<unknown[]>("check_for_updates"),
  });
}

export function useConfig() {
  return useQuery({
    queryKey: ["config"],
    queryFn: () => invoke<Record<string, unknown>>("get_config"),
  });
}

// --- Backup queries (mock data until backend commands exist) ---

export function useBackupList() {
  return useQuery({
    queryKey: ["backups"],
    queryFn: async () => {
      const { mockBackups } = await import("../mocks");
      return mockBackups;
    },
    staleTime: 5 * 60 * 1000,
  });
}

export function useBackupContents(archive: () => string | null) {
  return useQuery({
    queryKey: ["backup-contents", archive],
    queryFn: async () => {
      const { mockBackupContents } = await import("../mocks");
      return mockBackupContents;
    },
    enabled: () => archive() !== null,
  });
}

export function useBackupPreview(archive: () => string | null) {
  return useQuery({
    queryKey: ["backup-preview", archive],
    queryFn: async () => {
      const { mockRestorePreview } = await import("../mocks");
      return mockRestorePreview;
    },
    enabled: () => archive() !== null,
  });
}

export function useSyncCatalog() {
  const queryClient = useQueryClient();
  const onError = useMutationErrorHandler("Catalog sync");
  return useMutation({
    mutationFn: () => invoke<string>("sync_catalog"),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["software"] });
      queryClient.invalidateQueries({ queryKey: ["updates"] });
    },
    onError,
  });
}

// --- Mutations ---

export function useSaveConfig() {
  const queryClient = useQueryClient();
  const onError = useMutationErrorHandler("Save config");
  return useMutation({
    mutationFn: (config: Record<string, unknown>) =>
      invoke("save_config", { config }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["config"] });
    },
    onError,
  });
}

export function useInstallSoftware() {
  const queryClient = useQueryClient();
  const onError = useMutationErrorHandler("Install");
  return useMutation({
    mutationFn: (id: string) =>
      invoke<OperationId>("install_software", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["software"] });
    },
    onError,
  });
}

export function useUpdateSoftware() {
  const queryClient = useQueryClient();
  const onError = useMutationErrorHandler("Update");
  return useMutation({
    mutationFn: (id: string) =>
      invoke<OperationId>("update_software", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["software"] });
      queryClient.invalidateQueries({ queryKey: ["updates"] });
    },
    onError,
  });
}

export function useUpdateAll() {
  const queryClient = useQueryClient();
  const onError = useMutationErrorHandler("Update all");
  return useMutation({
    mutationFn: () =>
      invoke<OperationId>("update_all"),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["software"] });
      queryClient.invalidateQueries({ queryKey: ["updates"] });
    },
    onError,
  });
}

export function useScanInstalled() {
  const queryClient = useQueryClient();
  const onError = useMutationErrorHandler("Scan installed");
  return useMutation({
    mutationFn: () => invoke<unknown>("scan_installed"),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["software"] });
    },
    onError,
  });
}

export function useCreateBackup() {
  const onError = useMutationErrorHandler("Create backup");
  return useMutation({
    mutationFn: (paths: string[]) =>
      invoke<OperationId>("create_backup", { paths }),
    onError,
  });
}

export function useRestoreBackup() {
  const onError = useMutationErrorHandler("Restore backup");
  return useMutation({
    mutationFn: (params: { archive: string; filter?: string[] }) =>
      invoke<OperationId>("restore_backup", params),
    onError,
  });
}

export function useCancelOperation() {
  const onError = useMutationErrorHandler("Cancel operation");
  return useMutation({
    mutationFn: (operationId: string) =>
      invoke("cancel_operation", { operationId }),
    onError,
  });
}
