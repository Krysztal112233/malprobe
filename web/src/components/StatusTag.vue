<script setup lang="ts">
import { computed } from "vue";
import { NTag } from "naive-ui";

import type { FileStatus } from "@/api/types";

const props = defineProps<{ status: FileStatus }>();

const type = computed(() => {
  switch (props.status) {
    case "completed":
      return "success";
    case "failed":
      return "error";
    case "scanning":
      return "warning";
    default:
      return "default";
  }
});
</script>

<template>
  <n-tag :type="type" size="small" round>{{ status }}</n-tag>
</template>
