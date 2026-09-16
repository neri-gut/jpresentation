import { defineStore } from "pinia";
import { ref } from "vue";

import { invokeCommand } from "@/composables/invoke";
import type {
  AppearanceSetting,
  MonitorDto,
  PanelSetting,
  SettingDto,
  SettingKeyDto,
  SurfacesSetting,
} from "@/types/dto";

function parseJson<T>(value: string, fallback: T): T {
  try {
    return JSON.parse(value) as T;
  } catch {
    return fallback;
  }
}

const defaultAppearance: AppearanceSetting = {
  theme: "system",
  accent: "blue",
  density: "compact",
};

const defaultPanel: PanelSetting = {
  collapsed: false,
  width: 280,
};

const defaultSurfaces: SurfacesSetting = {
  audience_monitor_id: null,
  speaker_monitor_id: null,
  use_speaker: false,
};

/**
 * Operator chrome: theme, panel, monitors, toasts. Audience/speaker windows ignore this store.
 */
export const useUiStore = defineStore("ui", () => {
  const appearance = ref<AppearanceSetting>({ ...defaultAppearance });
  const panel = ref<PanelSetting>({ ...defaultPanel });
  const surfaces = ref<SurfacesSetting>({ ...defaultSurfaces });
  const monitors = ref<MonitorDto[]>([]);
  const toast = ref<string | null>(null);
  const aboutOpen = ref(false);
  const settingsOpen = ref(false);
  const openMenu = ref<"languages" | "tools" | "settings" | null>(null);

  /** Reads appearance, panel, surfaces, and the monitor list for a profile. */
  async function hydrate(profileId: string): Promise<void> {
    appearance.value = await readSetting(profileId, "appearance", defaultAppearance);
    panel.value = await readSetting(profileId, "panel", defaultPanel);
    surfaces.value = await readSetting(profileId, "surfaces", defaultSurfaces);
    applyAppearance(appearance.value);
    try {
      monitors.value = await invokeCommand<MonitorDto[]>("monitors_list");
    } catch {
      monitors.value = [];
    }
  }

  /** Persists appearance and applies CSS tokens on the operator document. */
  async function setAppearance(
    profileId: string,
    next: AppearanceSetting,
  ): Promise<void> {
    appearance.value = next;
    applyAppearance(next);
    await writeSetting(profileId, "appearance", next);
  }

  /** Persists right-panel collapsed/width. */
  async function setPanel(profileId: string, next: PanelSetting): Promise<void> {
    panel.value = next;
    await writeSetting(profileId, "panel", next);
  }

  /** Persists monitor assignment; Rust places the windows. */
  async function setSurfaces(
    profileId: string,
    next: SurfacesSetting,
  ): Promise<void> {
    surfaces.value = next;
    await writeSetting(profileId, "surfaces", next);
  }

  /** Shows a console-only toast. The audience is not notified. */
  function showToast(message: string): void {
    toast.value = message;
    window.setTimeout(() => {
      if (toast.value === message) {
        toast.value = null;
      }
    }, 4000);
  }

  return {
    appearance,
    panel,
    surfaces,
    monitors,
    toast,
    aboutOpen,
    settingsOpen,
    openMenu,
    hydrate,
    setAppearance,
    setPanel,
    setSurfaces,
    showToast,
  };
});

async function readSetting<T>(
  profileId: string,
  key: string,
  fallback: T,
): Promise<T> {
  const payload: SettingKeyDto = { profile_id: profileId, key };
  const cell = await invokeCommand<SettingDto, SettingKeyDto>("settings_get", payload);
  return parseJson(cell.value_json, fallback);
}

async function writeSetting<T>(
  profileId: string,
  key: string,
  value: T,
): Promise<void> {
  const payload: SettingDto = {
    profile_id: profileId,
    key,
    value_json: JSON.stringify(value),
  };
  await invokeCommand<SettingDto, SettingDto>("settings_set", payload);
}

function applyAppearance(setting: AppearanceSetting): void {
  const root = document.documentElement;
  let theme = setting.theme;
  if (theme === "system") {
    theme = window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  }
  root.dataset.theme = theme;
  root.dataset.density = setting.density;
  root.dataset.accent = setting.accent;
}
