<script setup lang="ts">
import ToggleSwitch from "primevue/toggleswitch";
import Select from "primevue/select";
import type { NotificationsConfig } from "../../types/config";

const config = defineModel<NotificationsConfig>({ required: true });

const durationOptions = [
  { label: "3 seconds", value: 3 },
  { label: "5 seconds", value: 5 },
  { label: "10 seconds", value: 10 },
  { label: "Never auto-dismiss", value: 0 },
];
</script>

<template>
  <div class="settings-section">
    <div class="field-toggle">
      <ToggleSwitch v-model="config.enabled" />
      <label>Enable notifications</label>
    </div>
    <div
      v-if="config.enabled"
      class="field"
    >
      <label>Display Duration</label>
      <Select
        v-model="config.display_duration"
        :options="durationOptions"
        option-label="label"
        option-value="value"
      />
    </div>
    <template v-if="config.enabled">
      <div class="field-toggle">
        <ToggleSwitch v-model="config.show_errors" />
        <label>Show errors</label>
      </div>
      <div class="field-toggle">
        <ToggleSwitch v-model="config.show_warnings" />
        <label>Show warnings</label>
      </div>
      <div class="field-toggle">
        <ToggleSwitch v-model="config.show_update_available" />
        <label>Show update available</label>
      </div>
      <div class="field-toggle">
        <ToggleSwitch v-model="config.show_operation_complete" />
        <label>Show operation complete</label>
      </div>
    </template>
  </div>
</template>

<style scoped>
.settings-section { display: flex; flex-direction: column; gap: 16px; }
.field { display: flex; flex-direction: column; gap: 4px; }
.field label { font-size: 13px; font-weight: 500; color: var(--p-surface-300); }
.field-toggle { display: flex; align-items: center; gap: 10px; }
.field-toggle label { font-size: 13px; color: var(--p-surface-300); }
</style>
