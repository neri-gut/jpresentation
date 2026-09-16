<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";

import { useTimerStore } from "@/stores/timer";

const { t } = useI18n();
const timer = useTimerStore();
const cursorVisible = ref(false);
let hideTimer = 0;

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
  <div class="speaker" :class="{ 'cursor-on': cursorVisible }">
    <div class="hud">
      <p class="title">{{ t("speaker.partPlaceholder") }}</p>
      <p class="time">{{ t("speaker.timePlaceholder") }}</p>
      <div class="bar" :data-state="timer.clock.state"><span /></div>
    </div>
  </div>
</template>

<style scoped>
.speaker {
  height: 100%;
  background: #000;
  color: #f4f4f4;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  cursor: none;
  user-select: none;
  padding: 1.25rem;
}

.speaker.cursor-on {
  cursor: default;
}

.hud {
  min-width: 12rem;
  background: rgb(0 0 0 / 55%);
  border-radius: 8px;
  padding: 0.55rem 0.8rem 0.7rem;
  text-align: center;
}

.title {
  margin: 0 0 0.15rem;
  font-size: 0.85rem;
  opacity: 0.85;
}

.time {
  margin: 0;
  font-variant-numeric: tabular-nums;
  font-size: 1.8rem;
  font-weight: 650;
}

.bar {
  margin-top: 0.35rem;
  height: 4px;
  border-radius: 99px;
  background: rgb(255 255 255 / 18%);
  overflow: hidden;
}

.bar span {
  display: block;
  width: 0;
  height: 100%;
  background: #12b76a;
}
</style>
