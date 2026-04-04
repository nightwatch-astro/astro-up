<script setup lang="ts">
import { ref } from "vue";
import InputText from "primevue/inputtext";
import Select from "primevue/select";
import Button from "primevue/button";
import { invoke } from "@tauri-apps/api/core";
import type { CatalogConfig } from "../../types/config";

const config = defineModel<CatalogConfig>({ required: true });
const resyncing = ref(false);

const cacheTtlOptions = [
  { label: "1 hour", value: "1h" },
  { label: "6 hours", value: "6h" },
  { label: "12 hours", value: "12h" },
  { label: "1 day", value: "1day" },
  { label: "7 days", value: "7days" },
];

async function redownloadCatalog() {
  resyncing.value = true;
  try {
    await invoke("sync_catalog", { force: true });
  } catch (e) {
    console.error("Catalog re-download failed:", e);
  } finally {
    resyncing.value = false;
  }
}
</script>

<template>
  <div class="settings-section">
    <div class="field">
      <label>Catalog URL</label>
      <InputText v-model="config.url" />
    </div>
    <div class="field">
      <label>Cache TTL</label>
      <Select
        v-model="config.cache_ttl"
        :options="cacheTtlOptions"
        option-label="label"
        option-value="value"
      />
    </div>
    <div class="field">
      <label>Re-download Catalog</label>
      <Button
        :label="resyncing ? 'Downloading...' : 'Re-download Now'"
        :icon="resyncing ? 'pi pi-spin pi-spinner' : 'pi pi-refresh'"
        :disabled="resyncing"
        severity="secondary"
        outlined
        @click="redownloadCatalog"
      />
    </div>
  </div>
</template>

<style scoped>
.settings-section { display: flex; flex-direction: column; gap: 16px; }
.field { display: flex; flex-direction: column; gap: 4px; }
.field label { font-size: 13px; font-weight: 500; color: var(--p-surface-300); }
</style>
