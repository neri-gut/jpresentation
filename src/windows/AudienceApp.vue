<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";

import { useContainFit } from "@/composables/containFit";
import { useOutputStore } from "@/stores/output";

const { t } = useI18n();
const output = useOutputStore();
const cursorVisible = ref(false);
let hideTimer = 0;

const mediaSrc = computed(() => {
  const path = output.stage.path;
  if (!path) {
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
  document.documentElement.dataset.surface = "audience";
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
  <div class="audience" :class="{ 'cursor-on': cursorVisible }">
    <img
      v-if="output.stage.kind === 'image' && mediaSrc"
      :key="`img-${output.stage.rev}`"
      ref="stillRef"
      class="still"
      :src="mediaSrc"
      :alt="output.stage.name ?? ''"
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
    />
    <p v-else class="mark">{{ t("audience.placeholder") }}</p>
  </div>
</template>

<style scoped>
.audience {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
  margin: 0;
  background: #000;
  color: #d0d0d0;
  cursor: none;
  user-select: none;
}

.audience.cursor-on {
  cursor: default;
}

/* Contain: JS sets pixel size so the image fills without cropping (WebKitGTK). */
.still {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: auto;
  height: auto;
}

.motion {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: contain;
  object-position: center;
  background: #000;
}

.mark {
  margin: 0;
  height: 100%;
  display: grid;
  place-items: center;
  font-size: 2rem;
  letter-spacing: 0.08em;
  opacity: 0.55;
}
</style>
