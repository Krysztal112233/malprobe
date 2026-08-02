<script setup lang="ts">
import { computed, ref } from "vue";
import { NButton, NCard, NIcon, NText } from "naive-ui";
import { CodeSlashOutline } from "@vicons/ionicons5";

import { parseEngineResults } from "@/api/types";
import VerdictTag from "@/components/VerdictTag.vue";

const props = defineProps<{ details: unknown }>();

const showRaw = ref(false);

const engines = computed(() => parseEngineResults(props.details));

// Engines that flagged the file (malicious or suspicious).
const detected = computed(
  () =>
    engines.value?.filter(
      (e) => e.verdict === "malicious" || e.verdict === "suspicious",
    ).length ?? 0,
);

const summaryType = computed(() => {
  if (!engines.value) return "default";
  if (detected.value === 0) return "success";
  return detected.value === engines.value.length ? "error" : "warning";
});
</script>

<template>
  <n-card v-if="engines">
    <template #header>
      Engine results
      <n-text :type="summaryType" depth="2" style="margin-left: 8px">
        {{ detected }} / {{ engines.length }} detected
      </n-text>
    </template>

    <template #header-extra>
      <n-button size="tiny" quaternary @click="showRaw = !showRaw">
        <template #icon>
          <n-icon :component="CodeSlashOutline" />
        </template>
        Raw JSON
      </n-button>
    </template>

    <div class="engine-grid">
      <n-card
        v-for="engine in engines"
        :key="engine.name"
        size="small"
        embedded
        :class="['engine-card', engine.verdict]"
      >
        <div class="engine-name">{{ engine.name }}</div>
        <verdict-tag :verdict="engine.verdict" />
        <div class="engine-signature">
          <n-text v-if="engine.malware_name" type="error">{{
            engine.malware_name
          }}</n-text>
          <n-text v-else depth="3">No detection</n-text>
        </div>
      </n-card>
    </div>

    <pre v-if="showRaw" class="raw-json">{{
      JSON.stringify(details, null, 2)
    }}</pre>
  </n-card>
</template>

<style scoped>
.engine-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 12px;
}

.engine-card {
  border-left: 3px solid transparent;
}

.engine-card.malicious {
  border-left-color: #d03050;
}

.engine-card.suspicious {
  border-left-color: #f0a020;
}

.engine-card.clean {
  border-left-color: #18a058;
}

.engine-name {
  font-weight: 600;
  margin-bottom: 8px;
}

.engine-signature {
  margin-top: 8px;
  font-family: monospace;
  font-size: 12px;
  word-break: break-all;
}

.raw-json {
  margin: 12px 0 0;
  padding: 12px;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.06);
  font-size: 12px;
  overflow-x: auto;
}
</style>
