<script setup lang="ts">
import { computed } from "vue";
import {
  NAlert,
  NDescriptions,
  NDescriptionsItem,
  NSpin,
  NButton,
  NSpace,
} from "naive-ui";
import { useQuery } from "@tanstack/vue-query";
import { useRoute, useRouter } from "vue-router";

import { getFile } from "@/api/client";
import { parseEngineResults } from "@/api/types";
import EngineResults from "@/components/EngineResults.vue";
import DetectionRatioBar from "@/components/DetectionRatioBar.vue";
import StatusTag from "@/components/StatusTag.vue";
import VerdictIcon from "@/components/VerdictIcon.vue";
import { formatBytes, formatTime } from "@/format";

const route = useRoute();
const router = useRouter();
const id = computed(() => route.params.id as string);

const {
  data: file,
  isPending,
  error,
} = useQuery({
  queryKey: computed(() => ["file", id.value]),
  queryFn: () => getFile(id.value),
  // Poll until the scan reaches a terminal state.
  refetchInterval: (query) => {
    const status = query.state.data?.status;
    return status === "completed" || status === "failed" ? false : 2_000;
  },
});
</script>

<template>
  <n-space vertical size="large">
    <n-button text @click="router.back()">← Back</n-button>

    <n-spin :show="isPending">
      <n-alert v-if="error" type="error">{{ error.message }}</n-alert>

      <template v-else-if="file">
        <n-alert
          v-if="file.status === 'failed'"
          type="error"
          style="margin-bottom: 16px"
        >
          Scan failed: {{ file.error ?? "unknown error" }}
        </n-alert>
        <n-alert
          v-else-if="file.verdict === 'malicious'"
          type="error"
          style="margin-bottom: 16px"
        >
          Malicious: {{ file.malware_name ?? "detected" }}
        </n-alert>
        <n-alert
          v-else-if="file.verdict === 'suspicious'"
          type="warning"
          style="margin-bottom: 16px"
        >
          Suspicious
        </n-alert>
        <n-alert
          v-else-if="file.verdict === 'clean'"
          type="success"
          style="margin-bottom: 16px"
        >
          Clean
        </n-alert>
        <n-alert v-else type="info" style="margin-bottom: 16px">
          Scan in progress… this page refreshes automatically.
        </n-alert>

        <n-descriptions bordered :column="1" label-placement="left">
          <n-descriptions-item label="ID">{{ file.id }}</n-descriptions-item>
          <n-descriptions-item label="Status"
            ><status-tag :status="file.status"
          /></n-descriptions-item>
          <n-descriptions-item label="Verdict">
            <n-space align="center">
              <detection-ratio-bar :details="file.details" />
              <verdict-icon :verdict="file.verdict" />
            </n-space>
          </n-descriptions-item>
          <n-descriptions-item label="SHA-256">{{
            file.sha256 ?? "—"
          }}</n-descriptions-item>
          <n-descriptions-item label="MIME type">{{
            file.mime_type ?? "—"
          }}</n-descriptions-item>
          <n-descriptions-item label="Size">{{
            formatBytes(file.size)
          }}</n-descriptions-item>
          <n-descriptions-item label="Malware name">{{
            file.malware_name ?? "—"
          }}</n-descriptions-item>
          <n-descriptions-item label="Submitted">{{
            formatTime(file.created_at)
          }}</n-descriptions-item>
          <n-descriptions-item label="Scanned at">{{
            formatTime(file.scanned_at)
          }}</n-descriptions-item>
        </n-descriptions>

        <engine-results :details="file.details" />

        <!-- Fallback: details exists but isn't per-engine results. -->
        <pre
          v-if="file.details && !parseEngineResults(file.details)"
          class="raw-json"
          >{{ JSON.stringify(file.details, null, 2) }}</pre>
      </template>
    </n-spin>
  </n-space>
</template>

<style scoped>
.raw-json {
  padding: 12px;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.06);
  font-size: 12px;
  overflow-x: auto;
}
</style>
