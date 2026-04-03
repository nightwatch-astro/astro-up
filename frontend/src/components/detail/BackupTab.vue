<script setup lang="ts">
import { computed } from "vue";
import Button from "primevue/button";
import Tag from "primevue/tag";
import { mockBackups } from "../../mocks";
import type { BackupListEntry } from "../../types/backup";

const props = defineProps<{
  packageId: string;
}>();

defineEmits<{
  backup: [];
  preview: [archive: string];
  delete: [archive: string];
}>();

const backups = computed<BackupListEntry[]>(() =>
  mockBackups.filter((b) => b.package_id === props.packageId),
);

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
  <div class="backup-tab">
    <div class="backup-actions">
      <Button
        label="Backup Now"
        icon="pi pi-database"
        size="small"
        @click="$emit('backup')"
      />
    </div>

    <div
      v-if="backups.length === 0"
      class="backup-empty"
    >
      No backups for this package.
    </div>

    <div
      v-for="backup in backups"
      :key="backup.archive_path"
      class="backup-item"
    >
      <div class="backup-info">
        <div class="backup-header">
          <Tag
            :value="'v' + backup.version"
            severity="secondary"
          />
          <span class="backup-date">{{ formatDate(backup.created_at) }}</span>
        </div>
        <span class="backup-meta">
          {{ backup.file_count }} files · {{ formatSize(backup.total_size) }}
        </span>
      </div>
      <div class="backup-item-actions">
        <Button
          icon="pi pi-eye"
          text
          rounded
          size="small"
          severity="secondary"
          title="Preview contents"
          @click="$emit('preview', backup.archive_path)"
        />
        <Button
          icon="pi pi-trash"
          text
          rounded
          size="small"
          severity="danger"
          title="Delete backup"
          @click="$emit('delete', backup.archive_path)"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.backup-tab {
  padding: 20px 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.backup-actions {
  margin-bottom: 8px;
}

.backup-empty {
  color: var(--p-surface-400);
  font-size: 14px;
  font-style: italic;
  padding: 16px 0;
}

.backup-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border: 1px solid var(--p-surface-700);
  border-radius: 8px;
}

.backup-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.backup-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.backup-date {
  font-size: 13px;
  color: var(--p-surface-300);
}

.backup-meta {
  font-size: 12px;
  color: var(--p-surface-400);
}

.backup-item-actions {
  display: flex;
  gap: 2px;
}
</style>
