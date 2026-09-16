import { createPinia } from "pinia";
import { createApp } from "vue";

import App from "@/App.vue";
import { isTauri } from "@/composables/invoke";
import { i18n } from "@/i18n";
import { router } from "@/router";
import { useOutputStore } from "@/stores/output";
import { useProfileStore } from "@/stores/profile";
import { useTimerStore } from "@/stores/timer";
import { useUiStore } from "@/stores/ui";

import "@/styles/base.css";

function isOperatorSurface(): boolean {
  const hash = window.location.hash;
  return !hash.startsWith("#/audience") && !hash.startsWith("#/speaker");
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
    const output = useOutputStore();
    const timer = useTimerStore();
    await output.subscribe();
    await timer.subscribe();
    if (isOperatorSurface()) {
      const profiles = useProfileStore();
      const ui = useUiStore();
      try {
        await profiles.hydrate();
        if (profiles.current) {
          await ui.hydrate(profiles.current.id);
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
