<script setup lang="ts">
import Button from "primevue/button";
import Tag from "primevue/tag";
import type { PackageWithStatus } from "../../types/package";

defineProps<{
  pkg: PackageWithStatus;
}>();

defineEmits<{
  update: [];
  backup: [];
  detail: [];
}>();
</script>

<template>
  <div
    class="package-row"
    @click="$emit('detail')"
  >
    <div class="row-icon">
      <i class="pi pi-box" />
    </div>
    <div class="row-info">
      <span class="row-name">{{ pkg.name }}</span>
      <span class="row-category">{{ pkg.category }}</span>
    </div>
    <div class="row-version">
      <template v-if="pkg.update_available">
        <span class="version-current">{{ pkg.installed_version }}</span>
        <i class="pi pi-arrow-right version-arrow" />
        <span class="version-latest">{{ pkg.latest_version }}</span>
      </template>
      <template v-else>
        <Tag
          :value="pkg.installed_version ?? 'Unknown'"
          severity="success"
        />
        <i class="pi pi-check version-check" />
      </template>
    </div>
    <div
      class="row-actions"
      @click.stop
    >
      <Button
        v-if="pkg.installed_version"
        icon="pi pi-database"
        text
        rounded
        size="small"
        severity="secondary"
        title="Backup Now"
        @click="$emit('backup')"
      />
      <Button
        v-if="pkg.update_available"
        label="Update"
        icon="pi pi-arrow-up"
        size="small"
        severity="warn"
        @click="$emit('update')"
      />
    </div>
  </div>
</template>

<style scoped>
.package-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s;
}

.package-row:hover {
  background: var(--p-surface-800);
}

.row-icon {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  background: var(--p-surface-700);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: var(--p-surface-300);
  font-size: 14px;
}

.row-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.row-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--p-surface-100);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.row-category {
  font-size: 12px;
  color: var(--p-surface-400);
}

.row-version {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  flex-shrink: 0;
}

.version-current {
  color: var(--p-surface-400);
}

.version-arrow {
  font-size: 10px;
  color: var(--p-yellow-400);
}

.version-latest {
  color: var(--p-yellow-400);
  font-weight: 500;
}

.version-check {
  color: var(--p-green-400);
  font-size: 12px;
}

.row-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}
</style>
