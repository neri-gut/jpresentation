import { createRouter, createWebHashHistory } from "vue-router";

import { registeredFeatures } from "@/features/registry";
import OperatorShell from "@/layout/OperatorShell.vue";

const featureChildren = registeredFeatures().map((feature) => ({
  path: feature.path,
  name: feature.id,
  component: feature.load,
}));

/**
 * Hash routes so file:// and the Tauri custom protocol keep operator / audience / speaker distinct.
 */
export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      component: OperatorShell,
      children: featureChildren,
    },
    {
      path: "/audience",
      name: "audience",
      component: () => import("@/windows/AudienceApp.vue"),
    },
    {
      path: "/speaker",
      name: "speaker",
      component: () => import("@/windows/SpeakerApp.vue"),
    },
    {
      path: "/identify",
      name: "identify",
      component: () => import("@/windows/IdentifyApp.vue"),
    },
  ],
});
