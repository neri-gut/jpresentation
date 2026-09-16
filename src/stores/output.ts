import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { ref } from "vue";

import { invokeCommand, isTauri } from "@/composables/invoke";
import type { OutputBundleDto, StageSnapshot } from "@/types/dto";

const OUTPUT_CHANGED = "output://changed";

/**
 * Audience stage cache. Rust owns the snapshot; this store only paints the last event.
 */
export const useOutputStore = defineStore("output", () => {
  const stage = ref<StageSnapshot>({ rev: 0, kind: "none" });
  let unlisten: UnlistenFn | undefined;

  /** Fetches the current snapshot and subscribes to `output://changed`. */
  async function subscribe(): Promise<void> {
    if (!isTauri()) {
      return;
    }
    const bundle = await invokeCommand<OutputBundleDto>("output_get");
    apply(bundle.stage);
    unlisten = await listen<StageSnapshot>(OUTPUT_CHANGED, (event) => {
      apply(event.payload);
    });
  }

  /** Applies a snapshot if it is not older than the one already painted. */
  function apply(next: StageSnapshot): void {
    if (next.rev < stage.value.rev) {
      return;
    }
    stage.value = next;
  }

  /** Drops the Tauri listener. */
  function dispose(): void {
    if (unlisten) {
      void unlisten();
      unlisten = undefined;
    }
  }

  return { stage, subscribe, apply, dispose };
});
