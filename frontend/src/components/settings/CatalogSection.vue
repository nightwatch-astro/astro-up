<script setup lang="ts">
import InputText from "primevue/inputtext";
import Dropdown from "primevue/dropdown";
import type { CatalogConfig } from "../../types/config";

const config = defineModel<CatalogConfig>({ required: true });

const cacheTtlOptions = [
  { label: "1 hour", value: "1h" },
  { label: "6 hours", value: "6h" },
  { label: "12 hours", value: "12h" },
  { label: "24 hours", value: "24h" },
  { label: "7 days", value: "168h" },
];
</script>

<template>
  <div class="settings-section">
    <div class="field">
      <label>Catalog URL</label>
      <InputText v-model="config.url" />
    </div>
    <div class="field">
      <label>Cache TTL</label>
      <Dropdown
        v-model="config.cache_ttl"
        :options="cacheTtlOptions"
        option-label="label"
        option-value="value"
      />
    </div>
  </div>
</template>

<style scoped>
.settings-section { display: flex; flex-direction: column; gap: 16px; }
.field { display: flex; flex-direction: column; gap: 4px; }
.field label { font-size: 13px; font-weight: 500; color: var(--p-surface-300); }
</style>
