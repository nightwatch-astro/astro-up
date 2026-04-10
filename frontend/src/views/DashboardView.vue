<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import Button from "primevue/button";
import ConfirmDialog from "../components/shared/ConfirmDialog.vue";
import SurveyDialog from "../components/shared/SurveyDialog.vue";
import PackageIcon from "../components/shared/PackageIcon.vue";
import { FEATURE_BACKUP } from "../features";
import { useSoftwareList, useScanInstalled, useUpdateSoftware, useLastScan } from "../composables/useInvoke";
import { useOperations } from "../composables/useOperations";
import { useUpdateQueue } from "../composables/useUpdateQueue";
import { logger } from "../utils/logger";
import type { PackageWithStatus } from "../types/package";

const router = useRouter();
const { data: software } = useSoftwareList(() => "all");
const { data: installedSoftware } = useSoftwareList(() => "installed");
const scanMutation = useScanInstalled();
const updateMutation = useUpdateSoftware();
const { isRunning, startOperation } = useOperations();
const { enqueue, isActive: queueActive } = useUpdateQueue();
const { data: lastScanData } = useLastScan();

const showUpdateAllConfirm = ref(false);
const showSingleUpdateConfirm = ref(false);
const showSurvey = ref(false);

onMounted(async () => {
  try {
    const eligible = await invoke<boolean>("check_survey_eligible");
    if (eligible) showSurvey.value = true;
  } catch (e) {
    logger.debug("DashboardView", `survey eligibility check failed: ${e}`);
  }
});
const pendingUpdatePkg = ref<PackageWithStatus | null>(null);

const catalogCount = computed(() => software.value?.length ?? 0);

const installedCount = computed(() => installedSoftware.value?.length ?? 0);

const updatablePackages = computed<PackageWithStatus[]>(() => {
  if (!software.value) return [];
  return (software.value as PackageWithStatus[]).filter((p) => p.update_available);
});

const updateCount = computed(() => updatablePackages.value.length);

const hasScanned = computed(() => installedCount.value > 0);

const lastScanLabel = computed(() => {
  const ts = lastScanData.value?.last_scan_at;
  if (!ts) return hasScanned.value ? `${installedCount.value} found` : "\u2014";
  const date = new Date(ts + "Z");
  return date.toLocaleDateString(undefined, { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" });
});

function runScan() {
  if (!startOperation("scan", "Scanning installed software")) return;
  logger.debug("DashboardView", "scan installed clicked");
  scanMutation.mutate();
}

function confirmUpdateAll() {
  logger.debug("DashboardView", "update all clicked");
  enqueue(updatablePackages.value.map((p) => ({ id: p.id, name: p.name })));
}

function handleSingleUpdate(pkg: PackageWithStatus) {
  pendingUpdatePkg.value = pkg;
  showSingleUpdateConfirm.value = true;
}

function confirmSingleUpdate() {
  const pkg = pendingUpdatePkg.value;
  if (!pkg || !startOperation(pkg.id, `Updating ${pkg.name}`)) return;
  logger.debug("DashboardView", `update clicked: ${pkg.id}`);
  updateMutation.mutate(pkg.id);
  pendingUpdatePkg.value = null;
}

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
        v-if="FEATURE_BACKUP"
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
            :disabled="isRunning || queueActive"
            @click.stop="handleSingleUpdate(pkg)"
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
        :disabled="isRunning || queueActive"
        @click="runScan"
      />
      <Button
        v-if="updateCount > 0"
        :label="`Update All (${updateCount})`"
        icon="pi pi-download"
        severity="secondary"
        outlined
        :disabled="isRunning || queueActive"
        @click="showUpdateAllConfirm = true"
      />
    </div>

    <SurveyDialog
      v-model:visible="showSurvey"
    />

    <ConfirmDialog
      v-model:visible="showUpdateAllConfirm"
      title="Update All"
      :message="`Update ${updateCount} package(s)?`"
      icon="pi-download"
      confirm-label="Update All"
      severity="warn"
      @confirm="confirmUpdateAll"
    />

    <ConfirmDialog
      v-model:visible="showSingleUpdateConfirm"
      title="Update Package"
      :message="`Update ${pendingUpdatePkg?.name} to ${pendingUpdatePkg?.latest_version}?`"
      icon="pi-arrow-up"
      confirm-label="Update"
      severity="warn"
      @confirm="confirmSingleUpdate"
    />

    <SurveyDialog
      v-model:visible="showSurvey"
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

</style>
