<script setup lang="ts">
import PackageIcon from "../shared/PackageIcon.vue";
import type { PackageWithStatus, Category } from "../../types/package";

defineProps<{
  pkg: PackageWithStatus;
}>();

defineEmits<{
  click: [];
}>();

const categoryClass: Record<Category, string> = {
  Capture: "tag-capture",
  Guiding: "tag-guiding",
  Platesolving: "tag-platesolving",
  Equipment: "tag-equipment",
  Focusing: "tag-focusing",
  Planetarium: "tag-planetarium",
  Viewers: "tag-viewers",
  Prerequisites: "tag-misc",
  Usb: "tag-misc",
  Driver: "tag-driver",
};
</script>

<template>
  <div
    class="sw-card card"
    tabindex="0"
    role="button"
    @click="$emit('click')"
    @keydown.enter="$emit('click')"
  >
    <div class="sw-card-hdr">
      <PackageIcon
        :icon-base64="pkg.icon_base64"
        :category="pkg.category"
        size="lg"
      />
      <div>
        <div class="sw-name">
          {{ pkg.name }}
        </div>
        <div class="sw-pub">
          {{ pkg.publisher }}
        </div>
      </div>
    </div>
    <div
      v-if="pkg.description"
      class="sw-desc"
    >
      {{ pkg.description }}
    </div>
    <div class="sw-foot">
      <span :class="['tag', categoryClass[pkg.category] ?? 'tag-misc']">
        {{ pkg.category }}
      </span>
      <div class="sw-status">
        <template v-if="pkg.update_available">
          <span class="status-dot update" />
          <span class="status-update">{{ pkg.installed_version === "0.0.0" ? "unknown" : pkg.installed_version }} &rarr; {{ pkg.latest_version }}</span>
        </template>
        <template v-else-if="pkg.installed_version">
          <span class="status-dot installed" />
          <span class="status-installed">{{ pkg.installed_version === "0.0.0" ? "unknown" : pkg.installed_version }}</span>
        </template>
        <template v-else>
          <span class="status-dot none" />
          <span class="status-none">Not installed</span>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.sw-card {
  padding: 18px;
  cursor: pointer;
  transition: all 0.15s;
  display: flex;
  flex-direction: column;
}

.sw-card:hover {
  border-color: var(--p-surface-500);
  transform: translateY(-1px);
}

.sw-card:focus-visible {
  outline: 2px solid var(--p-primary-400);
  outline-offset: 2px;
}

.sw-card-hdr {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  margin-bottom: 10px;
}

.sw-icon {
  width: 44px;
  height: 44px;
  border-radius: 10px;
  background: var(--p-surface-700);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  color: var(--p-primary-400);
  flex-shrink: 0;
}

.sw-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--p-surface-0);
}

.sw-pub {
  font-size: 11px;
  color: var(--p-surface-500);
  margin-top: 1px;
}

.sw-desc {
  font-size: 12px;
  color: var(--p-surface-400);
  line-height: 1.5;
  margin-bottom: 12px;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.sw-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: auto;
}

/* Category tags */
.tag {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.3px;
  font-weight: 500;
}

.tag-capture {
  background: color-mix(in srgb, var(--p-blue-500) 20%, transparent);
  color: var(--p-blue-400);
}

.tag-guiding {
  background: color-mix(in srgb, var(--p-green-500) 15%, transparent);
  color: var(--p-green-400);
}

.tag-platesolving {
  background: color-mix(in srgb, var(--p-purple-500) 20%, transparent);
  color: var(--p-purple-400);
}

.tag-driver {
  background: color-mix(in srgb, var(--p-yellow-500) 15%, transparent);
  color: var(--p-yellow-400);
}

.tag-focusing {
  background: color-mix(in srgb, var(--p-red-500) 15%, transparent);
  color: var(--p-red-300);
}

.tag-planetarium {
  background: color-mix(in srgb, var(--p-teal-500) 15%, transparent);
  color: var(--p-teal-400);
}

.tag-viewers {
  background: color-mix(in srgb, var(--p-surface-500) 15%, transparent);
  color: var(--p-surface-400);
}

.tag-equipment {
  background: color-mix(in srgb, var(--p-surface-500) 15%, transparent);
  color: var(--p-surface-400);
}

.tag-misc {
  background: color-mix(in srgb, var(--p-surface-500) 15%, transparent);
  color: var(--p-surface-400);
}

/* Status */
.sw-status {
  font-size: 11px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.status-dot {
  display: inline-block;
  width: 7px;
  height: 7px;
  border-radius: 50%;
}

.status-dot.installed {
  background: var(--p-green-400);
}

.status-dot.update {
  background: var(--p-yellow-400);
}

.status-dot.none {
  background: var(--p-surface-600);
}

.status-installed {
  color: var(--p-green-400);
}

.status-update {
  color: var(--p-yellow-400);
}

.status-none {
  color: var(--p-surface-600);
}
</style>
