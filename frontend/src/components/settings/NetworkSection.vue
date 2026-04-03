<script setup lang="ts">
import InputText from "primevue/inputtext";
import InputNumber from "primevue/inputnumber";
import type { NetworkConfig } from "../../types/config";

const config = defineModel<NetworkConfig>({ required: true });
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
      <InputText v-model="config.connect_timeout" />
    </div>
    <div class="field">
      <label>Request Timeout</label>
      <InputText v-model="config.timeout" />
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
