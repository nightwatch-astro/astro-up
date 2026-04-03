<script setup lang="ts">
import Button from "primevue/button";
import type { BackupListEntry } from "../../types/backup";

defineProps<{
  appId: string;
  backups: BackupListEntry[];
}>();

defineEmits<{
  preview: [archive: string];
  delete: [archive: string];
}>();

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}
</script>

<template>
  <div class="backup-group">
    <div class="group-header">
      <i class="pi pi-box" />
      <span class="group-name">{{ appId }}</span>
      <span class="group-count">{{ backups.length }} backup(s)</span>
    </div>
    <div
      v-for="backup in backups"
      :key="backup.archive_path"
      class="backup-item"
    >
      <div class="backup-info">
        <span class="backup-version">v{{ backup.version }}</span>
        <span class="backup-meta">
          {{ backup.file_count }} files · {{ formatSize(backup.total_size) }} ·
          {{ formatDate(backup.created_at) }}
        </span>
      </div>
      <div class="backup-actions">
        <Button
          icon="pi pi-eye"
          text
          rounded
          size="small"
          severity="secondary"
          title="Preview"
          @click="$emit('preview', backup.archive_path)"
        />
        <Button
          icon="pi pi-trash"
          text
          rounded
          size="small"
          severity="danger"
          title="Delete"
          @click="$emit('delete', backup.archive_path)"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.backup-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  color: var(--p-surface-200);
  font-size: 14px;
  font-weight: 600;
}

.group-header i {
  font-size: 16px;
  color: var(--p-surface-400);
}

.group-count {
  font-size: 12px;
  font-weight: 400;
  color: var(--p-surface-400);
}

.backup-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  margin-left: 24px;
  border: 1px solid var(--p-surface-700);
  border-radius: 6px;
}

.backup-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.backup-version {
  font-size: 13px;
  font-weight: 500;
  color: var(--p-surface-200);
}

.backup-meta {
  font-size: 12px;
  color: var(--p-surface-400);
}

.backup-actions {
  display: flex;
  gap: 2px;
}
</style>
