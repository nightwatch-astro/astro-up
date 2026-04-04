<script setup lang="ts">
import Select from "primevue/select";
import InputText from "primevue/inputtext";
import InputGroup from "primevue/inputgroup";
import ToggleSwitch from "primevue/toggleswitch";
import Button from "primevue/button";
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
      <InputGroup>
        <InputText
          :model-value="config.log_file"
          readonly
        />
        <Button
          icon="pi pi-file"
          severity="secondary"
          @click="browseLogFile"
        />
      </InputGroup>
    </div>
  </div>
</template>

<style scoped>
.settings-section { display: flex; flex-direction: column; gap: 16px; }
.field { display: flex; flex-direction: column; gap: 4px; }
.field label { font-size: 13px; font-weight: 500; color: var(--p-surface-300); }
.field-toggle { display: flex; align-items: center; gap: 10px; }
.field-toggle label { font-size: 13px; color: var(--p-surface-300); }
</style>
