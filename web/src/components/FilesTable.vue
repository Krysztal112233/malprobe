<script setup lang="ts">
import { h } from "vue";
import { NButton, NDataTable, NText } from "naive-ui";
import type { DataTableColumns } from "naive-ui";
import { useRouter } from "vue-router";

import type { FileVO } from "@/api/types";
import StatusTag from "@/components/StatusTag.vue";
import VerdictTag from "@/components/VerdictTag.vue";
import { formatBytes, formatTime, shortHash, shortId } from "@/format";

defineProps<{
  files: FileVO[];
  loading: boolean;
}>();

const router = useRouter();

// Center all column headers (and cells) for a more symmetric report look.
const centered = { align: "center" } as const;

const columns: DataTableColumns<FileVO> = [
  {
    title: "ID",
    key: "id",
    ...centered,
    render(row) {
      return h(
        NButton,
        {
          text: true,
          type: "primary",
          title: row.id,
          onClick: () => router.push(`/report/${row.id}`),
        },
        { default: () => shortId(row.id) },
      );
    },
  },
  {
    title: "SHA-256",
    key: "sha256",
    ...centered,
    render(row) {
      return h(
        NButton,
        {
          text: true,
          type: "primary",
          onClick: () => row.sha256 && router.push(`/hash/${row.sha256}`),
        },
        { default: () => shortHash(row.sha256) },
      );
    },
  },
  {
    title: "Type",
    key: "mime_type",
    ...centered,
    render: (row) => row.mime_type ?? "—",
  },
  {
    title: "Size",
    key: "size",
    ...centered,
    render: (row) => formatBytes(row.size),
  },
  {
    title: "Status",
    key: "status",
    ...centered,
    render: (row) => h(StatusTag, { status: row.status }),
  },
  {
    title: "Verdict",
    key: "verdict",
    ...centered,
    render: (row) => h(VerdictTag, { verdict: row.verdict }),
  },
  {
    title: "Malware",
    key: "malware_name",
    ...centered,
    render(row) {
      return row.malware_name
        ? h(NText, { type: "error" }, { default: () => row.malware_name })
        : "—";
    },
  },
  {
    title: "Submitted",
    key: "created_at",
    ...centered,
    render: (row) => formatTime(row.created_at),
  },
];
</script>

<template>
  <n-data-table
    :columns="columns"
    :data="files"
    :loading="loading"
    :row-key="(row: FileVO) => row.id"
  />
</template>
