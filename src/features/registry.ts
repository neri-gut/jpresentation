import type { Component } from "vue";

/**
 * Operator tab contract. The shell only reads this list; a later change adds a module
 * by pushing one object here (or to `extraFeatureModules`) without rewriting OperatorShell.
 */
export interface FeatureModule {
  id: string;
  navKey: string;
  path: string;
  load: () => Promise<{ default: Component }>;
}

/** Core operator tabs. Each view is a placeholder until its domain change lands. */
export const coreFeatureModules: FeatureModule[] = [
  {
    id: "songs",
    navKey: "nav.songs",
    path: "",
    load: () => import("@/views/SongsView.vue"),
  },
  {
    id: "timer",
    navKey: "nav.timer",
    path: "timer",
    load: () => import("@/views/TimerView.vue"),
  },
  {
    id: "media",
    navKey: "nav.media",
    path: "media",
    load: () => import("@/views/MediaView.vue"),
  },
  {
    id: "bible",
    navKey: "nav.bible",
    path: "bible",
    load: () => import("@/views/BibleView.vue"),
  },
  {
    id: "browser",
    navKey: "nav.browser",
    path: "browser",
    load: () => import("@/views/BrowserView.vue"),
  },
  {
    id: "text",
    navKey: "nav.text",
    path: "text",
    load: () => import("@/views/TextView.vue"),
  },
];

/** Extra Vue feature modules. Empty in change 001. */
export const extraFeatureModules: FeatureModule[] = [];

/** Tabs the operator shell should render. */
export function registeredFeatures(): FeatureModule[] {
  return [...coreFeatureModules, ...extraFeatureModules];
}
