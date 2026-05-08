<script setup lang="ts">
import { cn } from "@/lib/utils";

withDefaults(
  defineProps<{
    class?: string;
  }>(),
  {
    class: "",
  },
);
</script>

<template>
  <section :class="cn('rounded-2xl border bg-card text-card-foreground shadow-[0_16px_40px_-32px_rgba(24,56,118,0.42)] backdrop-blur', $props.class)">
    <slot />
  </section>
</template>
