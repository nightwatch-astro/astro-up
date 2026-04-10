<script setup lang="ts">
import { computed } from "vue";
import Badge from "primevue/badge";
import { FEATURE_BACKUP } from "../../features";
import { useSoftwareList } from "../../composables/useInvoke";
import type { PackageWithStatus } from "../../types/package";

const version = __APP_VERSION__;
const { data: software } = useSoftwareList(() => "all");
const updateCount = computed(() =>
  ((software.value ?? []) as PackageWithStatus[]).filter((p) => p.update_available).length,
);

const allNavItems = [
  { to: "/", label: "Dashboard", icon: "pi-home", exact: true },
  { to: "/catalog", label: "Catalog", icon: "pi-th-large" },
  { to: "/installed", label: "Installed", icon: "pi-check-circle" },
  { to: "/backup", label: "Backup", icon: "pi-database", feature: FEATURE_BACKUP },
  { to: "/settings", label: "Settings", icon: "pi-cog" },
];
const navItems = allNavItems.filter((item) => item.feature !== false);
</script>

<template>
  <nav class="app-sidebar">
    <div class="sidebar-header">
      <h1>Astro-Up</h1>
      <span class="version">v{{ version }}</span>
    </div>
    <div class="sidebar-nav">
      <router-link
        v-for="item in navItems"
        :key="item.to"
        :to="item.to"
        class="nav-item"
        :exact-active-class="item.exact ? 'active' : undefined"
        :active-class="item.exact ? undefined : 'active'"
      >
        <i :class="['pi', item.icon]" />
        {{ item.label }}
        <Badge
          v-if="item.to === '/installed' && updateCount > 0"
          :value="updateCount"
          severity="warn"
          class="ml-auto"
        />
      </router-link>
    </div>
    <div class="sidebar-footer">
      v{{ version }}
    </div>
  </nav>
</template>

<style scoped>
.app-sidebar {
  width: 220px;
  background: var(--p-surface-900);
  border-right: 1px solid var(--p-surface-700);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.sidebar-header {
  padding: 20px 16px;
  border-bottom: 1px solid var(--p-surface-700);
}

.sidebar-header h1 {
  font-size: 18px;
  font-weight: 700;
  color: var(--p-surface-0);
  margin: 0;
}

.sidebar-header .version {
  font-size: 11px;
  color: var(--p-surface-400);
}

.sidebar-nav {
  flex: 1;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 8px;
  color: var(--p-surface-300);
  font-size: 14px;
  text-decoration: none;
  transition: all 0.15s;
}

.nav-item:hover {
  background: var(--p-surface-800);
  color: var(--p-surface-0);
}

.nav-item.active {
  background: color-mix(in srgb, var(--p-primary-500) 20%, transparent);
  color: var(--p-primary-400);
}

.nav-item i {
  font-size: 16px;
  width: 20px;
  text-align: center;
}

.ml-auto {
  margin-left: auto;
}

.sidebar-footer {
  padding: 12px 16px;
  border-top: 1px solid var(--p-surface-700);
  font-size: 11px;
  color: var(--p-surface-500);
  line-height: 1.6;
}
</style>
