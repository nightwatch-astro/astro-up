<script setup lang="ts">
import type { Category } from "../../types/package";

defineProps<{
  active: Category | null;
  categories: Category[];
}>();

const emit = defineEmits<{
  select: [category: Category | null];
}>();

const categoryColors: Record<Category, string> = {
  Capture: "var(--p-blue-400)",
  Guiding: "var(--p-green-400)",
  Platesolving: "var(--p-purple-400)",
  Equipment: "var(--p-orange-400)",
  Focusing: "var(--p-cyan-400)",
  Planetarium: "var(--p-indigo-400)",
  Viewers: "var(--p-teal-400)",
  Prerequisites: "var(--p-surface-400)",
  Usb: "var(--p-yellow-400)",
  Driver: "var(--p-pink-400)",
};
</script>

<template>
  <div class="category-chips">
    <button
      class="chip"
      :class="{ active: active === null }"
      @click="emit('select', null)"
    >
      All
    </button>
    <button
      v-for="cat in categories"
      :key="cat"
      class="chip"
      :class="{ active: active === cat }"
      :style="active === cat ? { '--chip-color': categoryColors[cat] } : {}"
      @click="emit('select', active === cat ? null : cat)"
    >
      <span
        class="chip-dot"
        :style="{ background: categoryColors[cat] }"
      />
      {{ cat }}
    </button>
  </div>
</template>

<style scoped>
.category-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.chip {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border-radius: 16px;
  border: 1px solid var(--p-surface-700);
  background: transparent;
  color: var(--p-surface-300);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.chip:hover {
  border-color: var(--p-surface-500);
  color: var(--p-surface-100);
}

.chip.active {
  background: color-mix(in srgb, var(--chip-color, var(--p-primary-500)) 20%, transparent);
  border-color: var(--chip-color, var(--p-primary-500));
  color: var(--chip-color, var(--p-primary-400));
}

.chip-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
</style>
