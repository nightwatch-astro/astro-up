<script setup lang="ts">
import Card from "primevue/card";

const version = __APP_VERSION__;
</script>

<template>
  <Card style="width: 25rem">
    <template #title>
      Astro-Up
    </template>
    <template #subtitle>
      v{{ version }}
    </template>
    <template #content>
      <p>Astrophotography software manager for Windows.</p>
      <p>Install, detect, and update imaging software.</p>
    </template>
  </Card>
</template>
