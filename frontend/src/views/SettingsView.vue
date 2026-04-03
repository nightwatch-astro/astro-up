<script setup lang="ts">
import { ref, reactive, watch } from "vue";
import Button from "primevue/button";
import { useToast } from "primevue/usetoast";
import { safeParse } from "valibot";
import GeneralSection from "../components/settings/GeneralSection.vue";
import StartupSection from "../components/settings/StartupSection.vue";
import NotificationsSection from "../components/settings/NotificationsSection.vue";
import BackupSection from "../components/settings/BackupSection.vue";
import CatalogSection from "../components/settings/CatalogSection.vue";
import NetworkSection from "../components/settings/NetworkSection.vue";
import PathsSection from "../components/settings/PathsSection.vue";
import LoggingSection from "../components/settings/LoggingSection.vue";
import AboutSection from "../components/settings/AboutSection.vue";
import { useConfig, useSaveConfig } from "../composables/useInvoke";
import { useTheme } from "../composables/useTheme";
import { useConfigSnapshots } from "../stores/configSnapshots";
import { AppConfigSchema } from "../validation/config";
import type { AppConfig } from "../types/config";

const toast = useToast();
const { data: serverConfig } = useConfig();
const saveMutation = useSaveConfig();
const { setTheme } = useTheme();
const { save: saveSnapshot } = useConfigSnapshots();

const activeSection = ref("general");
const errors = ref<string[]>([]);

const sections = [
  { id: "general", label: "General", icon: "pi-cog" },
  { id: "startup", label: "Startup", icon: "pi-power-off" },
  { id: "notifications", label: "Notifications", icon: "pi-bell" },
  { id: "backup", label: "Backup", icon: "pi-database" },
  { id: "catalog", label: "Catalog", icon: "pi-th-large" },
  { id: "network", label: "Network", icon: "pi-globe" },
  { id: "paths", label: "Paths", icon: "pi-folder" },
  { id: "logging", label: "Logging", icon: "pi-list" },
  { id: "about", label: "About", icon: "pi-info-circle" },
];

const defaultConfig: AppConfig = {
  general: {
    theme: "system", font_size: "medium", auto_scan_on_launch: true,
    default_install_scope: "user", default_install_method: "silent",
    auto_check_updates: true, check_interval: "24h",
    auto_notify_updates: true, auto_install_updates: false,
  },
  startup: { start_at_login: false, start_minimized: false, minimize_to_tray_on_close: true },
  notifications: {
    enabled: true, display_duration: 5,
    show_errors: true, show_warnings: true,
    show_update_available: true, show_operation_complete: true,
  },
  backup: { scheduled_enabled: false, schedule: "weekly", max_per_package: 5, max_total_size_mb: 0, max_age_days: 0 },
  catalog: { url: "https://github.com/nightwatch-astro/astro-up-catalog/releases/latest/download/catalog.db", cache_ttl: "24h" },
  network: { proxy: null, connect_timeout: "10s", timeout: "30s", download_speed_limit: 0 },
  paths: { download_dir: "", cache_dir: "", keep_installers: false, purge_installers_after_days: 7 },
  logging: { level: "info", log_to_file: true, log_file: "" },
};

const config = reactive<AppConfig>(structuredClone(defaultConfig));

// Load server config when available
watch(serverConfig, (data) => {
  if (data) {
    Object.assign(config, data as unknown as AppConfig);
  }
}, { immediate: true });

// Apply theme immediately when changed
watch(() => config.general.theme, (theme) => {
  setTheme(theme);
});

function validate(): boolean {
  const result = safeParse(AppConfigSchema, config);
  if (!result.success) {
    errors.value = result.issues.map(
      (i) => `${i.path?.map((p) => p.key).join(".")} — ${i.message}`,
    );
    return false;
  }
  errors.value = [];
  return true;
}

function handleSave() {
  if (!validate()) {
    toast.add({ severity: "error", summary: "Validation failed", detail: errors.value.join("; "), life: 5000 });
    return;
  }
  saveSnapshot(config);
  saveMutation.mutate(config as unknown as Record<string, unknown>, {
    onSuccess: () => {
      toast.add({ severity: "success", summary: "Settings saved", life: 3000 });
    },
    onError: (err) => {
      toast.add({ severity: "error", summary: "Save failed", detail: String(err), life: 5000 });
    },
  });
}

function handleReset() {
  Object.assign(config, structuredClone(defaultConfig));
  toast.add({ severity: "info", summary: "Reset to defaults", life: 3000 });
}
</script>

<template>
  <div class="settings-view">
    <div class="settings-sidebar">
      <button
        v-for="section in sections"
        :key="section.id"
        class="settings-nav-item"
        :class="{ active: activeSection === section.id }"
        @click="activeSection = section.id"
      >
        <i :class="['pi', section.icon]" />
        {{ section.label }}
      </button>
    </div>

    <div class="settings-content">
      <div class="settings-header">
        <h2 class="page-title">
          {{ sections.find((s) => s.id === activeSection)?.label }}
        </h2>
        <div
          v-if="activeSection !== 'about'"
          class="settings-actions"
        >
          <Button
            label="Reset to Defaults"
            severity="secondary"
            text
            @click="handleReset"
          />
          <Button
            label="Save Changes"
            icon="pi pi-check"
            @click="handleSave"
          />
        </div>
      </div>

      <div
        v-if="errors.length > 0"
        class="settings-errors"
      >
        <p
          v-for="(err, i) in errors"
          :key="i"
        >
          {{ err }}
        </p>
      </div>

      <GeneralSection
        v-if="activeSection === 'general'"
        v-model="config.general"
      />
      <StartupSection
        v-else-if="activeSection === 'startup'"
        v-model="config.startup"
      />
      <NotificationsSection
        v-else-if="activeSection === 'notifications'"
        v-model="config.notifications"
      />
      <BackupSection
        v-else-if="activeSection === 'backup'"
        v-model="config.backup"
      />
      <CatalogSection
        v-else-if="activeSection === 'catalog'"
        v-model="config.catalog"
      />
      <NetworkSection
        v-else-if="activeSection === 'network'"
        v-model="config.network"
      />
      <PathsSection
        v-else-if="activeSection === 'paths'"
        v-model="config.paths"
      />
      <LoggingSection
        v-else-if="activeSection === 'logging'"
        v-model="config.logging"
      />
      <AboutSection v-else-if="activeSection === 'about'" />
    </div>
  </div>
</template>

<style scoped>
.settings-view {
  display: flex;
  height: 100%;
}

.settings-sidebar {
  width: 200px;
  border-right: 1px solid var(--p-surface-700);
  padding: 16px 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex-shrink: 0;
}

.settings-nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--p-surface-300);
  font-size: 13px;
  cursor: pointer;
  text-align: left;
  transition: all 0.15s;
}

.settings-nav-item:hover {
  background: var(--p-surface-800);
  color: var(--p-surface-0);
}

.settings-nav-item.active {
  background: color-mix(in srgb, var(--p-primary-500) 20%, transparent);
  color: var(--p-primary-400);
}

.settings-nav-item i {
  font-size: 14px;
  width: 18px;
  text-align: center;
}

.settings-content {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.page-title {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--p-surface-0);
}

.settings-actions {
  display: flex;
  gap: 8px;
}

.settings-errors {
  background: color-mix(in srgb, var(--p-red-500) 10%, transparent);
  border: 1px solid var(--p-red-500);
  border-radius: 8px;
  padding: 12px;
  margin-bottom: 16px;
}

.settings-errors p {
  margin: 0;
  font-size: 13px;
  color: var(--p-red-400);
  line-height: 1.5;
}
</style>
