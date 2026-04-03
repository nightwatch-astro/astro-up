<script setup lang="ts">
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Tag from "primevue/tag";
import type { BackupFile, FileChange } from "../../types/backup";

defineProps<{
  mode: "action" | "preview";
  files?: BackupFile[];
  changes?: FileChange[];
}>();

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatDate(iso: string | null): string {
  if (!iso) return "—";
  return new Date(iso).toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

function actionSeverity(action: FileChange["action"]): "danger" | "warn" | "success" | "secondary" {
  switch (action) {
    case "overwrite": return "danger";
    case "new": return "success";
    case "missing": return "warn";
    case "unchanged": return "secondary";
  }
}
</script>

<template>
  <!-- Action mode: restore preview with change indicators -->
  <DataTable
    v-if="mode === 'action' && changes"
    :value="changes"
    striped-rows
    size="small"
    scrollable
    scroll-height="300px"
  >
    <Column
      field="name"
      header="File"
      style="min-width: 250px"
    >
      <template #body="{ data }">
        <span class="file-name">{{ (data as FileChange).name }}</span>
      </template>
    </Column>
    <Column
      header="Action"
      style="width: 110px"
    >
      <template #body="{ data }">
        <Tag
          :value="(data as FileChange).action"
          :severity="actionSeverity((data as FileChange).action)"
        />
      </template>
    </Column>
    <Column
      header="Backup Size"
      style="width: 100px"
    >
      <template #body="{ data }">
        {{ formatSize((data as FileChange).backup_size) }}
      </template>
    </Column>
    <Column
      header="Backup Date"
      style="width: 120px"
    >
      <template #body="{ data }">
        {{ formatDate((data as FileChange).backup_modified) }}
      </template>
    </Column>
  </DataTable>

  <!-- Preview mode: simple file listing -->
  <DataTable
    v-else-if="mode === 'preview' && files"
    :value="files"
    striped-rows
    size="small"
    scrollable
    scroll-height="300px"
  >
    <Column
      field="name"
      header="File"
      style="min-width: 250px"
    >
      <template #body="{ data }">
        <span class="file-name">{{ (data as BackupFile).name }}</span>
      </template>
    </Column>
    <Column
      header="Size"
      style="width: 100px"
    >
      <template #body="{ data }">
        {{ formatSize((data as BackupFile).size) }}
      </template>
    </Column>
    <Column
      header="Modified"
      style="width: 120px"
    >
      <template #body="{ data }">
        {{ formatDate((data as BackupFile).modified) }}
      </template>
    </Column>
  </DataTable>
</template>

<style scoped>
.file-name {
  font-family: "JetBrains Mono", "Fira Code", monospace;
  font-size: 12px;
}
</style>
