<script setup lang="ts">
import { ref, computed, watch } from "vue";
import Select from "primevue/select";
import Button from "primevue/button";
import type { BackupListEntry } from "../../types/backup";

const props = defineProps<{
  backups: BackupListEntry[];
}>();

defineEmits<{
  restore: [archive: string];
}>();

const selectedApp = ref<string | null>(null);
const selectedBackup = ref<string | null>(null);

const apps = computed(() => {
  const seen = new Set<string>();
  for (const b of props.backups) seen.add(b.package_id);
  return [...seen].sort().map((id) => ({ label: id, value: id }));
});

const filteredBackups = computed(() => {
  if (!selectedApp.value) return [];
  return props.backups
    .filter((b) => b.package_id === selectedApp.value)
    .map((b) => ({
      label: `v${b.version} — ${new Date(b.created_at).toLocaleDateString()}`,
      value: b.archive_path,
    }));
});

watch(selectedApp, () => {
  selectedBackup.value = null;
});
</script>

<template>
  <div class="quick-restore">
    <h3 class="section-title">
      Quick Restore
    </h3>
    <div class="restore-controls">
      <Select
        v-model="selectedApp"
        :options="apps"
        option-label="label"
        option-value="value"
        placeholder="Select application..."
        class="restore-dropdown"
      />
      <Select
        v-model="selectedBackup"
        :options="filteredBackups"
        option-label="label"
        option-value="value"
        placeholder="Select backup..."
        :disabled="!selectedApp"
        class="restore-dropdown"
      />
      <Button
        label="Preview & Restore"
        icon="pi pi-replay"
        :disabled="!selectedBackup"
        @click="selectedBackup && $emit('restore', selectedBackup)"
      />
    </div>
  </div>
</template>

<style scoped>
.quick-restore {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--p-surface-200);
}

.restore-controls {
  display: flex;
  gap: 8px;
  align-items: center;
}

.restore-dropdown {
  width: 220px;
}
</style>
