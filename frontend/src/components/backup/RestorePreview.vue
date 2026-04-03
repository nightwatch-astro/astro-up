<script setup lang="ts">
import { computed } from "vue";
import Button from "primevue/button";
import FileTable from "../shared/FileTable.vue";
import type { FileChange } from "../../types/backup";

const props = defineProps<{
  changes: FileChange[];
}>();

defineEmits<{
  confirm: [];
  cancel: [];
}>();

const overwriteCount = computed(() => props.changes.filter((f) => f.action === "overwrite").length);
const newCount = computed(() => props.changes.filter((f) => f.action === "new").length);
const unchangedCount = computed(() => props.changes.filter((f) => f.action === "unchanged").length);
</script>

<template>
  <div class="restore-preview">
    <div class="preview-summary">
      <span class="summary-item overwrite">{{ overwriteCount }} overwrite</span>
      <span class="summary-item new">{{ newCount }} new</span>
      <span class="summary-item unchanged">{{ unchangedCount }} unchanged</span>
    </div>
    <FileTable
      mode="action"
      :changes="changes"
    />
    <div class="preview-actions">
      <Button
        label="Cancel"
        severity="secondary"
        text
        @click="$emit('cancel')"
      />
      <Button
        :label="`Confirm Restore (overwrite ${overwriteCount} files)`"
        severity="warn"
        @click="$emit('confirm')"
      />
    </div>
  </div>
</template>

<style scoped>
.restore-preview {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.preview-summary {
  display: flex;
  gap: 16px;
  font-size: 13px;
  font-weight: 500;
}

.summary-item.overwrite { color: var(--p-red-400); }
.summary-item.new { color: var(--p-green-400); }
.summary-item.unchanged { color: var(--p-surface-400); }

.preview-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
