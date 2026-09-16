import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { ref } from "vue";

import { invokeCommand, isTauri } from "@/composables/invoke";
import type { ClockArmDto, ClockSnapshot, OutputBundleDto } from "@/types/dto";
import { idleClock } from "@/types/dto";

const TIMER_CHANGED = "timer://changed";

/**
 * Meeting clock cache for the speaker HUD and the operator timer block.
 * The clock does not launch media. Vue is not the tick owner.
 */
export const useTimerStore = defineStore("timer", () => {
  const clock = ref<ClockSnapshot>(idleClock());
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

  /** Arms a single part. Does not start counting. */
  async function arm(title: string, minutes: number): Promise<void> {
    const payload: ClockArmDto = { title, minutes };
    const next = await invokeCommand<ClockSnapshot, ClockArmDto>("clock_arm", payload);
    apply(next);
  }

  /** Starts or resumes the armed part. */
  async function start(): Promise<void> {
    apply(await invokeCommand<ClockSnapshot>("clock_start"));
  }

  /** Pauses a running part. */
  async function pause(): Promise<void> {
    apply(await invokeCommand<ClockSnapshot>("clock_pause"));
  }

  /** Clears the part without arming a next row. */
  async function finish(): Promise<void> {
    apply(await invokeCommand<ClockSnapshot>("clock_finish"));
  }

  /** Drops the Tauri listener. */
  function dispose(): void {
    if (unlisten) {
      void unlisten();
      unlisten = undefined;
    }
  }

  return { clock, subscribe, apply, arm, start, pause, finish, dispose };
});
