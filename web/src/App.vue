<script setup lang="ts">
import {
  darkTheme,
  NConfigProvider,
  NDialogProvider,
  NLayout,
  NLayoutHeader,
  NLayoutContent,
  NMenu,
  NMessageProvider,
} from "naive-ui";
import type { MenuOption } from "naive-ui";
import { computed, h } from "vue";
import { RouterLink, RouterView, useRoute } from "vue-router";

const route = useRoute();

const menuOptions: MenuOption[] = [
  {
    label: () => h(RouterLink, { to: "/" }, { default: () => "Files" }),
    key: "files",
  },
  {
    label: () =>
      h(RouterLink, { to: "/submit" }, { default: () => "Submit URL" }),
    key: "submit",
  },
];

const activeKey = computed(() =>
  route.path.startsWith("/submit") ? "submit" : "files",
);
</script>

<template>
  <n-config-provider :theme="darkTheme">
    <n-message-provider>
      <n-dialog-provider>
        <n-layout style="min-height: 100vh">
          <n-layout-header
            bordered
            style="
              display: flex;
              align-items: center;
              gap: 24px;
              padding: 0 24px;
              height: 56px;
            "
          >
            <span style="font-size: 18px; font-weight: 700">malprobe</span>
            <n-menu
              mode="horizontal"
              :options="menuOptions"
              :value="activeKey"
            />
          </n-layout-header>
          <n-layout-content
            style="
              padding: 24px;
              max-width: 1200px;
              margin: 0 auto;
              width: 100%;
            "
          >
            <router-view />
          </n-layout-content>
        </n-layout>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>
