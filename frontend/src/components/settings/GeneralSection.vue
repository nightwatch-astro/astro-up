<script setup lang="ts">
import Select from "primevue/select";
import ToggleSwitch from "primevue/toggleswitch";
import type { UiConfig } from "../../types/config";

const config = defineModel<UiConfig>({ required: true });

const fontOptions = [
  { label: "Small", value: "small" },
  { label: "Medium", value: "medium" },
  { label: "Large", value: "large" },
];

const scopeOptions = [
  { label: "Current User", value: "user" },
  { label: "All Users (Machine)", value: "machine" },
];

const methodOptions = [
  { label: "Silent", value: "silent" },
  { label: "Interactive", value: "interactive" },
];

// Values must match humantime_serde output format
const intervalOptions = [
  { label: "1 hour", value: "1h" },
  { label: "6 hours", value: "6h" },
  { label: "12 hours", value: "12h" },
  { label: "1 day", value: "1day" },
  { label: "2 days", value: "2days" },
  { label: "7 days", value: "7days" },
];
</script>

<template>
  <div class="settings-section">
    <div class="field">
      <label>Font Size</label>
      <Select
        v-model="config.font_size"
        :options="fontOptions"
        option-label="label"
        option-value="value"
      />
    </div>
    <div class="field-toggle">
      <ToggleSwitch v-model="config.auto_scan_on_launch" />
      <label>Auto-scan on launch</label>
    </div>
    <div class="field">
      <label>Default Install Scope</label>
      <Select
        v-model="config.default_install_scope"
        :options="scopeOptions"
        option-label="label"
        option-value="value"
      />
    </div>
    <div class="field">
      <label>Default Install Method</label>
      <Select
        v-model="config.default_install_method"
        :options="methodOptions"
        option-label="label"
        option-value="value"
      />
    </div>
    <div class="field-toggle">
      <ToggleSwitch v-model="config.auto_check_updates" />
      <label>Auto-check for updates</label>
    </div>
    <div
      v-if="config.auto_check_updates"
      class="field"
    >
      <label>Check Interval</label>
      <Select
        v-model="config.check_interval"
        :options="intervalOptions"
        option-label="label"
        option-value="value"
      />
    </div>
    <div class="field-toggle">
      <ToggleSwitch v-model="config.auto_notify_updates" />
      <label>Notify on updates</label>
    </div>
    <div class="field-toggle">
      <ToggleSwitch v-model="config.auto_install_updates" />
      <label>Auto-install updates</label>
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
