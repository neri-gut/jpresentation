import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { ref } from "vue";

import { invokeCommand, isTauri } from "@/composables/invoke";
import type {
  WeekBundleDto,
  WeekPreviewDto,
  WeekPreviewRequestDto,
  WeekProgressDto,
  WeekScopeDto,
  WeekWhich,
} from "@/types/dto";

const WEEK_PROGRESS = "week://progress";

function emptyWeek(kind: "midweek" | "weekend"): WeekBundleDto["midweek"] {
  return {
    monday: "",
    kind,
    title: "",
    langwritten: "",
    pub_symbol: kind === "midweek" ? "mwb" : "w",
    issue: "",
    parts: [],
  };
}

/**
 * Operator cache of this/next week's programme. Rust owns disk and HTTP.
 */
export const useWeekStore = defineStore("week", () => {
  const which = ref<WeekWhich>("this");
  const bundle = ref<WeekBundleDto>({
    monday: "",
    langwritten: "",
    midweek: emptyWeek("midweek"),
    weekend: emptyWeek("weekend"),
  });
  const progress = ref<WeekProgressDto | null>(null);
  const busy = ref(false);
  let unlisten: UnlistenFn | undefined;

  /** Loads persisted weeks and listens for progress events. */
  async function hydrate(): Promise<void> {
    if (!isTauri()) {
      return;
    }
    await reload();
    if (!unlisten) {
      unlisten = await listen<WeekProgressDto>(WEEK_PROGRESS, (event) => {
        progress.value = event.payload;
      });
    }
  }

  /** Re-reads SQLite without hitting the catalog. */
  async function reload(): Promise<void> {
    if (!isTauri()) {
      return;
    }
    const payload: WeekScopeDto = { which: which.value };
    bundle.value = await invokeCommand<WeekBundleDto, WeekScopeDto>("week_get", payload);
  }

  /** Switches this/next and reloads. */
  async function setWhich(next: WeekWhich): Promise<void> {
    which.value = next;
    await reload();
  }

  /** GETPUBMEDIALINKS + parse. Operator-triggered only. */
  async function fetchWeek(): Promise<void> {
    busy.value = true;
    try {
      const payload: WeekScopeDto = { which: which.value };
      bundle.value = await invokeCommand<WeekBundleDto, WeekScopeDto>(
        "week_fetch",
        payload,
      );
    } finally {
      busy.value = false;
      progress.value = null;
    }
  }

  /** Downloads pending catalog videos, not hymnal tracks. */
  async function downloadMedia(): Promise<void> {
    busy.value = true;
    try {
      const payload: WeekScopeDto = { which: which.value };
      bundle.value = await invokeCommand<WeekBundleDto, WeekScopeDto>(
        "week_download_media",
        payload,
      );
    } finally {
      busy.value = false;
      progress.value = null;
    }
  }

  /** Signals the download loop to stop after the current file. */
  async function cancel(): Promise<void> {
    await invokeCommand("week_cancel");
  }

  /** Image preview as a data URL. Video/song return null. */
  async function preview(itemId: string): Promise<WeekPreviewDto | null> {
    const payload: WeekPreviewRequestDto = { which: which.value, item_id: itemId };
    return invokeCommand<WeekPreviewDto | null, WeekPreviewRequestDto>(
      "week_preview",
      payload,
    );
  }

  return {
    which,
    bundle,
    progress,
    busy,
    hydrate,
    reload,
    setWhich,
    fetchWeek,
    downloadMedia,
    cancel,
    preview,
  };
});
