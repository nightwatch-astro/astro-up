<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import InputText from "primevue/inputtext";
import IconField from "primevue/iconfield";
import InputIcon from "primevue/inputicon";
import Button from "primevue/button";
import PackageRow from "../components/installed/PackageRow.vue";
import ConfirmDialog from "../components/shared/ConfirmDialog.vue";
import EmptyState from "../components/shared/EmptyState.vue";
import { useSoftwareList, useInstallSoftware, useUpdateSoftware, useScanInstalled, useCreateBackup } from "../composables/useInvoke";
import { useOperations } from "../composables/useOperations";
import { useUpdateQueue } from "../composables/useUpdateQueue";
import { FEATURE_BACKUP } from "../features";
import type { PackageWithStatus } from "../types/package";

const router = useRouter();
const { data: software, isLoading } = useSoftwareList(() => "installed");
const installMutation = useInstallSoftware();
const updateMutation = useUpdateSoftware();
const scanMutation = useScanInstalled();
const backupMutation = useCreateBackup();
const { startOperation, isRunning } = useOperations();
const { enqueue, isActive: queueActive } = useUpdateQueue();

const searchFilter = ref("");
const showUpdateConfirm = ref(false);
const showUpdateAllConfirm = ref(false);
const showReinstallConfirm = ref(false);
const pendingUpdatePkg = ref<PackageWithStatus | null>(null);
const pendingReinstallPkg = ref<PackageWithStatus | null>(null);

// Backend "installed" filter returns only packages with ledger entries (post-scan).
// No client-side filtering needed — all returned packages are installed.
const installed = computed<PackageWithStatus[]>(() =>
  (software.value ?? []) as PackageWithStatus[],
);

const filtered = computed(() => {
  const q = searchFilter.value.trim().toLowerCase();
  if (!q) return installed.value;
  return installed.value.filter(
    (p) =>
      p.id.toLowerCase().includes(q) ||
      p.name.toLowerCase().includes(q) ||
      p.category.toLowerCase().includes(q),
  );
});

const updatable = computed(() => filtered.value.filter((p) => p.update_available));
const upToDate = computed(() => filtered.value.filter((p) => !p.update_available));

function handleUpdate(pkg: PackageWithStatus) {
  pendingUpdatePkg.value = pkg;
  showUpdateConfirm.value = true;
}

function confirmUpdate() {
  const pkg = pendingUpdatePkg.value;
  if (!pkg || !startOperation(pkg.id, `Updating ${pkg.name}`)) return;
  updateMutation.mutate(pkg.id);
  pendingUpdatePkg.value = null;
}

function confirmUpdateAll() {
  enqueue(updatable.value.map((p) => ({ id: p.id, name: p.name })));
}

function confirmScan() {
  if (!startOperation("scan", "Scanning installed software")) return;
  scanMutation.mutate();
}

function handleReinstall(pkg: PackageWithStatus) {
  pendingReinstallPkg.value = pkg;
  showReinstallConfirm.value = true;
}

function confirmReinstall() {
  const pkg = pendingReinstallPkg.value;
  if (!pkg || !startOperation(pkg.id, `Reinstalling ${pkg.name}`)) return;
  installMutation.mutate(pkg.id);
  pendingReinstallPkg.value = null;
}

function handleBackup(pkg: PackageWithStatus) {
  if (!FEATURE_BACKUP) return;
  if (!startOperation(pkg.id, `Backing up ${pkg.name}`)) return;
  backupMutation.mutate([]);
}
</script>

