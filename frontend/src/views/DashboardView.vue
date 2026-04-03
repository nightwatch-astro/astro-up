<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import Button from "primevue/button";
import ConfirmDialog from "../components/shared/ConfirmDialog.vue";
import { useSoftwareList, useUpdateCheck, useScanInstalled, useUpdateSoftware } from "../composables/useInvoke";
import { useOperations } from "../composables/useOperations";
import { mockActivity } from "../mocks";
import type { PackageWithStatus } from "../types/package";

const router = useRouter();
const { data: software } = useSoftwareList(() => "all");
const { data: updates } = useUpdateCheck();
const scanMutation = useScanInstalled();
const updateMutation = useUpdateSoftware();
const { startOperation, isRunning } = useOperations();

const showScanConfirm = ref(false);
const showUpdateAllConfirm = ref(false);

const installedCount = computed(() => {
  if (!software.value) return 0;
  return (software.value as PackageWithStatus[]).filter(
    (p) => p.detection.type === "Installed" || p.detection.type === "InstalledUnknownVersion",
  ).length;
});

const updateCount = computed(() => updates.value?.length ?? 0);

const updatablePackages = computed<PackageWithStatus[]>(() => {
  if (!software.value) return [];
  return (software.value as PackageWithStatus[]).filter((p) => p.update_available);
});

function confirmScan() {
  if (!startOperation("scan", "Scanning installed software")) return;
  scanMutation.mutate();
}

function confirmUpdateAll() {
  if (!startOperation("update-all", "Updating all packages")) return;
  for (const pkg of updatablePackages.value) {
    updateMutation.mutate(pkg.id);
  }
}

function relativeTime(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

const activityIcons: Record<string, string> = {
  install: "pi-download",
  update: "pi-arrow-up",
  scan: "pi-refresh",
  backup: "pi-database",
  restore: "pi-replay",
};
</script>

<template>
  <div class="dashboard-view">
    <h2 class="page-title">
      Dashboard
    </h2>

    <!-- Stats cards -->
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-value">
          {{ installedCount }}
        </div>
        <div class="stat-label">
          Installed
        </div>
      </div>
      <div
        class="stat-card clickable"
        @click="router.push('/installed')"
      >
        <div
          class="stat-value"
          :class="{ highlight: updateCount > 0 }"
        >
          {{ updateCount }}
        </div>
        <div class="stat-label">
          Updates
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-value">
          —
        </div>
        <div class="stat-label">
          Last Scan
        </div>
      </div>
      <div
        class="stat-card clickable"
        @click="router.push('/backup')"
      >
        <div class="stat-value">
          —
        </div>
        <div class="stat-label">
          Backups
        </div>
      </div>
    </div>

    <!-- Updates preview + quick actions -->
    <div class="dashboard-columns">
      <section class="dashboard-section">
        <h3 class="section-title">
          Updates Available
        </h3>
        <div
          v-if="updatablePackages.length === 0"
          class="empty-note"
        >
          All packages are up to date.
        </div>
        <div
          v-for="pkg in updatablePackages.slice(0, 5)"
          :key="pkg.id"
          class="update-row"
          @click="router.push({ name: 'package-detail', params: { id: pkg.id } })"
        >
          <span class="update-name">{{ pkg.name }}</span>
          <span class="update-version">
            {{ pkg.installed_version }} → {{ pkg.latest_version }}
          </span>
        </div>
      </section>

      <section class="dashboard-section">
        <h3 class="section-title">
          Quick Actions
        </h3>
        <div class="quick-actions">
          <Button
            label="Scan"
            icon="pi pi-refresh"
            severity="secondary"
            outlined
            :disabled="isRunning"
            @click="showScanConfirm = true"
          />
          <Button
            v-if="updateCount > 0"
            :label="`Update All (${updateCount})`"
            icon="pi pi-arrow-up"
            severity="warn"
            :disabled="isRunning"
            @click="showUpdateAllConfirm = true"
          />
        </div>
      </section>
    </div>

    <!-- Activity feed -->
    <section class="dashboard-section">
      <h3 class="section-title">
        Recent Activity
      </h3>
      <div
        v-for="entry in mockActivity"
        :key="entry.id"
        class="activity-row"
      >
        <i :class="['pi', activityIcons[entry.type] ?? 'pi-info-circle', 'activity-icon']" />
        <div class="activity-info">
          <span class="activity-name">{{ entry.name }}</span>
          <span class="activity-detail">{{ entry.detail }}</span>
        </div>
        <span class="activity-time">{{ relativeTime(entry.timestamp) }}</span>
      </div>
    </section>

    <ConfirmDialog
      v-model:visible="showScanConfirm"
      title="Scan"
      message="Scan your system for installed astrophotography software?"
      icon="pi-refresh"
      confirm-label="Scan"
      @confirm="confirmScan"
    />

    <ConfirmDialog
      v-model:visible="showUpdateAllConfirm"
      title="Update All"
      :message="`Update ${updateCount} package(s)?`"
      icon="pi-arrow-up"
      confirm-label="Update All"
      severity="warn"
      @confirm="confirmUpdateAll"
    />
  </div>
</template>

<style scoped>
.dashboard-view {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.page-title {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--p-surface-0);
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
}

.stat-card {
  background: var(--p-surface-800);
  border: 1px solid var(--p-surface-700);
  border-radius: 10px;
  padding: 16px;
  text-align: center;
}

.stat-card.clickable {
  cursor: pointer;
}

.stat-card.clickable:hover {
  border-color: var(--p-surface-500);
}

.stat-value {
  font-size: 28px;
  font-weight: 700;
  color: var(--p-surface-0);
}

.stat-value.highlight {
  color: var(--p-yellow-400);
}

.stat-label {
  font-size: 13px;
  color: var(--p-surface-400);
  margin-top: 2px;
}

.dashboard-columns {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.dashboard-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--p-surface-200);
}

.empty-note {
  color: var(--p-surface-400);
  font-size: 13px;
  font-style: italic;
}

.update-row {
  display: flex;
  justify-content: space-between;
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
}

.update-row:hover {
  background: var(--p-surface-800);
}

.update-name {
  color: var(--p-surface-200);
  font-weight: 500;
}

.update-version {
  color: var(--p-yellow-400);
  font-size: 12px;
}

.quick-actions {
  display: flex;
  gap: 8px;
}

.activity-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 0;
}

.activity-icon {
  font-size: 14px;
  color: var(--p-surface-400);
  width: 20px;
  text-align: center;
}

.activity-info {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.activity-name {
  font-size: 13px;
  color: var(--p-surface-200);
}

.activity-detail {
  font-size: 12px;
  color: var(--p-surface-400);
}

.activity-time {
  font-size: 12px;
  color: var(--p-surface-500);
  flex-shrink: 0;
}
</style>
