<script setup lang="ts">
import ToggleSwitch from "primevue/toggleswitch";
import Dropdown from "primevue/dropdown";
import InputNumber from "primevue/inputnumber";
import type { BackupPolicyConfig } from "../../types/config";

const config = defineModel<BackupPolicyConfig>({ required: true });

const scheduleOptions = [
  { label: "Daily", value: "daily" },
  { label: "Weekly", value: "weekly" },
  { label: "Monthly", value: "monthly" },
];
</script>

<template>
  <div class="settings-section">
    <div class="field-toggle">
      <ToggleSwitch v-model="config.scheduled_enabled" />
      <label>Scheduled backups</label>
    </div>
    <div
      v-if="config.scheduled_enabled"
      class="field"
    >
      <label>Schedule</label>
      <Dropdown
        v-model="config.schedule"
        :options="scheduleOptions"
        option-label="label"
        option-value="value"
      />
    </div>

    <h4 class="subsection-title">
      Retention
    </h4>
    <div class="field">
      <label>Max backups per package (0 = unlimited)</label>
      <InputNumber
        v-model="config.max_per_package"
        :min="0"
      />
    </div>
    <div class="field">
      <label>Max total size (MB, 0 = unlimited)</label>
      <InputNumber
        v-model="config.max_total_size_mb"
        :min="0"
      />
    </div>
    <div class="field">
      <label>Max age (days, 0 = never expire)</label>
      <InputNumber
        v-model="config.max_age_days"
        :min="0"
      />
    </div>
  </div>
</template>

<style scoped>
.settings-section { display: flex; flex-direction: column; gap: 16px; }
.field { display: flex; flex-direction: column; gap: 4px; }
.field label { font-size: 13px; font-weight: 500; color: var(--p-surface-300); }
.field-toggle { display: flex; align-items: center; gap: 10px; }
.field-toggle label { font-size: 13px; color: var(--p-surface-300); }
.subsection-title { margin: 8px 0 0; font-size: 13px; font-weight: 600; color: var(--p-surface-200); }
</style>
