<script setup lang="ts">
import type { Category } from "../../types/package";

defineProps<{
  iconBase64?: string | null;
  category?: Category;
  size?: "sm" | "md" | "lg" | "xl";
}>();

const categoryIcons: Record<Category, string> = {
  Capture: "pi-camera",
  Guiding: "pi-compass",
  Platesolving: "pi-star",
  Equipment: "pi-server",
  Focusing: "pi-bullseye",
  Planetarium: "pi-globe",
  Viewers: "pi-images",
  Prerequisites: "pi-cog",
  Usb: "pi-bolt",
  Driver: "pi-bolt",
};
</script>

<template>
  <div :class="['pkg-icon', `pkg-icon-${size ?? 'md'}`]">
    <img
      v-if="iconBase64"
      :src="'data:image/png;base64,' + iconBase64"
      class="pkg-icon-img"
    >
    <i
      v-else
      :class="['pi', categoryIcons[category ?? 'Capture'] ?? 'pi-box']"
    />
  </div>
</template>

<style scoped>
.pkg-icon {
  border-radius: 10px;
  background: var(--p-surface-700);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: var(--p-primary-400);
  overflow: hidden;
}

.pkg-icon-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.pkg-icon-sm {
  width: 30px;
  height: 30px;
  font-size: 12px;
  border-radius: 8px;
}

.pkg-icon-md {
  width: 38px;
  height: 38px;
  font-size: 15px;
  border-radius: 8px;
}

.pkg-icon-lg {
  width: 44px;
  height: 44px;
  font-size: 18px;
  border-radius: 10px;
}

.pkg-icon-xl {
  width: 64px;
  height: 64px;
  font-size: 28px;
  border-radius: 12px;
}
</style>
