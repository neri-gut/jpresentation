<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";

import ClockBar from "@/components/ClockBar.vue";
import { clockHeatColor } from "@/composables/clockColor";
import { formatRemaining } from "@/composables/clockDisplay";
import { useContainFit } from "@/composables/containFit";
import { useOutputStore } from "@/stores/output";
import { useTimerStore } from "@/stores/timer";

const { t } = useI18n();
const output = useOutputStore();
const timer = useTimerStore();
const cursorVisible = ref(false);
let hideTimer = 0;

/** Full-surface clock when idle stage or speaker mode is HUD-only. */
const fullHud = computed(
  () => output.stage.kind === "none" || output.speakerMode === "hud_only",
);

const title = computed(
  () => timer.clock.title ?? t("speaker.partPlaceholder"),
);

const time = computed(() => formatRemaining(timer.clock.remaining_ms));

const heat = computed(() => {
  if (timer.clock.remaining_ms < 0 || timer.clock.progress_pct >= 100) {
    return "#f04438";
  }
  const assigned = timer.clock.assigned_ms;
  if (!assigned) {
    return "#12b76a";
  }
  return clockHeatColor(timer.clock.elapsed_ms / assigned);
});

const mediaSrc = computed(() => {
  const path = output.stage.path;
  if (!path || fullHud.value) {
    return "";
  }
  return convertFileSrc(path);
});

const stillRef = ref<HTMLImageElement | null>(null);
const { fitStyle, layout } = useContainFit(stillRef);

function onMove(): void {
  cursorVisible.value = true;
  window.clearTimeout(hideTimer);
  hideTimer = window.setTimeout(() => {
    cursorVisible.value = false;
  }, 2000);
}

onMounted(() => {
  document.documentElement.dataset.surface = "speaker";
  document.documentElement.style.background = "#000";
  document.body.style.background = "#000";
  window.addEventListener("mousemove", onMove);
});

onUnmounted(() => {
  window.removeEventListener("mousemove", onMove);
  window.clearTimeout(hideTimer);
});
</script>

<template>
  <div
    class="speaker"
    :class="{ 'cursor-on': cursorVisible, 'is-full': fullHud, 'is-overlay': !fullHud }"
  >
    <img
      v-if="output.stage.kind === 'image' && mediaSrc"
      :key="`img-${output.stage.rev}`"
      ref="stillRef"
      class="still"
      :src="mediaSrc"
      alt=""
      :style="fitStyle"
      @load="layout"
    />
    <video
      v-else-if="output.stage.kind === 'video' && mediaSrc"
      :key="`vid-${output.stage.rev}`"
      class="motion"
      :src="mediaSrc"
      autoplay
      playsinline
      muted
    />
    <div class="hud">
      <p class="title">{{ title }}</p>
      <p class="time" :style="{ color: heat }">{{ time }}</p>
    </div>
    <ClockBar
      class="hud-bar"
      :progress-pct="timer.clock.progress_pct"
      :remaining-ms="timer.clock.remaining_ms"
    />
  </div>
</template>

<style scoped>
.speaker {
  height: 100%;
  background: #000;
  color: #f4f4f4;
  cursor: none;
  user-select: none;
  position: relative;
}

.still {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: auto;
  height: auto;
  z-index: 0;
}

.motion {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: contain;
  object-position: center;
  z-index: 0;
}

.hud {
  position: relative;
  z-index: 1;
}

.speaker.cursor-on {
  cursor: default;
}

.speaker.is-full {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 2rem 2rem 2.5rem;
}

.speaker.is-full .hud {
  text-align: center;
}

.speaker.is-full .title {
  margin: 0 0 0.4rem;
  font-size: clamp(1rem, 3.2vw, 2.2rem);
  letter-spacing: 0.04em;
  opacity: 0.8;
}

.speaker.is-full .time {
  margin: 0;
  font-variant-numeric: tabular-nums;
  font-size: clamp(4.5rem, 18vw, 12rem);
  font-weight: 700;
  line-height: 1;
}

.speaker.is-overlay {
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding: 1.25rem 1.25rem 2.5rem;
}

.speaker.is-overlay .hud {
  min-width: 12rem;
  background: rgb(0 0 0 / 55%);
  border-radius: 8px;
  padding: 0.55rem 0.8rem 0.7rem;
  text-align: center;
}

.speaker.is-overlay .title {
  margin: 0 0 0.15rem;
  font-size: 0.85rem;
  opacity: 0.85;
}

.speaker.is-overlay .time {
  margin: 0;
  font-variant-numeric: tabular-nums;
  font-size: 1.8rem;
  font-weight: 650;
}

.hud-bar {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 2;
}
</style>
