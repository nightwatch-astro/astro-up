<script setup lang="ts">
import Button from "primevue/button";
import type { PackageWithStatus } from "../../types/package";

defineProps<{
  pkg: PackageWithStatus;
}>();

defineEmits<{
  install: [];
  update: [];
  backup: [];
}>();
</script>

<template>
  <div class="detail-hero">
    <div class="hero-icon">
      <i class="pi pi-box" />
    </div>
    <div class="hero-info">
      <h1 class="hero-name">
        {{ pkg.name }}
      </h1>
      <div class="hero-meta">
        <span
          v-if="pkg.publisher"
          class="hero-publisher"
        >
          {{ pkg.publisher }}
        </span>
        <a
          v-if="pkg.homepage"
          :href="pkg.homepage"
          target="_blank"
          class="hero-link"
        >
          <i class="pi pi-external-link" />
          Homepage
        </a>
      </div>
      <p
        v-if="pkg.description"
        class="hero-description"
      >
        {{ pkg.description }}
      </p>
    </div>
    <div class="hero-actions">
      <Button
        v-if="pkg.update_available"
        label="Update"
        icon="pi pi-arrow-up"
        severity="warn"
        @click="$emit('update')"
      />
      <Button
        v-else-if="pkg.installed_version"
        label="Installed"
        icon="pi pi-check"
        severity="success"
        outlined
        disabled
      />
      <Button
        v-else
        label="Install"
        icon="pi pi-download"
        @click="$emit('install')"
      />
      <Button
        v-if="pkg.installed_version"
        label="Backup Now"
        icon="pi pi-database"
        severity="secondary"
        outlined
        @click="$emit('backup')"
      />
      <Button
        v-if="pkg.homepage"
        icon="pi pi-external-link"
        severity="secondary"
        text
        rounded
        :as="'a'"
        :href="pkg.homepage"
        target="_blank"
      />
    </div>
  </div>
</template>

<style scoped>
.detail-hero {
  display: flex;
  gap: 20px;
  padding: 24px;
  border-bottom: 1px solid var(--p-surface-700);
}

.hero-icon {
  width: 64px;
  height: 64px;
  border-radius: 12px;
  background: var(--p-surface-700);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: var(--p-surface-300);
  font-size: 28px;
}

.hero-info {
  flex: 1;
  min-width: 0;
}

.hero-name {
  margin: 0;
  font-size: 22px;
  font-weight: 700;
  color: var(--p-surface-0);
}

.hero-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 4px;
}

.hero-publisher {
  font-size: 13px;
  color: var(--p-surface-400);
}

.hero-link {
  font-size: 13px;
  color: var(--p-primary-400);
  text-decoration: none;
  display: flex;
  align-items: center;
  gap: 4px;
}

.hero-link:hover {
  text-decoration: underline;
}

.hero-description {
  margin: 8px 0 0;
  font-size: 14px;
  color: var(--p-surface-300);
  line-height: 1.5;
}

.hero-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex-shrink: 0;
}
</style>
