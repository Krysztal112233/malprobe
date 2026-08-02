<script setup lang="ts">
import { computed } from "vue";
import { NTag } from "naive-ui";

import type { FileVerdict } from "@/api/types";

const props = defineProps<{ verdict: FileVerdict | null }>();

const type = computed(() => {
  switch (props.verdict) {
    case "clean":
      return "success";
    case "suspicious":
      return "warning";
    case "malicious":
      return "error";
    case "error":
      return "default";
    default:
      return "default";
  }
});
</script>

<template>
  <n-tag v-if="verdict" :type="type" size="small" round>{{ verdict }}</n-tag>
  <span v-else>—</span>
</template>
