<script setup lang="ts">
import { computed, ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import { listen } from "@tauri-apps/api/event";
import Button from "primevue/button";
import ConfirmDialog from "../components/shared/ConfirmDialog.vue";
import PackageIcon from "../components/shared/PackageIcon.vue";
import { useSoftwareList, useUpdateCheck, useScanInstalled, useUpdateAll, useActivity } from "../composables/useInvoke";
import { useOperations } from "../composables/useOperations";
import { logger } from "../utils/logger";
import type { PackageWithStatus } from "../types/package";

const router = useRouter();
const { data: software } = useSoftwareList(() => "all");
const { data: installedSoftware } = useSoftwareList(() => "installed");
const { data: updates } = useUpdateCheck();
const scanMutation = useScanInstalled();
const updateAllMutation = useUpdateAll();
const { isRunning, startOperation } = useOperations();
const { data: activity } = useActivity(10);

const showUpdateAllConfirm = ref(false);
const lastScanTime = ref<Date | null>(null);

const catalogCount = computed(() => software.value?.length ?? 0);

const installedCount = computed(() => installedSoftware.value?.length ?? 0);

const updateCount = computed(() => updates.value?.length ?? 0);

const updatablePackages = computed<PackageWithStatus[]>(() => {
  if (!software.value) return [];
  return (software.value as PackageWithStatus[]).filter((p) => p.update_available);
});

const lastScanLabel = computed(() => {
  if (!lastScanTime.value) return "\u2014";
  const now = new Date();
  const diff = now.getTime() - lastScanTime.value.getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "Just now";
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  return lastScanTime.value.toLocaleDateString();
});

function runScan() {
  if (!startOperation("scan", "Scanning installed software")) return;
  logger.debug("DashboardView", "scan installed clicked");
  scanMutation.mutate();
}

function confirmUpdateAll() {
  logger.debug("DashboardView", "update all clicked");
  updateAllMutation.mutate();
}

interface ActivityRecord {
  id: number;
  package_id: string;
  operation_type: string;
  from_version: string | null;
  to_version: string | null;
  status: string;
  duration_ms: number;
  error_message: string | null;
  created_at: string;
}

const activityRecords = computed<ActivityRecord[]>(() =>
  (activity.value ?? []) as ActivityRecord[],
);

function activityIcon(record: ActivityRecord): string {
  if (record.status === "failed") return "pi-times-circle";
  switch (record.operation_type) {
    case "install": return "pi-download";
    case "update": return "pi-arrow-up";
    case "uninstall": return "pi-trash";
    default: return "pi-info-circle";
  }
}

function activityLabel(record: ActivityRecord): string {
  const verb = record.operation_type === "install" ? "Installed"
    : record.operation_type === "update" ? "Updated"
    : record.operation_type === "uninstall" ? "Uninstalled"
    : record.operation_type;
  if (record.status === "failed") return `${verb} ${record.package_id} (failed)`;
  return `${verb} ${record.package_id}`;
}

function activityDetail(record: ActivityRecord): string {
  const parts: string[] = [];
  if (record.to_version) parts.push(`v${record.to_version}`);
  if (record.from_version && record.to_version) parts[0] = `${record.from_version} \u2192 ${record.to_version}`;
  const date = new Date(record.created_at);
  parts.push(date.toLocaleDateString(undefined, { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" }));
  return parts.join(" \u00B7 ");
}

onMounted(async () => {
  await listen("core-event", (event) => {
    const payload = event.payload as { type?: string };
    if (payload.type === "scan_complete") {
      lastScanTime.value = new Date();
    }
  });
});
</script>

<template>
  <div class="page-view">
    <div class="page-hdr">
      <h2>Dashboard</h2>
      <p>Overview of your astrophotography software</p>
    </div>

    <!-- Stats cards -->
    <div class="stats-grid">
      <div class="stat-card card">
        <div class="stat-label">
          Installed
        </div>
        <div class="stat-value">
          {{ installedCount }}
        </div>
        <div class="stat-sub">
          of {{ catalogCount }} in catalog
        </div>
      </div>
      <div
        class="stat-card card warn clickable"
        @click="router.push('/installed')"
      >
        <div class="stat-label">
          Updates Available
        </div>
        <div class="stat-value">
          {{ updateCount }}
        </div>
        <div class="stat-sub">
          Review and install
        </div>
      </div>
      <div class="stat-card card">
        <div class="stat-label">
          Last Scan
        </div>
        <div class="stat-value scan-val">
          {{ lastScanLabel }}
        </div>
        <div class="stat-sub">
          {{ installedCount }} packages detected
        </div>
      </div>
      <div
        class="stat-card card clickable"
        @click="router.push('/backup')"
      >
        <div class="stat-label">
          Backups
        </div>
        <div class="stat-value">
          &mdash;
        </div>
        <div class="stat-sub">
&nbsp;
        </div>
      </div>
    </div>

    <!-- Updates Available -->
    <template v-if="updatablePackages.length > 0">
      <div class="section-title">
        <i class="pi pi-exclamation-circle section-title-icon" />
        Updates Available
      </div>
      <div class="card updates-card">
        <div
          v-for="pkg in updatablePackages.slice(0, 5)"
          :key="pkg.id"
          class="upd-row"
          @click="router.push({ name: 'package-detail', params: { id: pkg.id } })"
        >
          <PackageIcon
            :icon-base64="pkg.icon_base64"
            :category="pkg.category"
            size="md"
          />
          <div class="upd-info">
            <div class="upd-name">
              {{ pkg.name }}
            </div>
            <div class="upd-sub">
              {{ pkg.category }} &middot; {{ pkg.software_type }}
            </div>
          </div>
          <div class="upd-arrow">
            {{ pkg.installed_version }} &rarr; {{ pkg.latest_version }}
          </div>
          <Button
            label="Update"
            severity="warn"
            size="small"
            @click.stop="showUpdateAllConfirm = true"
          />
        </div>
        <div class="upd-footer">
          <Button
            label="View all installed"
            icon="pi pi-arrow-right"
            text
            size="small"
            severity="secondary"
            @click="router.push('/installed')"
          />
        </div>
      </div>
    </template>

    <!-- Quick Actions -->
    <div class="quick-actions">
      <Button
        label="Scan Installed"
        icon="pi pi-refresh"
        :disabled="isRunning"
        @click="runScan"
      />
      <Button
        v-if="updateCount > 0"
        :label="`Update All (${updateCount})`"
        icon="pi pi-download"
        severity="secondary"
        outlined
        :disabled="isRunning"
        @click="showUpdateAllConfirm = true"
      />
    </div>

    <!-- Activity feed -->
    <div class="section-title">
      Recent Activity
    </div>
    <div class="card activity-card">
      <template v-if="activityRecords.length > 0">
        <div
          v-for="record in activityRecords"
          :key="record.id"
          class="act-row"
        >
          <div
            class="act-icon"
            :class="record.status === 'failed' ? 'act-error' : 'act-ok'"
          >
            <i :class="'pi ' + activityIcon(record)" />
          </div>
          <div class="act-text">
            <div class="act-name">
              {{ activityLabel(record) }}
            </div>
            <div class="act-det">
              {{ activityDetail(record) }}
            </div>
          </div>
        </div>
      </template>
      <template v-else-if="lastScanTime">
        <div class="act-row">
          <div class="act-icon act-scan">
            <i class="pi pi-search" />
          </div>
          <div class="act-text">
            <div class="act-name">
              Scan completed
            </div>
            <div class="act-det">
              {{ installedCount }} packages detected &middot; {{ lastScanLabel }}
            </div>
          </div>
        </div>
      </template>
      <div
        v-else
        class="act-row"
      >
        <div class="act-icon act-scan">
          <i class="pi pi-info-circle" />
        </div>
        <div class="act-text">
          <div class="act-name">
            No activity yet
          </div>
          <div class="act-det">
            Run a scan to detect installed software
          </div>
        </div>
      </div>
    </div>

    <ConfirmDialog
      v-model:visible="showUpdateAllConfirm"
      title="Update All"
      :message="`Update ${updateCount} package(s)?`"
      icon="pi-download"
      confirm-label="Update All"
      severity="warn"
      @confirm="confirmUpdateAll"
    />
  </div>
</template>

<style scoped>
/* Stats grid */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 14px;
}

.stat-card {
  padding: 18px;
}

.stat-label {
  font-size: 11px;
  color: var(--p-surface-500);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.stat-value {
  font-size: 28px;
  font-weight: 700;
  color: var(--p-surface-0);
  margin-top: 2px;
}

.stat-value.scan-val {
  font-size: 18px;
}

.stat-sub {
  font-size: 11px;
  color: var(--p-surface-400);
  margin-top: 2px;
}

.stat-card.warn .stat-value {
  color: var(--p-yellow-400);
}

.stat-card.clickable {
  cursor: pointer;
}

.stat-card.clickable:hover {
  border-color: var(--p-surface-500);
}

/* Section title */
.section-title-icon {
  color: var(--p-yellow-400);
  font-size: 13px;
  margin-right: 6px;
}

/* Updates card */
.updates-card {
  overflow: hidden;
}

.upd-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--p-surface-700);
  cursor: pointer;
  transition: background 0.1s;
}

.upd-row:last-child {
  border-bottom: none;
}

.upd-row:hover {
  background: rgba(255, 255, 255, 0.02);
}

.upd-icon {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  background: var(--p-surface-700);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 15px;
  color: var(--p-primary-400);
  flex-shrink: 0;
}

.upd-info {
  flex: 1;
  min-width: 0;
}

.upd-name {
  font-size: 13px;
  color: var(--p-surface-0);
  font-weight: 500;
}

.upd-sub {
  font-size: 11px;
  color: var(--p-surface-500);
  margin-top: 1px;
}

.upd-arrow {
  color: var(--p-yellow-400);
  font-size: 12px;
  flex-shrink: 0;
}

.upd-footer {
  padding: 10px 16px;
  text-align: right;
}

/* Quick Actions */
.quick-actions {
  display: flex;
  gap: 10px;
}

/* Activity card */
.activity-card {
  overflow: hidden;
}

.act-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 11px 16px;
  border-bottom: 1px solid var(--p-surface-700);
}

.act-row:last-child {
  border-bottom: none;
}

.act-icon {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  flex-shrink: 0;
}

.act-icon.act-install {
  background: color-mix(in srgb, var(--p-green-500) 20%, transparent);
  color: var(--p-green-400);
}

.act-icon.act-update {
  background: color-mix(in srgb, var(--p-yellow-500) 15%, transparent);
  color: var(--p-yellow-400);
}

.act-icon.act-scan {
  background: color-mix(in srgb, var(--p-indigo-500) 20%, transparent);
  color: var(--p-indigo-400);
}

.act-text {
  flex: 1;
  min-width: 0;
}

.act-name {
  font-size: 13px;
  color: var(--p-surface-0);
}

.act-det {
  font-size: 11px;
  color: var(--p-surface-500);
}

.act-time {
  font-size: 11px;
  color: var(--p-surface-600);
  flex-shrink: 0;
  margin-left: auto;
}
</style>
