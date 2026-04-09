<script setup lang="ts">
import { computed, ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import Tabs from "primevue/tabs";
import TabList from "primevue/tablist";
import Tab from "primevue/tab";
import TabPanels from "primevue/tabpanels";
import TabPanel from "primevue/tabpanel";
import DetailHero from "../components/detail/DetailHero.vue";
import OverviewTab from "../components/detail/OverviewTab.vue";
import VersionsTab from "../components/detail/VersionsTab.vue";
import BackupTab from "../components/detail/BackupTab.vue";
import TechnicalTab from "../components/detail/TechnicalTab.vue";
import ConfirmDialog from "../components/shared/ConfirmDialog.vue";
import EmptyState from "../components/shared/EmptyState.vue";
import { useSoftwareList, useVersions, useInstallSoftware, useUpdateSoftware, useCreateBackup } from "../composables/useInvoke";
import { useOperations } from "../composables/useOperations";
import { useUpdateQueue } from "../composables/useUpdateQueue";
import { logger } from "../utils/logger";
import type { PackageWithStatus, VersionEntry } from "../types/package";

const props = defineProps<{
  id: string;
}>();

const router = useRouter();
const { data: software, isLoading } = useSoftwareList(() => "all");
const { data: versions } = useVersions(() => props.id);
const installMutation = useInstallSoftware();
const updateMutation = useUpdateSoftware();
const backupMutation = useCreateBackup();
const { isRunning } = useOperations();
const { isActive: queueActive } = useUpdateQueue();
const actionsDisabled = computed(() => isRunning.value || queueActive.value);

const showBackupConfirm = ref(false);
const tabsReady = ref(false);
onMounted(() => { tabsReady.value = true; });

const pkg = computed<PackageWithStatus | undefined>(() => {
  if (!software.value) return undefined;
  const found = (software.value as PackageWithStatus[]).find((p) => p.id === props.id);
  if (!found) return undefined;
  // Merge latest_version from versions query without mutating the readonly VueQuery cache
  const vList = versions.value as VersionEntry[] | undefined;
  const latestVersion = found.latest_version ?? (vList?.length ? vList[0].version : undefined);
  return { ...found, latest_version: latestVersion };
});


function handleInstall() {
  if (!pkg.value) return;
  logger.debug("PackageDetailView", `install clicked: ${pkg.value.id}`);
  installMutation.mutate(pkg.value.id);
}

function handleUpdate() {
  if (!pkg.value) return;
  logger.debug("PackageDetailView", `update clicked: ${pkg.value.id}`);
  updateMutation.mutate(pkg.value.id);
}

function handleBackup() {
  logger.debug("PackageDetailView", `backup clicked: ${pkg.value?.id}`);
  showBackupConfirm.value = true;
}

function confirmBackup() {
  if (!pkg.value) return;
  const paths = pkg.value.backup?.config_paths ?? [];
  logger.debug("PackageDetailView", `backup confirmed: ${pkg.value.id}, paths: ${paths.join(", ")}`);
  backupMutation.mutate(paths);
}
</script>

<template>
  <div class="detail-view">
    <div class="detail-breadcrumb">
      <button
        class="breadcrumb-back"
        @click="router.push('/catalog')"
      >
        <i class="pi pi-arrow-left" />
        Catalog
      </button>
      <span class="breadcrumb-sep">/</span>
      <span class="breadcrumb-current">{{ pkg?.name ?? id }}</span>
    </div>

    <EmptyState
      v-if="!isLoading && !pkg"
      icon="pi-question-circle"
      message="Package not found."
      action-label="Back to Catalog"
      @action="router.push('/catalog')"
    />

    <template v-else-if="pkg">
      <DetailHero
        :pkg="pkg"
        :actions-disabled="actionsDisabled"
        @install="handleInstall"
        @update="handleUpdate"
        @backup="handleBackup"
      />

      <div class="detail-content">
        <Tabs
          v-if="tabsReady"
          value="0"
        >
          <TabList>
            <Tab value="0">
              Overview
            </Tab>
            <Tab value="1">
              Versions
            </Tab>
            <Tab
              v-if="pkg.backup?.config_paths?.length"
              value="2"
            >
              Backup
            </Tab>
            <Tab value="3">
              Technical
            </Tab>
          </TabList>
          <TabPanels>
            <TabPanel value="0">
              <OverviewTab :pkg="pkg" />
            </TabPanel>
            <TabPanel value="1">
              <VersionsTab
                :versions="(versions as VersionEntry[] | undefined) ?? []"
                :installed-version="pkg.installed_version ?? null"
                @install="handleInstall"
              />
            </TabPanel>
            <TabPanel value="2">
              <BackupTab
                :package-id="pkg.id"
                :config-paths="pkg.backup?.config_paths"
                @backup="handleBackup"
              />
            </TabPanel>
            <TabPanel value="3">
              <TechnicalTab :pkg="pkg" />
            </TabPanel>
          </TabPanels>
        </Tabs>
      </div>
    </template>

    <ConfirmDialog
      v-model:visible="showBackupConfirm"
      title="Backup Now"
      :message="`Create a backup of ${pkg?.name ?? 'this package'}?`"
      icon="pi-database"
      confirm-label="Backup"
      @confirm="confirmBackup"
    />
  </div>
</template>

<style scoped>
.detail-view {
  display: flex;
  flex-direction: column;
}

.detail-breadcrumb {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 32px;
  font-size: 13px;
  border-bottom: 1px solid var(--p-surface-800);
}

.breadcrumb-back {
  background: none;
  border: none;
  color: var(--p-primary-400);
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  padding: 0;
}

.breadcrumb-back:hover {
  text-decoration: underline;
}

.breadcrumb-sep {
  color: var(--p-surface-500);
}

.breadcrumb-current {
  color: var(--p-surface-300);
}

.detail-content {
  padding: 0 32px 24px;
  max-width: 1200px;
}
</style>
