<script setup lang="ts">
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Button from "primevue/button";
import Tag from "primevue/tag";
import type { VersionEntry } from "../../types/package";

defineProps<{
  versions: VersionEntry[];
  installedVersion: string | null;
  actionsDisabled?: boolean;
}>();

defineEmits<{
  install: [version: string];
}>();

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}
</script>

<template>
  <DataTable
    :value="versions"
    striped-rows
    size="small"
    class="versions-table"
  >
    <Column
      field="version"
      header="Version"
    >
      <template #body="{ data }">
        <span class="version-text">
          {{ (data as VersionEntry).version }}
        </span>
        <Tag
          v-if="(data as VersionEntry).version === installedVersion"
          value="Installed"
          severity="success"
          class="version-tag"
        />
      </template>
    </Column>
    <Column header="Discovered">
      <template #body="{ data }">
        {{ formatDate((data as VersionEntry).discovered_at) }}
      </template>
    </Column>
    <Column header="Pre-release">
      <template #body="{ data }">
        <Tag
          v-if="(data as VersionEntry).pre_release"
          value="Pre-release"
          severity="warn"
        />
      </template>
    </Column>
    <Column
      header=""
      style="width: 100px"
    >
      <template #body="{ data }">
        <Button
          v-if="(data as VersionEntry).version !== installedVersion"
          label="Install"
          size="small"
          outlined
          :disabled="actionsDisabled"
          @click="$emit('install', (data as VersionEntry).version)"
        />
      </template>
    </Column>
  </DataTable>
</template>

<style scoped>
.version-text {
  font-weight: 500;
}

.version-tag {
  margin-left: 8px;
}
</style>
