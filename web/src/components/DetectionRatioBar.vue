<script setup lang="ts">
import { computed } from "vue";
import { NProgress } from "naive-ui";

import { parseEngineResults } from "@/api/types";

const props = defineProps<{ details: unknown }>();

const engines = computed(() => parseEngineResults(props.details));

const total = computed(() => engines.value?.length ?? 0);
const detected = computed(
  () =>
    engines.value?.filter(
      (e) => e.verdict === "malicious" || e.verdict === "suspicious",
    ).length ?? 0,
);

const percentage = computed(() =>
  total.value === 0 ? 0 : Math.round((detected.value / total.value) * 100),
);

const barColor = computed(() => {
  if (detected.value === 0) return "#18a058";
  if (detected.value === total.value) return "#d03050";
  return "#f0a020";
});
</script>

<template>
  <n-progress
    v-if="engines"
    type="line"
    :percentage="percentage"
    :color="barColor"
    rail-color="rgba(255, 255, 255, 0.12)"
    :height="16"
    indicator-placement="inside"
    style="width: 280px"
  >
    <span style="font-size: 12px">{{ detected }} / {{ total }}</span>
  </n-progress>
</template>
