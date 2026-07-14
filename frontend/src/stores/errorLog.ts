// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

import { ref } from "vue";
import type { ErrorLogEntry } from "../types/commands";

const MAX_ENTRIES = 100;

const entries = ref<ErrorLogEntry[]>([]);

export function useErrorLog() {
  function addEntry(
    severity: "error" | "warning",
    summary: string,
    detail: string,
  ) {
    entries.value.unshift({
      timestamp: new Date(),
      severity,
      summary,
      detail,
    });
    if (entries.value.length > MAX_ENTRIES) {
      entries.value.length = MAX_ENTRIES;
    }
  }

  function clearEntries() {
    entries.value = [];
  }

  return {
    entries,
    addEntry,
    clearEntries,
  };
}
