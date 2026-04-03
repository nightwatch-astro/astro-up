<script setup lang="ts">
import { computed } from "vue";
import InputText from "primevue/inputtext";
import IconField from "primevue/iconfield";
import InputIcon from "primevue/inputicon";
import { useRouter } from "vue-router";
import CategoryChips from "../components/catalog/CategoryChips.vue";
import PackageGrid from "../components/catalog/PackageGrid.vue";
import EmptyState from "../components/shared/EmptyState.vue";
import { useSearch } from "../composables/useSearch";
import { useSoftwareList } from "../composables/useInvoke";
import type { PackageWithStatus, Category } from "../types/package";

const router = useRouter();
const { data: software, isLoading, isError, refetch } = useSoftwareList(() => "all");

const packages = computed(() => (software.value ?? []) as PackageWithStatus[]);

const { searchText, activeCategory, filtered, setCategory } = useSearch(packages);

const categories = computed<Category[]>(() => {
  if (!packages.value.length) return [];
  const seen = new Set<Category>();
  for (const pkg of packages.value) {
    seen.add(pkg.category);
  }
  return [...seen].sort();
});

function navigateToDetail(id: string) {
  router.push({ name: "package-detail", params: { id } });
}
</script>

<template>
  <div class="page-view">
    <div class="page-hdr">
      <h2>Software Catalog</h2>
      <p>Browse and install astrophotography software</p>
    </div>

    <div class="catalog-bar">
      <IconField class="catalog-search">
        <InputIcon class="pi pi-search" />
        <InputText
          v-model="searchText"
          placeholder="Search packages..."
        />
      </IconField>
    </div>

    <CategoryChips
      v-if="categories.length > 0"
      :active="activeCategory"
      :categories="categories"
      @select="setCategory"
    />

    <EmptyState
      v-if="isError"
      icon="pi-exclamation-triangle"
      message="Failed to load catalog. Check your connection and try again."
      action-label="Retry"
      @action="refetch()"
    />

    <PackageGrid
      v-else-if="filtered.length > 0 || isLoading"
      :packages="(filtered as PackageWithStatus[])"
      :loading="isLoading"
      @select="navigateToDetail"
    />

    <EmptyState
      v-else
      icon="pi-search"
      message="No packages match your search."
      action-label="Clear filters"
      @action="searchText = ''; setCategory(null)"
    />
  </div>
</template>

<style scoped>
.catalog-bar {
  display: flex;
  align-items: center;
  gap: 12px;
}

.catalog-search {
  flex: 1;
}
</style>
