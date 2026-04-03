<script setup lang="ts">
import InputText from "primevue/inputtext";
import InputNumber from "primevue/inputnumber";
import ToggleSwitch from "primevue/toggleswitch";
import Button from "primevue/button";
import type { PathsConfig } from "../../types/config";

const config = defineModel<PathsConfig>({ required: true });

defineEmits<{
  clearCache: [];
  clearDownloads: [];
}>();
</script>

<template>
  <div class="settings-section">
    <div class="field">
      <label>Download Directory</label>
      <InputText v-model="config.download_dir" />
    </div>
    <div class="field">
      <label>Cache Directory</label>
      <InputText v-model="config.cache_dir" />
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
</style>
