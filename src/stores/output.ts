import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { ref } from "vue";

import { invokeCommand, isTauri } from "@/composables/invoke";
import type { OutputBundleDto, SpeakerMode, SpeakerUiDto, StageSnapshot } from "@/types/dto";

const OUTPUT_CHANGED = "output://changed";
const SPEAKER_UI_CHANGED = "speaker://ui";

/**
 * Audience stage cache. Rust owns the snapshot; this store only paints the last event.
 */
export const useOutputStore = defineStore("output", () => {
  const stage = ref<StageSnapshot>({
    rev: 0,
    kind: "none",
    name: null,
    mime: null,
    path: null,
  });
  const speakerMode = ref<SpeakerMode>("mirror");
  let unlistenOutput: UnlistenFn | undefined;
  let unlistenSpeaker: UnlistenFn | undefined;

  /** Fetches the current snapshot and subscribes to stage / speaker-ui events. */
  async function subscribe(): Promise<void> {
    if (!isTauri()) {
      return;
    }
    const bundle = await invokeCommand<OutputBundleDto>("output_get");
    apply(bundle.stage);
    speakerMode.value = bundle.speaker_mode;
    unlistenOutput = await listen<StageSnapshot>(OUTPUT_CHANGED, (event) => {
      apply(event.payload);
    });
    unlistenSpeaker = await listen<SpeakerUiDto>(SPEAKER_UI_CHANGED, (event) => {
      speakerMode.value = event.payload.mode;
    });
  }

  /** Applies a snapshot if it is not older than the one already painted. */
  function apply(next: StageSnapshot): void {
    if (next.rev < stage.value.rev) {
      return;
    }
    stage.value = next;
  }

  /** Drops the Tauri listeners. */
  function dispose(): void {
    if (unlistenOutput) {
      void unlistenOutput();
      unlistenOutput = undefined;
    }
    if (unlistenSpeaker) {
      void unlistenSpeaker();
      unlistenSpeaker = undefined;
    }
  }

  return { stage, speakerMode, subscribe, apply, dispose };
});
