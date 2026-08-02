<script setup lang="ts">
import { ref } from "vue";
import {
  NAlert,
  NButton,
  NCard,
  NForm,
  NFormItem,
  NInput,
  NSpace,
  NText,
} from "naive-ui";
import { useMutation } from "@tanstack/vue-query";
import { useRouter } from "vue-router";

import { submitFile } from "@/api/client";

const router = useRouter();
const url = ref("");

const { mutate, isPending, error } = useMutation({
  mutationFn: (u: string) => submitFile(u),
  onSuccess: (file) => router.push(`/report/${file.id}`),
});

function submit() {
  const trimmed = url.value.trim();
  if (trimmed) mutate(trimmed);
}
</script>

<template>
  <n-card
    title="Submit a file for scanning"
    style="max-width: 640px; margin: 0 auto"
  >
    <n-space vertical size="large">
      <n-text depth="3">
        Files are submitted by URL — the worker downloads the bytes and scans
        them with ClamAV.
      </n-text>

      <n-form @submit.prevent="submit">
        <n-form-item label="Download URL" :show-label="true">
          <n-input
            v-model:value="url"
            placeholder="https://example.com/sample.bin"
            :disabled="isPending"
            size="large"
          />
        </n-form-item>
        <n-button
          type="primary"
          attr-type="submit"
          :loading="isPending"
          :disabled="!url.trim()"
          block
        >
          Scan
        </n-button>
      </n-form>

      <n-alert v-if="error" type="error">{{ error.message }}</n-alert>
    </n-space>
  </n-card>
</template>
