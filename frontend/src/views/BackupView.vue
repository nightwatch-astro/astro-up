<script setup lang="ts">
import EmptyState from "../components/shared/EmptyState.vue";
</script>

<template>
  <div class="page-view">
    <div class="page-hdr">
      <h2>Backup &amp; Restore</h2>
      <p>Manage configuration backups for your software</p>
    </div>

    <EmptyState
      icon="pi-database"
      message="No backups yet. Backups are created automatically before updates, or you can create one manually from a package's detail page."
    />
  </div>
</template>
