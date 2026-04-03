<script setup lang="ts">
import Tag from "primevue/tag";
import type { PackageWithStatus } from "../../types/package";

defineProps<{
  pkg: PackageWithStatus;
}>();

defineEmits<{
  click: [];
}>();

function statusLabel(pkg: PackageWithStatus): string | null {
  if (pkg.update_available) return "Update";
  if (pkg.installed_version) return "Installed";
  return null;
}

function statusSeverity(pkg: PackageWithStatus): "warn" | "success" | undefined {
  if (pkg.update_available) return "warn";
  if (pkg.installed_version) return "success";
  return undefined;
}
</script>

<template>
  <div
    class="package-card"
    tabindex="0"
    role="button"
    @click="$emit('click')"
    @keydown.enter="$emit('click')"
  >
    <div class="card-header">
      <div class="card-icon">
        <i class="pi pi-box" />
      </div>
      <div class="card-meta">
        <h3 class="card-name">
          {{ pkg.name }}
        </h3>
        <span
          v-if="pkg.publisher"
          class="card-publisher"
        >
          {{ pkg.publisher }}
        </span>
      </div>
      <Tag
        v-if="statusLabel(pkg)"
        :value="statusLabel(pkg)!"
        :severity="statusSeverity(pkg)"
        class="card-status"
      />
    </div>
    <p
      v-if="pkg.description"
      class="card-description"
    >
      {{ pkg.description }}
    </p>
    <Tag
      :value="pkg.category"
      severity="secondary"
      class="card-category"
    />
  </div>
</template>

<style scoped>
.package-card {
  background: var(--p-surface-800);
  border: 1px solid var(--p-surface-700);
  border-radius: 10px;
  padding: 16px;
  cursor: pointer;
  transition: all 0.15s;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.package-card:hover {
  border-color: var(--p-surface-500);
  background: var(--p-surface-750, var(--p-surface-800));
}

.package-card:focus-visible {
  outline: 2px solid var(--p-primary-400);
  outline-offset: 2px;
}

.card-header {
  display: flex;
  align-items: flex-start;
  gap: 10px;
}

.card-icon {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  background: var(--p-surface-700);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: var(--p-surface-300);
  font-size: 16px;
}

.card-meta {
  flex: 1;
  min-width: 0;
}

.card-name {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--p-surface-0);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-publisher {
  font-size: 12px;
  color: var(--p-surface-400);
}

.card-description {
  margin: 0;
  font-size: 13px;
  color: var(--p-surface-300);
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.card-category {
  align-self: flex-start;
}
</style>
