<script setup lang="ts">
import Dropdown from "primevue/dropdown";
import ToggleSwitch from "primevue/toggleswitch";
import type { LogConfig } from "../../types/config";

const config = defineModel<LogConfig>({ required: true });

const levelOptions = [
  { label: "Error", value: "error" },
  { label: "Warn", value: "warn" },
  { label: "Info", value: "info" },
  { label: "Debug", value: "debug" },
  { label: "Trace", value: "trace" },
];
</script>

<template>
  <div class="settings-section">
    <div class="field">
      <label>Log Level</label>
      <Dropdown
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
      v-if="config.log_to_file && config.log_file"
      class="log-path"
    >
      <span class="log-path-label">Log file:</span>
      <code class="log-path-value">{{ config.log_file }}</code>
    </div>
  </div>
</template>

<style scoped>
.settings-section { display: flex; flex-direction: column; gap: 16px; }
.field { display: flex; flex-direction: column; gap: 4px; }
.field label { font-size: 13px; font-weight: 500; color: var(--p-surface-300); }
.field-toggle { display: flex; align-items: center; gap: 10px; }
.field-toggle label { font-size: 13px; color: var(--p-surface-300); }
.log-path { display: flex; align-items: center; gap: 8px; }
.log-path-label { font-size: 12px; color: var(--p-surface-400); }
.log-path-value { font-size: 12px; color: var(--p-surface-300); background: var(--p-surface-800); padding: 4px 8px; border-radius: 4px; word-break: break-all; }
</style>
