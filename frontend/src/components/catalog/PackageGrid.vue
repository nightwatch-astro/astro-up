<script setup lang="ts">
import Skeleton from "primevue/skeleton";
import PackageCard from "./PackageCard.vue";
import type { PackageWithStatus } from "../../types/package";

defineProps<{
  packages: PackageWithStatus[];
  loading?: boolean;
}>();

defineEmits<{
  select: [id: string];
}>();
</script>

<template>
  <!-- Loading skeleton -->
  <div
    v-if="loading"
    class="package-grid"
  >
    <div
      v-for="i in 8"
      :key="i"
      class="skeleton-card"
    >
      <div class="skeleton-header">
        <Skeleton
          shape="square"
          width="36px"
          height="36px"
          border-radius="8px"
        />
        <div class="skeleton-meta">
          <Skeleton
            width="120px"
            height="14px"
          />
          <Skeleton
            width="80px"
            height="12px"
          />
        </div>
      </div>
      <Skeleton
        width="100%"
        height="12px"
      />
      <Skeleton
        width="70%"
        height="12px"
      />
    </div>
  </div>

  <!-- Package cards -->
  <div
    v-else
    class="package-grid"
  >
    <PackageCard
      v-for="pkg in packages"
      :key="pkg.id"
      :pkg="pkg"
      @click="$emit('select', pkg.id)"
    />
  </div>
</template>

<style scoped>
.package-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 14px;
}

.skeleton-card {
  background: var(--p-surface-800);
  border: 1px solid var(--p-surface-700);
  border-radius: 10px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.skeleton-header {
  display: flex;
  gap: 10px;
  align-items: flex-start;
}

.skeleton-meta {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
</style>
