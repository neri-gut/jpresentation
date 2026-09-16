<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";

import { formatRemaining } from "@/composables/clockDisplay";
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
    :data-hue="timer.clock.hue"
  >
    <div class="hud">
      <p class="title">{{ title }}</p>
      <p class="time">{{ time }}</p>
    </div>
    <div class="bar" :data-state="timer.clock.state" :data-hue="timer.clock.hue">
      <span :style="{ width: `${timer.clock.progress_pct}%` }" />
    </div>
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

.speaker[data-hue="green"] .time {
  color: #12b76a;
}

.speaker[data-hue="amber"] .time {
  color: #f5a524;
}

.speaker[data-hue="red"] .time {
  color: #f04438;
}

.bar {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: 12px;
  background: rgb(255 255 255 / 18%);
  overflow: hidden;
}

.bar span {
  display: block;
  width: 0;
  height: 100%;
  background: #12b76a;
}

.bar[data-hue="amber"] span {
  background: #f5a524;
}

.bar[data-hue="red"] span {
  background: #f04438;
}
</style>
