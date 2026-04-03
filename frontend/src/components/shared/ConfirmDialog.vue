<script setup lang="ts">
import Dialog from "primevue/dialog";
import Button from "primevue/button";

withDefaults(defineProps<{
  visible: boolean;
  title: string;
  message: string;
  icon?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  severity?: "primary" | "danger" | "warn";
}>(), {
  icon: "pi-question-circle",
  confirmLabel: "Confirm",
  cancelLabel: "Cancel",
  severity: "primary",
});

const emit = defineEmits<{
  "update:visible": [value: boolean];
  confirm: [];
  cancel: [];
}>();

function handleConfirm() {
  emit("confirm");
  emit("update:visible", false);
}

function handleCancel() {
  emit("cancel");
  emit("update:visible", false);
}
</script>

<template>
  <Dialog
    :visible="visible"
    :header="title"
    modal
    :closable="true"
    :style="{ width: '480px' }"
    @update:visible="emit('update:visible', $event)"
  >
    <div class="confirm-body">
      <i
        v-if="icon"
        :class="['pi', icon, 'confirm-icon']"
      />
      <div class="confirm-content">
        <p class="confirm-message">
          {{ message }}
        </p>
        <slot />
      </div>
    </div>

    <template #footer>
      <Button
        :label="cancelLabel"
        text
        severity="secondary"
        @click="handleCancel"
      />
      <Button
        :label="confirmLabel"
        :severity="severity === 'primary' ? undefined : severity"
        @click="handleConfirm"
      />
    </template>
  </Dialog>
</template>

<style scoped>
.confirm-body {
  display: flex;
  gap: 16px;
  padding: 8px 0;
}

.confirm-icon {
  font-size: 24px;
  color: var(--p-primary-400);
  flex-shrink: 0;
  margin-top: 2px;
}

.confirm-content {
  flex: 1;
}

.confirm-message {
  margin: 0;
  color: var(--p-surface-200);
  font-size: 14px;
  line-height: 1.5;
}
</style>
