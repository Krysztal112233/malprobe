<script setup lang="ts">
import { computed } from "vue";
import { NIcon, NTooltip } from "naive-ui";
import { AlertCircle, CheckmarkCircle, HelpCircle } from "@vicons/ionicons5";

import type { FileVerdict } from "@/api/types";

const props = defineProps<{ verdict: FileVerdict | null }>();

// Three states: checkmark = clean, question mark = uncertain, exclamation = threat.
const icon = computed(() => {
  switch (props.verdict) {
    case "clean":
      return { component: CheckmarkCircle, color: "#18a058", label: "clean" };
    case "malicious":
      return { component: AlertCircle, color: "#d03050", label: "malicious" };
    case "suspicious":
      return { component: HelpCircle, color: "#f0a020", label: "suspicious" };
    default:
      return {
        component: HelpCircle,
        color: "#888",
        label: props.verdict ?? "unknown",
      };
  }
});
</script>

<template>
  <n-tooltip>
    <template #trigger>
      <n-icon
        :component="icon.component"
        :color="icon.color"
        size="22"
        style="display: inline-flex; vertical-align: middle"
      />
    </template>
    {{ icon.label }}
  </n-tooltip>
</template>
