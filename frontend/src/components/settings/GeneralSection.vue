<script setup lang="ts">
import Dropdown from "primevue/dropdown";
import ToggleSwitch from "primevue/toggleswitch";
import InputText from "primevue/inputtext";
import type { GeneralConfig } from "../../types/config";

const config = defineModel<GeneralConfig>({ required: true });

const themeOptions = [
  { label: "Dark", value: "dark" },
  { label: "Light", value: "light" },
  { label: "System", value: "system" },
];

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
</script>

<template>
  <div class="settings-section">
    <div class="field">
      <label>Theme</label>
      <Dropdown
        v-model="config.theme"
        :options="themeOptions"
        option-label="label"
        option-value="value"
      />
    </div>
    <div class="field">
      <label>Font Size</label>
      <Dropdown
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
      <Dropdown
        v-model="config.default_install_scope"
        :options="scopeOptions"
        option-label="label"
        option-value="value"
      />
    </div>
    <div class="field">
      <label>Default Install Method</label>
      <Dropdown
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
      <InputText v-model="config.check_interval" />
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
