<script setup lang="ts">
import InputNumber from "primevue/inputnumber";
import ToggleSwitch from "primevue/toggleswitch";
import Button from "primevue/button";
import type { PathsConfig } from "../../types/config";
import { useFilePicker } from "../../composables/useFilePicker";

const config = defineModel<PathsConfig>({ required: true });

defineEmits<{
  clearCache: [];
  clearDownloads: [];
}>();

const { pickDirectory } = useFilePicker();

async function browseDirectory(field: keyof PathsConfig) {
  const current = config.value[field];
  const selected = await pickDirectory(
    typeof current === "string" ? current : undefined,
  );
  if (selected) {
    (config.value[field] as string) = selected;
  }
}
</script>

<template>
  <div class="settings-section">
    <div class="field">
      <label>Download Directory</label>
      <div class="path-row">
        <code class="path-display">{{ config.download_dir || "Not set" }}</code>
        <Button
          label="Browse"
          icon="pi pi-folder-open"
          outlined
          size="small"
          @click="browseDirectory('download_dir')"
        />
      </div>
    </div>
    <div class="field">
      <label>Cache Directory</label>
      <div class="path-row">
        <code class="path-display">{{ config.cache_dir || "Not set" }}</code>
        <Button
          label="Browse"
          icon="pi pi-folder-open"
          outlined
          size="small"
          @click="browseDirectory('cache_dir')"
        />
      </div>
    </div>
    <div class="field">
      <label>Data Directory</label>
      <div class="path-row">
        <code class="path-display">{{ config.data_dir || "Not set" }}</code>
        <Button
          label="Browse"
          icon="pi pi-folder-open"
          outlined
          size="small"
          @click="browseDirectory('data_dir')"
        />
      </div>
    </div>
    <div class="field">
      <label>Portable Apps Directory</label>
      <div class="path-row">
        <code class="path-display">{{ config.portable_apps_dir || "Not set" }}</code>
        <Button
          label="Browse"
          icon="pi pi-folder-open"
          outlined
          size="small"
          @click="browseDirectory('portable_apps_dir')"
        />
      </div>
    </div>
    <div class="field-toggle">
      <ToggleSwitch v-model="config.keep_installers" />
      <label>Keep installers after install</label>
    </div>
    <div class="field">
      <label>Purge installers after (days)</label>
      <InputNumber
        v-model="config.purge_installers_after_days"
        :min="0"
      />
    </div>
    <div class="field-actions">
      <Button
        label="Clear Cache"
        icon="pi pi-trash"
        severity="secondary"
        outlined
        size="small"
        @click="$emit('clearCache')"
      />
      <Button
        label="Clear Downloads"
        icon="pi pi-trash"
        severity="secondary"
        outlined
        size="small"
        @click="$emit('clearDownloads')"
      />
    </div>
  </div>
</template>

<style scoped>
.settings-section { display: flex; flex-direction: column; gap: 16px; }
.field { display: flex; flex-direction: column; gap: 4px; }
.field label { font-size: 13px; font-weight: 500; color: var(--p-surface-300); }
.field-toggle { display: flex; align-items: center; gap: 10px; }
.field-toggle label { font-size: 13px; color: var(--p-surface-300); }
.field-actions { display: flex; gap: 8px; }
.path-row { display: flex; align-items: center; gap: 8px; }
.path-display {
  flex: 1;
  font-size: 12px;
  color: var(--p-surface-300);
  background: var(--p-surface-800);
  padding: 6px 10px;
  border-radius: 6px;
  word-break: break-all;
  min-height: 20px;
}
</style>
