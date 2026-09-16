<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";

import { useOutputStore } from "@/stores/output";

const { t } = useI18n();
const output = useOutputStore();
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
    <p class="mark">{{ t("audience.placeholder") }}</p>
    <span class="sr-only">{{ output.stage.kind }}</span>
  </div>
</template>

<style scoped>
.audience {
  height: 100%;
  margin: 0;
  background: #000;
  color: #d0d0d0;
  display: grid;
  place-items: center;
  cursor: none;
  user-select: none;
}

.audience.cursor-on {
  cursor: default;
}

.mark {
  margin: 0;
  font-size: 2rem;
  letter-spacing: 0.08em;
  opacity: 0.55;
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip: rect(0 0 0 0);
}
</style>
