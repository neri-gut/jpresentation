import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { ref } from "vue";

import { invokeCommand, isTauri } from "@/composables/invoke";
import type { ClockSnapshot, OutputBundleDto } from "@/types/dto";

const TIMER_CHANGED = "timer://changed";

/**
 * Meeting clock cache for the speaker HUD and the operator timer block.
 * The clock does not launch media.
 */
export const useTimerStore = defineStore("timer", () => {
  const clock = ref<ClockSnapshot>({ rev: 0, state: "idle" });
  let unlisten: UnlistenFn | undefined;

  /** Fetches the current clock and subscribes to `timer://changed`. */
  async function subscribe(): Promise<void> {
    if (!isTauri()) {
      return;
    }
    const bundle = await invokeCommand<OutputBundleDto>("output_get");
    apply(bundle.clock);
    unlisten = await listen<ClockSnapshot>(TIMER_CHANGED, (event) => {
      apply(event.payload);
    });
  }

  /** Applies a snapshot if it is not older than the one already painted. */
  function apply(next: ClockSnapshot): void {
    if (next.rev < clock.value.rev) {
      return;
    }
    clock.value = next;
  }

  /** Drops the Tauri listener. */
  function dispose(): void {
    if (unlisten) {
      void unlisten();
      unlisten = undefined;
    }
  }

  return { clock, subscribe, apply, dispose };
});
