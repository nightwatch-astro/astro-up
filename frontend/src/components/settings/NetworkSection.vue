<script setup lang="ts">
import InputText from "primevue/inputtext";
import InputNumber from "primevue/inputnumber";
import Dropdown from "primevue/dropdown";
import type { NetworkConfig } from "../../types/config";

const config = defineModel<NetworkConfig>({ required: true });

const timeoutOptions = [
  { label: "5 seconds", value: "5s" },
  { label: "10 seconds", value: "10s" },
  { label: "30 seconds", value: "30s" },
  { label: "60 seconds", value: "60s" },
  { label: "120 seconds", value: "120s" },
];
</script>

<template>
  <div class="settings-section">
    <div class="field">
      <label>Proxy (leave empty for none)</label>
      <InputText
        :model-value="config.proxy ?? ''"
        @update:model-value="config.proxy = $event || null"
      />
    </div>
    <div class="field">
      <label>Connection Timeout</label>
      <Dropdown
        v-model="config.connect_timeout"
        :options="timeoutOptions"
        option-label="label"
        option-value="value"
      />
    </div>
    <div class="field">
      <label>Request Timeout</label>
      <Dropdown
        v-model="config.timeout"
        :options="timeoutOptions"
        option-label="label"
        option-value="value"
      />
    </div>
    <div class="field">
      <label>Download Speed Limit (bytes/s, 0 = unlimited)</label>
      <InputNumber
        v-model="config.download_speed_limit"
        :min="0"
      />
    </div>
  </div>
</template>

<style scoped>
.settings-section { display: flex; flex-direction: column; gap: 16px; }
.field { display: flex; flex-direction: column; gap: 4px; }
.field label { font-size: 13px; font-weight: 500; color: var(--p-surface-300); }
</style>
