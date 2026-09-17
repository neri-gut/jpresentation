import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { ref } from "vue";

import { invokeCommand, isTauri } from "@/composables/invoke";
import type {
  ApplyTemplateDto,
  EventTemplateDto,
  MeetingKind,
  MeetingPart,
  SaveTemplateDto,
  SetPartsDto,
  TemplateIdDto,
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
    media: [],
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
  const templates = ref<EventTemplateDto[]>([]);
  const selectedMeeting = ref<MeetingKind>("midweek");
  const selectedPartId = ref<string | null>(null);
  let unlisten: UnlistenFn | undefined;

  /** Loads persisted weeks and listens for progress events. */
  async function hydrate(): Promise<void> {
    if (!isTauri()) {
      return;
    }
    await reload();
    await loadTemplates();
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

  /** Re-parses cached JWPUB (or the system skeleton) without the catalog. */
  async function restore(): Promise<void> {
    const payload: WeekScopeDto = { which: which.value };
    bundle.value = await invokeCommand<WeekBundleDto, WeekScopeDto>("week_restore", payload);
  }

  /** Replaces the part list of one meeting. */
  async function setParts(meeting: MeetingKind, parts: MeetingPart[]): Promise<void> {
    const payload: SetPartsDto = { which: which.value, meeting, parts };
    bundle.value = await invokeCommand<WeekBundleDto, SetPartsDto>("week_set_parts", payload);
  }

  async function loadTemplates(): Promise<void> {
    if (!isTauri()) {
      return;
    }
    templates.value = await invokeCommand<EventTemplateDto[]>("template_list");
  }

  async function applyTemplate(meeting: MeetingKind, templateId: string): Promise<void> {
    const payload: ApplyTemplateDto = {
      which: which.value,
      meeting,
      template_id: templateId,
    };
    bundle.value = await invokeCommand<WeekBundleDto, ApplyTemplateDto>(
      "template_apply",
      payload,
    );
  }

  async function saveTemplate(
    name: string,
    kind: SaveTemplateDto["kind"],
    parts: MeetingPart[],
  ): Promise<void> {
    const payload: SaveTemplateDto = { name, kind, parts };
    await invokeCommand<EventTemplateDto, SaveTemplateDto>("template_save", payload);
    await loadTemplates();
  }

  async function deleteTemplate(id: string): Promise<void> {
    const payload: TemplateIdDto = { id };
    await invokeCommand<void, TemplateIdDto>("template_delete", payload);
    await loadTemplates();
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
    templates,
    selectedMeeting,
    selectedPartId,
    hydrate,
    reload,
    setWhich,
    fetchWeek,
    downloadMedia,
    cancel,
    restore,
    setParts,
    loadTemplates,
    applyTemplate,
    saveTemplate,
    deleteTemplate,
    preview,
  };
});
