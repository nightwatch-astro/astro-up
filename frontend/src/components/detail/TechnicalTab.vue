<script setup lang="ts">
import type { PackageWithStatus, InstallMethod, InstallScope, InstallElevation } from "../../types/package";

defineProps<{
  pkg: PackageWithStatus;
}>();

function detectionDetails(pkg: PackageWithStatus): Record<string, string> {
  if (!pkg.detection) return { Status: "Not scanned" };
  switch (pkg.detection.type) {
    case "Installed":
      return { Method: pkg.detection.method, Version: pkg.detection.version, Status: "Installed" };
    case "InstalledUnknownVersion":
      return { Method: pkg.detection.method, Status: "Installed (version unknown)" };
    case "NotInstalled":
      return { Status: "Not installed" };
    case "Unavailable":
      return { Status: "Unavailable", Reason: pkg.detection.reason };
  }
}

const methodLabels: Record<InstallMethod, string> = {
  exe: "Exe",
  msi: "MSI",
  inno_setup: "InnoSetup",
  nsis: "NSIS",
  wix: "WiX",
  burn: "Burn",
  zip: "Zip",
  portable: "Portable",
  download_only: "Download Only",
};

const scopeLabels: Record<InstallScope, string> = {
  machine: "Machine",
  user: "User",
  either: "Either",
};

const elevationLabels: Record<InstallElevation, string> = {
  required: "Required",
  prohibited: "Prohibited",
  self: "Self",
};
</script>

<template>
  <div class="technical-tab">
    <section class="tech-section">
      <h3 class="section-title">
        Detection
      </h3>
      <div class="tech-grid">
        <div
          v-for="(value, key) in detectionDetails(pkg)"
          :key="key"
          class="tech-item"
        >
          <span class="tech-label">{{ key }}</span>
          <span class="tech-value">{{ value }}</span>
        </div>
      </div>
    </section>

    <section
      v-if="pkg.install"
      class="tech-section"
    >
      <h3 class="section-title">
        Install Method
      </h3>
      <div class="tech-grid">
        <div class="tech-item">
          <span class="tech-label">Method</span>
          <span class="tech-value">{{ methodLabels[pkg.install.method] ?? pkg.install.method }}</span>
        </div>
        <div
          v-if="pkg.install.scope"
          class="tech-item"
        >
          <span class="tech-label">Scope</span>
          <span class="tech-value">{{ scopeLabels[pkg.install.scope] ?? pkg.install.scope }}</span>
        </div>
        <div
          v-if="pkg.install.elevation"
          class="tech-item"
        >
          <span class="tech-label">Elevation</span>
          <span class="tech-value">{{ elevationLabels[pkg.install.elevation] ?? pkg.install.elevation }}</span>
        </div>
        <div
          v-if="pkg.install.zip_wrapped"
          class="tech-item"
        >
          <span class="tech-label">Zip Wrapped</span>
          <span class="tech-value">Yes</span>
        </div>
      </div>
    </section>

    <section class="tech-section">
      <h3 class="section-title">
        Package Info
      </h3>
      <div class="tech-grid">
        <div class="tech-item">
          <span class="tech-label">Type</span>
          <span class="tech-value">{{ pkg.software_type }}</span>
        </div>
        <div class="tech-item">
          <span class="tech-label">Manifest Version</span>
          <span class="tech-value">{{ pkg.manifest_version }}</span>
        </div>
        <div
          v-if="pkg.aliases.length > 0"
          class="tech-item"
        >
          <span class="tech-label">Aliases</span>
          <span class="tech-value">{{ pkg.aliases.join(", ") }}</span>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.technical-tab {
  padding: 20px 0;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.tech-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--p-surface-200);
}

.tech-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 12px;
}

.tech-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  background: var(--p-surface-800);
  padding: 14px;
  border-radius: 8px;
  border: 1px solid var(--p-surface-700);
}

.tech-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--p-surface-400);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.tech-value {
  font-size: 14px;
  color: var(--p-surface-200);
  font-family: "JetBrains Mono", "Fira Code", monospace;
}
</style>
