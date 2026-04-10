<script setup lang="ts">
import Button from "primevue/button";
import { FEATURE_BACKUP } from "../../features";
import PackageIcon from "../shared/PackageIcon.vue";
import type { PackageWithStatus } from "../../types/package";

defineProps<{
  pkg: PackageWithStatus;
  actionsDisabled?: boolean;
}>();

defineEmits<{
  update: [];
  backup: [];
  detail: [];
}>();
</script>

<template>
  <div
    class="inst-row"
    @click="$emit('detail')"
  >
    <PackageIcon
      :icon-base64="pkg.icon_base64"
      :category="pkg.category"
      size="md"
    />
    <div class="inst-info">
      <div class="inst-name">
        {{ pkg.name }}
      </div>
      <div class="inst-sub">
        {{ pkg.category }} &middot; {{ pkg.detection?.type === 'Installed' || pkg.detection?.type === 'InstalledUnknownVersion' ? pkg.detection.method : pkg.software_type }}
      </div>
    </div>
    <div class="inst-version">
      <template v-if="pkg.update_available">
        <span class="ver-update">{{ pkg.installed_version }} &rarr; {{ pkg.latest_version }}</span>
      </template>
      <template v-else>
        <span class="ver-ok">
          {{ pkg.installed_version }}
          <i class="pi pi-check" />
        </span>
      </template>
    </div>
    <div
      class="inst-actions"
      @click.stop
    >
      <Button
        v-if="FEATURE_BACKUP && pkg.backup?.config_paths?.length"
        icon="pi pi-database"
        label="Backup Now"
        text
        size="small"
        severity="secondary"
        class="action-btn"
        :disabled="actionsDisabled"
        @click="$emit('backup')"
      />
      <Button
        v-if="pkg.update_available"
        label="Update"
        severity="warn"
        size="small"
        class="action-btn"
        :disabled="actionsDisabled"
        @click="$emit('update')"
      />
    </div>
  </div>
</template>

<style scoped>
.inst-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--p-surface-700);
  cursor: pointer;
  transition: background 0.1s;
}

.inst-row:last-child {
  border-bottom: none;
}

.inst-row:hover {
  background: rgba(255, 255, 255, 0.02);
}

.sw-icon {
  width: 38px;
  height: 38px;
  border-radius: 8px;
  background: var(--p-surface-700);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 15px;
  color: var(--p-primary-400);
  flex-shrink: 0;
}

.inst-info {
  flex: 1;
  min-width: 0;
}

.inst-name {
  font-size: 14px;
  color: var(--p-surface-0);
  font-weight: 500;
}

.inst-sub {
  font-size: 11px;
  color: var(--p-surface-500);
  margin-top: 1px;
}

.inst-version {
  font-size: 12px;
  min-width: 120px;
  text-align: right;
  margin-right: 8px;
  flex-shrink: 0;
}

.ver-update {
  color: var(--p-yellow-400);
}

.ver-ok {
  color: var(--p-green-400);
}

.ver-ok i {
  font-size: 10px;
  margin-left: 4px;
}

.inst-actions {
  display: flex;
  gap: 6px;
  align-items: center;
  flex-shrink: 0;
}

.action-btn {
  font-size: 11px;
}
</style>
