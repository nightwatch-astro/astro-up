<script setup lang="ts">
import Select from "primevue/select";
import ToggleSwitch from "primevue/toggleswitch";
import Button from "primevue/button";
import InputNumber from "primevue/inputnumber";
import type { LogConfig } from "../../types/config";
import { useFilePicker } from "../../composables/useFilePicker";

const config = defineModel<LogConfig>({ required: true });

const levelOptions = [
  { label: "Error", value: "error" },
  { label: "Warn", value: "warn" },
  { label: "Info", value: "info" },
  { label: "Debug", value: "debug" },
  { label: "Trace", value: "trace" },
];

const { pickLogFile } = useFilePicker();

async function browseLogFile() {
  const selected = await pickLogFile(config.value.log_file || undefined);
  if (selected) {
    config.value.log_file = selected;
  }
}
</script>

<template>
  <div class="settings-section">
    <div class="field">
      <label>Log Level</label>
      <Select
        v-model="config.level"
        :options="levelOptions"
        option-label="label"
        option-value="value"
      />
    </div>
    <div class="field-toggle">
      <ToggleSwitch v-model="config.log_to_file" />
      <label>Log to file</label>
    </div>
    <div
      v-if="config.log_to_file"
      class="field"
    >
      <label>Log file</label>
      <div class="path-row">
        <code class="path-display">{{ config.log_file || "Not set" }}</code>
        <Button
          label="Browse"
          icon="pi pi-file"
          outlined
          size="small"
          @click="browseLogFile"
        />
      </div>
    </div>
    <div class="field">
      <label>Delete logs older than (days)</label>
      <InputNumber
        v-model="config.max_age_days"
        :min="0"
        :max="9999"
        :use-grouping="false"
      />
      <small class="hint">0 = never delete old logs</small>
    </div>
  </div>
</template>

<style scoped>
.settings-section { display: flex; flex-direction: column; gap: 16px; }
.field { display: flex; flex-direction: column; gap: 4px; }
.field label { font-size: 13px; font-weight: 500; color: var(--p-surface-300); }
.field-toggle { display: flex; align-items: center; gap: 10px; }
.field-toggle label { font-size: 13px; color: var(--p-surface-300); }
.path-row { display: flex; align-items: center; gap: 8px; }
.hint { font-size: 11px; color: var(--p-surface-500); }
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
