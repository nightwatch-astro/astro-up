<script setup lang="ts">
import { computed, ref } from "vue";
import Dropdown from "primevue/dropdown";
import Dialog from "primevue/dialog";
import QuickRestore from "../components/backup/QuickRestore.vue";
import RestorePreview from "../components/backup/RestorePreview.vue";
import BackupGroup from "../components/backup/BackupGroup.vue";
import ConfirmDialog from "../components/shared/ConfirmDialog.vue";
import EmptyState from "../components/shared/EmptyState.vue";
import FileTable from "../components/shared/FileTable.vue";
import { mockBackups, mockBackupContents, mockRestorePreview } from "../mocks";
import { useRestoreBackup } from "../composables/useInvoke";
import { useOperations } from "../composables/useOperations";

const restoreMutation = useRestoreBackup();
const { startOperation } = useOperations();

const appFilter = ref<string | null>(null);
const showDeleteConfirm = ref(false);
const pendingDelete = ref<string | null>(null);
const showPreview = ref(false);
const showRestoreFlow = ref(false);
const restoreArchive = ref<string | null>(null);

const apps = computed(() => {
  const seen = new Set<string>();
  for (const b of mockBackups) seen.add(b.package_id);
  return [...seen].sort();
});

const appOptions = computed(() => [
  { label: "All applications", value: null },
  ...apps.value.map((a) => ({ label: a, value: a })),
]);

const filteredBackups = computed(() => {
  if (!appFilter.value) return mockBackups;
  return mockBackups.filter((b) => b.package_id === appFilter.value);
});

const groupedBackups = computed(() => {
  const groups: Record<string, typeof mockBackups> = {};
  for (const b of filteredBackups.value) {
    if (!groups[b.package_id]) groups[b.package_id] = [];
    groups[b.package_id].push(b);
  }
  return Object.entries(groups).sort(([a], [b]) => a.localeCompare(b));
});

function handleRestore(archive: string) {
  restoreArchive.value = archive;
  showRestoreFlow.value = true;
}

function confirmRestore() {
  if (!restoreArchive.value) return;
  if (!startOperation("restore", "Restoring backup")) return;
  restoreMutation.mutate({ archive: restoreArchive.value });
  showRestoreFlow.value = false;
}

function handlePreview() {
  showPreview.value = true;
}

function handleDelete(archive: string) {
  pendingDelete.value = archive;
  showDeleteConfirm.value = true;
}

function confirmDelete() {
  // Mock: would call delete_backup command
  pendingDelete.value = null;
}
</script>

<template>
  <div class="backup-view">
    <h2 class="page-title">
      Backup &amp; Restore
    </h2>

    <QuickRestore
      :backups="mockBackups"
      @restore="handleRestore"
    />

    <div class="backup-list-header">
      <h3 class="section-title">
        All Backups
      </h3>
      <Dropdown
        v-model="appFilter"
        :options="appOptions"
        option-label="label"
        option-value="value"
        class="app-filter"
      />
    </div>

    <EmptyState
      v-if="filteredBackups.length === 0"
      icon="pi-database"
      message="No backups found."
    />

    <BackupGroup
      v-for="[appId, backups] in groupedBackups"
      :key="appId"
      :app-id="appId"
      :backups="backups"
      @preview="handlePreview"
      @delete="handleDelete"
    />

    <!-- Restore flow -->
    <Dialog
      v-model:visible="showRestoreFlow"
      header="Restore Preview"
      modal
      :style="{ width: '700px' }"
    >
      <RestorePreview
        :changes="mockRestorePreview"
        @confirm="confirmRestore"
        @cancel="showRestoreFlow = false"
      />
    </Dialog>

    <!-- Backup contents preview -->
    <Dialog
      v-model:visible="showPreview"
      header="Backup Contents"
      modal
      :style="{ width: '600px' }"
    >
      <FileTable
        mode="preview"
        :files="mockBackupContents.files"
      />
    </Dialog>

    <!-- Delete confirmation -->
    <ConfirmDialog
      v-model:visible="showDeleteConfirm"
      title="Delete Backup"
      message="This backup will be permanently deleted. This cannot be undone."
      icon="pi-trash"
      confirm-label="Delete"
      severity="danger"
      @confirm="confirmDelete"
    />
  </div>
</template>

<style scoped>
.backup-view {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.page-title {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--p-surface-0);
}

.backup-list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.section-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--p-surface-200);
}

.app-filter {
  width: 200px;
}
</style>
