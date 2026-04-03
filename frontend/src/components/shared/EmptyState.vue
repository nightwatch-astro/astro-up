<script setup lang="ts">
import Button from "primevue/button";

defineProps<{
  icon?: string;
  message: string;
  actionLabel?: string;
}>();

defineEmits<{
  action: [];
}>();
</script>

<template>
  <div class="empty-state">
    <i
      v-if="icon"
      :class="['pi', icon, 'empty-icon']"
    />
    <p class="empty-message">
      {{ message }}
    </p>
    <Button
      v-if="actionLabel"
      :label="actionLabel"
      severity="secondary"
      outlined
      size="small"
      @click="$emit('action')"
    />
  </div>
</template>

<style scoped>
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  text-align: center;
  gap: 12px;
}

.empty-icon {
  font-size: 48px;
  color: var(--p-surface-500);
}

.empty-message {
  margin: 0;
  color: var(--p-surface-400);
  font-size: 14px;
  max-width: 320px;
}
</style>
