/// <reference types="vite/client" />

interface ImportMetaEnv {
  /**
   * Backend origin baked in at build time, e.g. "http://192.168.1.10:8000".
   * Unset / empty = same-origin requests (dev proxy or reverse proxy).
   */
  readonly VITE_API_BASE_URL?: string;
}

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<object, object, unknown>;
  export default component;
}
