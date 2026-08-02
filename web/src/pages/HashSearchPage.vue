<script setup lang="ts">
import { computed } from "vue";
import { NAlert, NButton, NCard, NSpace, NText } from "naive-ui";
import { useQuery } from "@tanstack/vue-query";
import { useRoute, useRouter } from "vue-router";

import { getFilesByHash } from "@/api/client";
import FilesTable from "@/components/FilesTable.vue";

const route = useRoute();
const router = useRouter();
const sha256 = computed(() => route.params.sha256 as string);

const { data, isPending, isFetching, error } = useQuery({
  queryKey: computed(() => ["hash", sha256.value]),
  queryFn: () => getFilesByHash(sha256.value),
  refetchInterval: (query) =>
    query.state.data?.items.some(
      (f) => f.status === "pending" || f.status === "scanning",
    )
      ? 3_000
      : false,
});
</script>

<template>
  <n-space vertical size="large">
    <n-button text @click="router.push('/')">← All files</n-button>

    <n-card>
      <n-text depth="3">All scan reports for hash</n-text>
      <div
        style="word-break: break-all; font-family: monospace; margin-top: 4px"
      >
        {{ sha256 }}
      </div>
    </n-card>

    <n-alert v-if="error" type="error">{{ error.message }}</n-alert>
    <n-alert v-else-if="data && data.items.length === 0" type="info">
      No scan reports found for this hash.
    </n-alert>

    <files-table
      :files="data?.items ?? []"
      :loading="isPending || isFetching"
    />
  </n-space>
</template>
