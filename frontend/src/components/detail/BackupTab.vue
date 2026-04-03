<script setup lang="ts">
import { computed, ref } from "vue";
import Button from "primevue/button";
import Tag from "primevue/tag";
import ToggleSwitch from "primevue/toggleswitch";
import InputText from "primevue/inputtext";
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

// Mock manifest paths (would come from backend package definition)
const manifestPaths = computed(() => [
  "%APPDATA%/N.I.N.A/Profiles",
  "%APPDATA%/N.I.N.A/Settings",
]);

const customPaths = ref<string[]>([]);
const newPath = ref("");
const backupBeforeUpdate = ref(true);
const includeInScheduled = ref(false);

function addCustomPath() {
  const path = newPath.value.trim();
  if (path && !customPaths.value.includes(path)) {
    customPaths.value.push(path);
    newPath.value = "";
  }
}

function removeCustomPath(index: number) {
  customPaths.value.splice(index, 1);
}

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
    <!-- Backup config -->
    <section class="backup-section">
      <h4 class="section-title">
        Backup Paths
      </h4>

      <!-- Manifest paths (read-only) -->
      <div
        v-for="path in manifestPaths"
        :key="path"
        class="path-item"
      >
        <Tag
          value="MANIFEST"
          severity="secondary"
          class="path-badge"
        />
        <span class="path-text">{{ path }}</span>
      </div>

      <!-- Custom paths (editable) -->
      <div
        v-for="(path, i) in customPaths"
        :key="'custom-' + i"
        class="path-item"
      >
        <Tag
          value="CUSTOM"
          severity="info"
          class="path-badge"
        />
        <span class="path-text">{{ path }}</span>
        <Button
          icon="pi pi-times"
          text
          rounded
          size="small"
          severity="danger"
          @click="removeCustomPath(i)"
        />
      </div>

      <div class="add-path">
        <InputText
          v-model="newPath"
          placeholder="Add custom path..."
          size="small"
          @keydown.enter="addCustomPath"
        />
        <Button
          icon="pi pi-plus"
          size="small"
          severity="secondary"
          outlined
          :disabled="!newPath.trim()"
          @click="addCustomPath"
        />
      </div>
    </section>

    <!-- Auto-backup toggles (FR-026) -->
    <section class="backup-section">
      <h4 class="section-title">
        Auto-Backup
      </h4>
      <div class="field-toggle">
        <ToggleSwitch v-model="backupBeforeUpdate" />
        <label>Backup before update</label>
      </div>
      <div class="field-toggle">
        <ToggleSwitch v-model="includeInScheduled" />
        <label>Include in scheduled backups</label>
      </div>
    </section>

    <!-- Actions + history -->
    <section class="backup-section">
      <div class="backup-section-header">
        <h4 class="section-title">
          Backup History
        </h4>
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
    </section>
  </div>
</template>

<style scoped>
.backup-tab {
  padding: 20px 0;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.backup-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.backup-section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.section-title {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--p-surface-300);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.path-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
}

.path-badge {
  flex-shrink: 0;
}

.path-text {
  font-size: 13px;
  font-family: "JetBrains Mono", "Fira Code", monospace;
  color: var(--p-surface-300);
}

.add-path {
  display: flex;
  gap: 8px;
  align-items: center;
}

.add-path .p-inputtext {
  flex: 1;
}

.field-toggle {
  display: flex;
  align-items: center;
  gap: 10px;
}

.field-toggle label {
  font-size: 13px;
  color: var(--p-surface-300);
}

.backup-empty {
  color: var(--p-surface-400);
  font-size: 14px;
  font-style: italic;
  padding: 8px 0;
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
