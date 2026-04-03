<script setup lang="ts">
import InputText from "primevue/inputtext";
import type { CatalogConfig } from "../../types/config";

const config = defineModel<CatalogConfig>({ required: true });
</script>

<template>
  <div class="settings-section">
    <div class="field">
      <label>Catalog URL</label>
      <InputText v-model="config.url" />
    </div>
    <div class="field">
      <label>Cache TTL</label>
      <InputText v-model="config.cache_ttl" />
    </div>
  </div>
</template>

<style scoped>
.settings-section { display: flex; flex-direction: column; gap: 16px; }
.field { display: flex; flex-direction: column; gap: 4px; }
.field label { font-size: 13px; font-weight: 500; color: var(--p-surface-300); }
</style>
