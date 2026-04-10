<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { open as openUrl } from "@tauri-apps/plugin-shell";
import Dialog from "primevue/dialog";
import Button from "primevue/button";
import { useToast } from "primevue/usetoast";
import { logger } from "../../utils/logger";

defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  "update:visible": [value: boolean];
}>();

const toast = useToast();

async function handleFeedback() {
  try {
    await invoke("complete_survey");
    await openUrl("https://tally.so/r/lb7dd5");
  } catch (e) {
    logger.error("survey", `Failed to complete survey: ${e}`);
    toast.add({ severity: "error", summary: "Error", detail: "Could not record survey response", life: 3000 });
  }
  emit("update:visible", false);
}

async function handleNotNow() {
  try {
    await invoke("dismiss_survey");
  } catch (e) {
    logger.error("survey", `Failed to dismiss survey: ${e}`);
  }
  emit("update:visible", false);
}

async function handleDontAskAgain() {
  try {
    await invoke("complete_survey");
  } catch (e) {
    logger.error("survey", `Failed to complete survey: ${e}`);
  }
  emit("update:visible", false);
}

async function handleClose(value: boolean) {
  if (!value) {
    try {
      await invoke("dismiss_survey");
    } catch (e) {
      logger.error("survey", `Failed to dismiss survey: ${e}`);
    }
  }
  emit("update:visible", value);
}
</script>

<template>
  <Dialog
    :visible="visible"
    header="How's Astro-Up working for you?"
    modal
    :closable="true"
    :style="{ width: '480px' }"
    @update:visible="handleClose"
  >
    <div class="survey-body">
      <i class="pi pi-star survey-icon" />
      <div class="survey-content">
        <p class="survey-message">
          We'd love to hear your thoughts! Your feedback helps us make Astro-Up
          better for the astrophotography community.
        </p>
      </div>
    </div>

    <template #footer>
      <div class="survey-footer">
        <div class="survey-secondary-actions">
          <Button
            label="Don't ask again"
            text
            severity="secondary"
            size="small"
            @click="handleDontAskAgain"
          />
          <Button
            label="Not now"
            text
            severity="secondary"
            size="small"
            @click="handleNotNow"
          />
        </div>
        <Button
          label="Leave feedback"
          icon="pi pi-external-link"
          @click="handleFeedback"
        />
      </div>
    </template>
  </Dialog>
</template>

<style scoped>
.survey-body {
  display: flex;
  gap: 16px;
  padding: 8px 0;
}

.survey-icon {
  font-size: 24px;
  color: var(--p-yellow-400);
  flex-shrink: 0;
  margin-top: 2px;
}

.survey-content {
  flex: 1;
}

.survey-message {
  margin: 0;
  color: var(--p-surface-200);
  font-size: 14px;
  line-height: 1.5;
}

.survey-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.survey-secondary-actions {
  display: flex;
  gap: 4px;
}
</style>
