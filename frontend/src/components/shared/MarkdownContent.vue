<script setup lang="ts">
import VueMarkdown from "vue-markdown-render";
import { open as openUrl } from "@tauri-apps/plugin-shell";

defineProps<{
  content: string;
}>();

function handleClick(e: MouseEvent) {
  const anchor = (e.target as HTMLElement).closest("a");
  if (anchor?.href) {
    e.preventDefault();
    openUrl(anchor.href);
  }
}
</script>

<template>
  <div
    class="markdown-content"
    @click="handleClick"
  >
    <VueMarkdown :source="content" />
  </div>
</template>

<style scoped>
.markdown-content {
  font-size: 13px;
  line-height: 1.6;
  color: var(--p-surface-200);
}
.markdown-content :deep(h2) { font-size: 15px; margin: 0 0 8px; color: var(--p-surface-100); }
.markdown-content :deep(h3) { font-size: 14px; margin: 12px 0 6px; color: var(--p-surface-100); }
.markdown-content :deep(ul) { padding-left: 20px; margin: 4px 0; }
.markdown-content :deep(li) { margin: 2px 0; }
.markdown-content :deep(a) { color: var(--p-primary-400); text-decoration: none; cursor: pointer; }
.markdown-content :deep(a:hover) { text-decoration: underline; }
.markdown-content :deep(code) { background: var(--p-surface-800); padding: 1px 4px; border-radius: 3px; font-size: 12px; }
</style>
