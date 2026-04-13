<script setup lang="ts">
import { open as openPath } from "@tauri-apps/plugin-shell";
import type { PackageWithStatus } from "../../types/package";

defineProps<{
  pkg: PackageWithStatus;
}>();

function detectionMethod(pkg: PackageWithStatus): string {
  if (!pkg.detection) return "Not scanned";
  switch (pkg.detection.type) {
    case "Installed": return pkg.detection.method;
    case "InstalledUnknownVersion": return pkg.detection.method;
    case "NotInstalled": return "Not installed";
    case "Unavailable": return pkg.detection.reason;
  }
}
</script>

<template>
  <div class="info-grid">
    <div
      v-if="pkg.installed_version"
      class="info-item"
    >
      <span class="info-label">Installed Version</span>
      <span class="info-value">{{ pkg.installed_version === "0.0.0" ? "unknown" : pkg.installed_version }}</span>
    </div>
    <div
      v-if="pkg.latest_version"
      class="info-item"
    >
      <span class="info-label">Latest Version</span>
      <span class="info-value">{{ pkg.latest_version }}</span>
    </div>
    <div
      v-if="pkg.detection && 'install_path' in pkg.detection && pkg.detection.install_path"
      class="info-item"
    >
      <span class="info-label">Install Location</span>
      <span class="info-value info-path">
        {{ (pkg.detection as { install_path: string }).install_path }}
        <button
          class="path-open-btn"
          title="Open folder"
          @click="openPath((pkg.detection as { install_path: string }).install_path)"
        >
          <i class="pi pi-folder-open" />
        </button>
      </span>
    </div>
    <div class="info-item">
      <span class="info-label">Category</span>
      <span class="info-value">{{ pkg.category }}</span>
    </div>
    <div class="info-item">
      <span class="info-label">Type</span>
      <span class="info-value">{{ pkg.software_type }}</span>
    </div>
    <div
      v-if="pkg.license"
      class="info-item"
    >
      <span class="info-label">License</span>
      <span class="info-value">{{ pkg.license }}</span>
    </div>
    <div class="info-item">
      <span class="info-label">Detection</span>
      <span class="info-value">{{ detectionMethod(pkg) }}</span>
    </div>
    <div
      v-if="pkg.dependencies.length > 0"
      class="info-item info-wide"
    >
      <span class="info-label">Dependencies</span>
      <span class="info-value">{{ pkg.dependencies.join(", ") }}</span>
    </div>
    <div
      v-if="pkg.tags.length > 0"
      class="info-item info-wide"
    >
      <span class="info-label">Tags</span>
      <span class="info-value">{{ pkg.tags.join(", ") }}</span>
    </div>
  </div>
</template>

<style scoped>
.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 16px;
  padding: 20px 0;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  background: var(--p-surface-800);
  padding: 14px;
  border-radius: 8px;
  border: 1px solid var(--p-surface-700);
}

.info-wide {
  grid-column: 1 / -1;
}

.info-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--p-surface-400);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.info-value {
  font-size: 14px;
  color: var(--p-surface-200);
}

.info-path {
  display: flex;
  align-items: center;
  gap: 6px;
  word-break: break-all;
}

.path-open-btn {
  background: none;
  border: none;
  color: var(--p-primary-400);
  cursor: pointer;
  padding: 2px;
  font-size: 14px;
  flex-shrink: 0;
}

.path-open-btn:hover {
  color: var(--p-primary-300);
}
</style>
