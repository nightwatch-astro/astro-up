<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import InputText from "primevue/inputtext";
import Button from "primevue/button";
import PackageRow from "../components/installed/PackageRow.vue";
import ConfirmDialog from "../components/shared/ConfirmDialog.vue";
import EmptyState from "../components/shared/EmptyState.vue";
import { useSoftwareList, useUpdateSoftware, useScanInstalled, useCreateBackup } from "../composables/useInvoke";
import { useOperations } from "../composables/useOperations";
import type { PackageWithStatus } from "../types/package";

const router = useRouter();
const { data: software, isLoading } = useSoftwareList(() => "installed");
const updateMutation = useUpdateSoftware();
const scanMutation = useScanInstalled();
const backupMutation = useCreateBackup();
const { startOperation, isRunning } = useOperations();

const searchFilter = ref("");
const showUpdateConfirm = ref(false);
const showUpdateAllConfirm = ref(false);
const showScanConfirm = ref(false);
const pendingUpdatePkg = ref<PackageWithStatus | null>(null);

const installed = computed<PackageWithStatus[]>(() =>
  (software.value ?? []) as PackageWithStatus[],
);

const filtered = computed(() => {
  const q = searchFilter.value.trim().toLowerCase();
  if (!q) return installed.value;
  return installed.value.filter(
    (p) => p.name.toLowerCase().includes(q) || p.category.toLowerCase().includes(q),
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
  if (!startOperation("update-all", "Updating all packages")) return;
  for (const pkg of updatable.value) {
    updateMutation.mutate(pkg.id);
  }
}

function confirmScan() {
  if (!startOperation("scan", "Scanning installed software")) return;
  scanMutation.mutate();
}

function handleBackup(pkg: PackageWithStatus) {
  if (!startOperation(pkg.id, `Backing up ${pkg.name}`)) return;
  backupMutation.mutate([]);
}
</script>

<template>
  <div class="installed-view">
    <div class="installed-header">
      <h2 class="page-title">
        Installed
      </h2>
      <div class="header-actions">
        <InputText
          v-model="searchFilter"
          placeholder="Filter..."
          class="installed-search"
        />
        <Button
          label="Re-scan"
          icon="pi pi-refresh"
          severity="secondary"
          outlined
          :disabled="isRunning"
          @click="showScanConfirm = true"
        />
        <Button
          v-if="updatable.length > 0"
          :label="`Update All (${updatable.length})`"
          icon="pi pi-arrow-up"
          severity="warn"
          :disabled="isRunning"
          @click="showUpdateAllConfirm = true"
        />
      </div>
    </div>

    <EmptyState
      v-if="!isLoading && installed.length === 0"
      icon="pi-inbox"
      message="No installed software detected. Run a scan to detect installed packages."
      action-label="Scan Now"
      @action="showScanConfirm = true"
    />

    <template v-else>
      <section v-if="updatable.length > 0">
        <h3 class="section-label">
          Updates Available ({{ updatable.length }})
        </h3>
        <PackageRow
          v-for="pkg in updatable"
          :key="pkg.id"
          :pkg="pkg"
          @update="handleUpdate(pkg)"
          @backup="handleBackup(pkg)"
          @detail="router.push({ name: 'package-detail', params: { id: pkg.id } })"
        />
      </section>

      <section v-if="upToDate.length > 0">
        <h3 class="section-label">
          Up to Date ({{ upToDate.length }})
        </h3>
        <PackageRow
          v-for="pkg in upToDate"
          :key="pkg.id"
          :pkg="pkg"
          @backup="handleBackup(pkg)"
          @detail="router.push({ name: 'package-detail', params: { id: pkg.id } })"
        />
      </section>
    </template>

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
          {{ pkg.name }}: {{ pkg.installed_version }} → {{ pkg.latest_version }}
        </li>
      </ul>
    </ConfirmDialog>

    <ConfirmDialog
      v-model:visible="showScanConfirm"
      title="Re-scan"
      message="Scan your system for installed astrophotography software? This may take a moment."
      icon="pi-refresh"
      confirm-label="Scan"
      @confirm="confirmScan"
    />
  </div>
</template>

<style scoped>
.installed-view {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.installed-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.page-title {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--p-surface-0);
}

.header-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.installed-search {
  width: 200px;
}

.section-label {
  margin: 8px 0 4px;
  font-size: 13px;
  font-weight: 600;
  color: var(--p-surface-400);
  text-transform: uppercase;
  letter-spacing: 0.5px;
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
