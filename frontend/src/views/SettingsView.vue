<script setup lang="ts">
import { ref, reactive, watch } from "vue";
import Button from "primevue/button";
import { useToast } from "primevue/usetoast";
import ConfirmDialog from "../components/shared/ConfirmDialog.vue";
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
import type { AppConfig } from "../types/config";

const toast = useToast();
const { data: serverConfig } = useConfig();
const saveMutation = useSaveConfig();
const { applyTheme } = useTheme();

const activeSection = ref("general");
const showResetConfirm = ref(false);
let saveTimer: ReturnType<typeof setTimeout> | null = null;

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
  ui: {
    theme: "system", font_size: "medium", auto_scan_on_launch: true,
    default_install_scope: "user", default_install_method: "silent",
    auto_check_updates: true, check_interval: "1day",
    auto_notify_updates: true, auto_install_updates: false,
  },
  startup: { start_at_login: true, start_minimized: true, minimize_to_tray_on_close: true },
  notifications: {
    enabled: true, display_duration: 5,
    show_errors: true, show_warnings: true,
    show_update_available: true, show_operation_complete: true,
  },
  backup_policy: { scheduled_enabled: false, schedule: "weekly", max_per_package: 5, max_total_size_mb: 0, max_age_days: 0 },
  catalog: { url: "https://github.com/nightwatch-astro/astro-up-manifests/releases/download/catalog/latest/catalog.db", cache_ttl: "1day" },
  network: { proxy: null, connect_timeout: "10s", timeout: "30s", user_agent: "", download_speed_limit: 0 },
  paths: { download_dir: "", cache_dir: "", data_dir: "", keep_installers: false, purge_installers_after_days: 7 },
  updates: { auto_check: true, check_interval: "1day" },
  logging: { level: "info", log_to_file: true, log_file: "" },
  telemetry: { enabled: false },
};

const config = reactive<AppConfig>(structuredClone(defaultConfig));
let initialized = false;

// Load server config when available (JSON round-trip strips VueQuery's readonly proxy)
watch(serverConfig, (data) => {
  if (data) {
    Object.assign(config, JSON.parse(JSON.stringify(data)) as AppConfig);
    initialized = true;
  }
}, { immediate: true });

// Auto-save on any config change (debounced 500ms)
watch(config, () => {
  // Apply theme + font immediately (even before backend config loads)
  if (config.ui?.theme) {
    applyTheme(config.ui.theme);
  }
  if (config.ui?.font_size) {
    document.documentElement.dataset.fontSize = config.ui.font_size;
  }

  // Only persist to backend once real config has loaded
  if (!initialized) return;

  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    saveMutation.mutate(config as unknown as Record<string, unknown>, {
      onError: (err) => {
        toast.add({ severity: "error", summary: "Save failed", detail: String(err), life: 5000 });
      },
    });
  }, 500);
}, { deep: true });

function confirmReset() {
  Object.assign(config, structuredClone(defaultConfig));
  showResetConfirm.value = false;
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
        <Button
          v-if="activeSection !== 'about'"
          label="Reset to Defaults"
          severity="secondary"
          text
          size="small"
          @click="showResetConfirm = true"
        />
      </div>

      <GeneralSection
        v-if="activeSection === 'general'"
        v-model="config.ui"
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
        v-model="config.backup_policy"
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
      <AboutSection
        v-else-if="activeSection === 'about'"
      />
    </div>

    <ConfirmDialog
      v-model:visible="showResetConfirm"
      title="Reset to Defaults"
      message="This will reset all settings to their default values."
      icon="pi-refresh"
      confirm-label="Reset"
      severity="danger"
      @confirm="confirmReset"
    />
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
</style>
