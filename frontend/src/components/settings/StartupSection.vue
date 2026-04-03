<script setup lang="ts">
import ToggleSwitch from "primevue/toggleswitch";
import type { StartupConfig } from "../../types/config";

const config = defineModel<StartupConfig>({ required: true });
</script>

<template>
  <div class="settings-section">
    <div class="field-toggle">
      <ToggleSwitch v-model="config.start_at_login" />
      <label>Start at login</label>
    </div>
    <div class="field-toggle">
      <ToggleSwitch v-model="config.start_minimized" />
      <label>Start minimized</label>
    </div>
    <div class="field-toggle">
      <ToggleSwitch v-model="config.minimize_to_tray_on_close" />
      <label>Minimize to tray on close</label>
    </div>
  </div>
</template>

<style scoped>
.settings-section { display: flex; flex-direction: column; gap: 16px; }
.field-toggle { display: flex; align-items: center; gap: 10px; }
.field-toggle label { font-size: 13px; color: var(--p-surface-300); }
</style>
