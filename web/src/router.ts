import { createRouter, createWebHistory } from "vue-router";

import FileDetailPage from "./pages/FileDetailPage.vue";
import FileListPage from "./pages/FileListPage.vue";
import HashSearchPage from "./pages/HashSearchPage.vue";
import SubmitPage from "./pages/SubmitPage.vue";

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", name: "files", component: FileListPage },
    { path: "/submit", name: "submit", component: SubmitPage },
    { path: "/report/:id", name: "file-detail", component: FileDetailPage },
    { path: "/hash/:sha256", name: "hash-search", component: HashSearchPage },
  ],
});
