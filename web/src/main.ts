import { createApp } from "vue";
import { VueQueryPlugin } from "@tanstack/vue-query";

import App from "./App.vue";
import { router } from "./router";

const app = createApp(App);
app.use(router);
app.use(VueQueryPlugin, {
  queryClientConfig: {
    defaultOptions: {
      queries: {
        retry: 1,
        staleTime: 5_000,
      },
    },
  },
});
app.mount("#app");
