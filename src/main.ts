import { createPinia } from "pinia";
import { createApp } from "vue";

import App from "@/App.vue";
import { isTauri } from "@/composables/invoke";
import { speakerMonitorMissing } from "@/composables/surfaces";
import { i18n } from "@/i18n";
import { router } from "@/router";
import { useOutputStore } from "@/stores/output";
import { useProfileStore } from "@/stores/profile";
import { useTimerStore } from "@/stores/timer";
import { useUiStore } from "@/stores/ui";

import "@/styles/base.css";

function surfaceKind(): "operator" | "audience" | "speaker" | "identify" {
  const hash = window.location.hash;
  if (hash.startsWith("#/audience")) {
    return "audience";
  }
  if (hash.startsWith("#/speaker")) {
    return "speaker";
  }
  if (hash.startsWith("#/identify")) {
    return "identify";
  }
  return "operator";
}

/**
 * Boots Vue. Operator hydrates profiles and chrome; audience/speaker only subscribe to output.
 */
async function bootstrap(): Promise<void> {
  const app = createApp(App);
  const pinia = createPinia();
  app.use(pinia);
  app.use(i18n);
  app.use(router);

  if (isTauri()) {
    const kind = surfaceKind();
    if (kind !== "identify") {
      const output = useOutputStore();
      const timer = useTimerStore();
      await output.subscribe();
      await timer.subscribe();
    }
    if (kind === "operator") {
      const profiles = useProfileStore();
      const ui = useUiStore();
      try {
        await profiles.hydrate();
        if (profiles.current) {
          await ui.hydrate(profiles.current.id);
          if (speakerMonitorMissing(ui.surfaces, ui.monitors)) {
            ui.showToast(String(i18n.global.t("errors.monitorMissing")));
          }
        }
      } catch (err) {
        const message =
          err && typeof err === "object" && "message" in err
            ? String((err as { message: string }).message)
            : "bootstrap failed";
        ui.showToast(message);
      }
    }
  }

  await router.isReady();
  app.mount("#app");
}

void bootstrap();
