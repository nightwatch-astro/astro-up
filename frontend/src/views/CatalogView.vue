<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import InputText from "primevue/inputtext";
import IconField from "primevue/iconfield";
import InputIcon from "primevue/inputicon";
import ProgressSpinner from "primevue/progressspinner";
import Button from "primevue/button";
import { useRouter } from "vue-router";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import CategoryChips from "../components/catalog/CategoryChips.vue";
import PackageGrid from "../components/catalog/PackageGrid.vue";
import EmptyState from "../components/shared/EmptyState.vue";
import { useSearch } from "../composables/useSearch";
import { useSoftwareList, useSyncCatalog } from "../composables/useInvoke";
import type { PackageWithStatus, Category } from "../types/package";

const router = useRouter();
const { data: software, isLoading, isError, refetch } = useSoftwareList(() => "all");
const syncCatalog = useSyncCatalog();

const catalogStatus = ref<"unknown" | "syncing" | "ready" | "error">("unknown");
let unlistenStatus: UnlistenFn | null = null;

onMounted(async () => {
  unlistenStatus = await listen<string>("catalog-status", (event) => {
    catalogStatus.value = event.payload as typeof catalogStatus.value;
    if (event.payload === "ready") {
      refetch();
    }
  });
});

onUnmounted(() => {
  if (unlistenStatus) unlistenStatus();
});

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

const isSyncing = computed(() => catalogStatus.value === "syncing" || syncCatalog.isPending.value);

function handleRetry() {
  syncCatalog.mutate();
}

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

    <!-- Syncing state -->
    <div
      v-if="isSyncing && !software"
      class="sync-state"
    >
      <ProgressSpinner
        style="width: 40px; height: 40px"
        stroke-width="4"
      />
      <span class="sync-text">Syncing catalog...</span>
    </div>

    <template v-else>
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
        @action="handleRetry"
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
    </template>
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

.sync-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 48px 0;
}

.sync-text {
  font-size: 14px;
  color: var(--p-surface-400);
}
</style>