<template>
  <div class="page-view">
    <div class="page-hdr">
      <h2>Installed Software</h2>
      <p>{{ installed.length }} installed &middot; {{ updatable.length }} updates available</p>
    </div>

    <div class="installed-bar">
      <IconField class="installed-search">
        <InputIcon class="pi pi-search" />
        <InputText
          v-model="searchFilter"
          placeholder="Filter installed..."
        />
      </IconField>
      <Button
        v-if="updatable.length > 0"
        :label="`Update All (${updatable.length})`"
        icon="pi pi-download"
        severity="warn"
        size="small"
        :disabled="isRunning || queueActive"
        @click="showUpdateAllConfirm = true"
      />
      <Button
        label="Re-scan"
        icon="pi pi-refresh"
        severity="secondary"
        outlined
        size="small"
        :disabled="isRunning || queueActive"
        @click="confirmScan"
      />
    </div>

    <EmptyState
      v-if="!isLoading && installed.length === 0"
      icon="pi-inbox"
      message="No installed software detected. Run a scan to detect installed packages."
      action-label="Scan Now"
      @action="confirmScan"
    />

    <div
      v-else
      class="card installed-card"
    >
      <!-- Updates group -->
      <template v-if="updatable.length > 0">
        <div class="inst-group-hdr warn">
          <i class="pi pi-exclamation-triangle" />
          Updates Available ({{ updatable.length }})
        </div>
        <PackageRow
          v-for="pkg in updatable"
          :key="pkg.id"
          :pkg="pkg"
          :actions-disabled="isRunning || queueActive"
          @update="handleUpdate(pkg)"
          @reinstall="handleReinstall(pkg)"
          @backup="handleBackup(pkg)"
          @detail="router.push({ name: 'package-detail', params: { id: pkg.id } })"
        />
      </template>

      <!-- Up to date group -->
      <template v-if="upToDate.length > 0">
        <div class="inst-group-hdr ok">
          <i class="pi pi-check-circle" />
          Up to Date ({{ upToDate.length }})
        </div>
        <PackageRow
          v-for="pkg in upToDate"
          :key="pkg.id"
          :pkg="pkg"
          :actions-disabled="isRunning || queueActive"
          @backup="handleBackup(pkg)"
          @detail="router.push({ name: 'package-detail', params: { id: pkg.id } })"
        />
      </template>
    </div>

    <ConfirmDialog
      v-model:visible="showUpdateConfirm"
      title="Update Package"
      :message="`Update ${pendingUpdatePkg?.name} from ${pendingUpdatePkg?.installed_version} to ${pendingUpdatePkg?.latest_version}?`"
      icon="pi-arrow-up"
      confirm-label="Update"
      severity="warn"
      @confirm="confirmUpdate"
    />

    <ConfirmDialog
      v-model:visible="showUpdateAllConfirm"
      title="Update All"
      :message="`Update ${updatable.length} package(s) to their latest versions?`"
      icon="pi-arrow-up"
      confirm-label="Update All"
      severity="warn"
      @confirm="confirmUpdateAll"
    >
      <ul class="update-list">
        <li
          v-for="pkg in updatable"
          :key="pkg.id"
        >
          {{ pkg.name }}: {{ pkg.installed_version }} &rarr; {{ pkg.latest_version }}
        </li>
      </ul>
    </ConfirmDialog>

    <ConfirmDialog
      v-model:visible="showReinstallConfirm"
      title="Reinstall Package"
      :message="`Download and reinstall ${pendingReinstallPkg?.name}?`"
      icon="pi-refresh"
      confirm-label="Reinstall"
      @confirm="confirmReinstall"
    />

  </div>
</template>

<style scoped>
.installed-bar {
  display: flex;
  align-items: center;
  gap: 12px;
}

.installed-search {
  flex: 1;
}

.installed-card {
  overflow: hidden;
}

.inst-group-hdr {
  padding: 10px 16px;
  font-size: 12px;
  font-weight: 600;
  border-bottom: 1px solid var(--p-surface-700);
  display: flex;
  align-items: center;
  gap: 6px;
}

.inst-group-hdr.warn {
  background: color-mix(in srgb, var(--p-yellow-500) 10%, transparent);
  color: var(--p-yellow-400);
}

.inst-group-hdr.ok {
  background: var(--p-surface-900);
  color: var(--p-green-400);
}

.update-list {
  margin: 8px 0 0;
  padding-left: 20px;
  font-size: 13px;
  color: var(--p-surface-300);
}

.update-list li {
  line-height: 1.6;
}
</style>
