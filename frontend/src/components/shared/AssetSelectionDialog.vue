<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import Dialog from "primevue/dialog";
import Button from "primevue/button";
import RadioButton from "primevue/radiobutton";

interface AssetOption {
  index: number;
  name: string;
  size: number;
}

interface AssetSelectionRequest {
  package_name: string;
  assets: AssetOption[];
}

const visible = ref(false);
const packageName = ref("");
const assets = ref<AssetOption[]>([]);
const selectedIndex = ref<number>(0);

let unlisten: UnlistenFn | null = null;

function formatSize(bytes: number): string {
  if (bytes === 0) return "unknown size";
  const units = ["B", "KB", "MB", "GB"];
  let i = 0;
  let size = bytes;
  while (size >= 1024 && i < units.length - 1) {
    size /= 1024;
    i++;
  }
  return `${size.toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
}

const formattedAssets = computed(() =>
  assets.value.map((a) => ({
    ...a,
    formattedSize: formatSize(a.size),
  })),
);

async function handleConfirm() {
  visible.value = false;
  await invoke("resolve_asset_selection", {
    response: { index: selectedIndex.value },
  });
}

async function handleCancel() {
  visible.value = false;
  await invoke("resolve_asset_selection", {
    response: { index: null },
  });
}

onMounted(async () => {
  try {
    unlisten = await listen<AssetSelectionRequest>(
      "asset-selection-required",
      (event) => {
        packageName.value = event.payload.package_name;
        assets.value = event.payload.assets;
        selectedIndex.value = 0;
        visible.value = true;
      },
    );
  } catch {
    // Not running inside Tauri
  }
});

onUnmounted(() => {
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
});
</script>

<template>
  <Dialog
    :visible="visible"
    header="Select Download"
    modal
    :closable="false"
    :style="{ width: '520px' }"
    @update:visible="visible = $event"
  >
    <div class="asset-dialog-body">
      <p class="asset-dialog-message">
        <strong>{{ packageName }}</strong> has multiple download options.
        Select which file to install:
      </p>

      <div class="asset-options">
        <label
          v-for="asset in formattedAssets"
          :key="asset.index"
          class="asset-option"
          :class="{ selected: selectedIndex === asset.index }"
        >
          <RadioButton
            v-model="selectedIndex"
            :value="asset.index"
            name="asset"
          />
          <div class="asset-info">
            <span class="asset-name">{{ asset.name }}</span>
            <span class="asset-size">{{ asset.formattedSize }}</span>
          </div>
        </label>
      </div>
    </div>

    <template #footer>
      <Button
        label="Cancel"
        text
        severity="secondary"
        @click="handleCancel"
      />
      <Button
        label="Download"
        @click="handleConfirm"
      />
    </template>
  </Dialog>
</template>

<style scoped>
.asset-dialog-body {
  padding: 4px 0;
}

.asset-dialog-message {
  margin: 0 0 16px;
  color: var(--p-surface-200);
  font-size: 14px;
  line-height: 1.5;
}

.asset-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.asset-option {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 6px;
  border: 1px solid var(--p-surface-700);
  cursor: pointer;
  transition: border-color 0.15s, background-color 0.15s;
}

.asset-option:hover {
  border-color: var(--p-surface-500);
  background-color: var(--p-surface-800);
}

.asset-option.selected {
  border-color: var(--p-primary-400);
  background-color: color-mix(in srgb, var(--p-primary-400) 8%, transparent);
}

.asset-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.asset-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--p-surface-100);
  word-break: break-all;
}

.asset-size {
  font-size: 12px;
  color: var(--p-surface-400);
}
</style>
