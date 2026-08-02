<script setup lang="ts">
import { computed, ref } from "vue";
import {
  NAlert,
  NButton,
  NFlex,
  NIcon,
  NInput,
  NInputGroup,
  NPagination,
  NSpace,
} from "naive-ui";
import { ReloadOutline } from "@vicons/ionicons5";
import { useQuery } from "@tanstack/vue-query";
import { useRouter } from "vue-router";

import { listFiles } from "@/api/client";
import FilesTable from "@/components/FilesTable.vue";

const router = useRouter();
const page = ref(1);
const pageSize = 20;
const hashQuery = ref("");

const { data, isPending, isFetching, error, refetch } = useQuery({
  queryKey: computed(() => ["files", page.value]),
  queryFn: () => listFiles(page.value, pageSize),
});

function searchHash() {
  const hash = hashQuery.value.trim();
  if (hash) router.push(`/hash/${hash}`);
}
</script>

<template>
  <n-space vertical size="large">
    <n-flex justify="space-between" align="center">
      <n-input-group style="max-width: 640px">
        <n-input
          v-model:value="hashQuery"
          placeholder="Search by SHA-256…"
          clearable
          @keyup.enter="searchHash"
        />
        <n-button type="primary" @click="searchHash">Search</n-button>
      </n-input-group>
      <n-button :loading="isFetching" @click="refetch()">
        <template #icon>
          <n-icon :component="ReloadOutline" />
        </template>
        Refresh
      </n-button>
    </n-flex>

    <n-alert v-if="error" type="error">{{ error.message }}</n-alert>

    <files-table
      :files="data?.items ?? []"
      :loading="isPending || isFetching"
    />

    <n-pagination
      v-if="data"
      :page="page"
      :item-count="data.page_info.total"
      :page-size="pageSize"
      @update:page="(p: number) => (page = p)"
    />
  </n-space>
</template>
