<script setup lang="ts">
import { useI18n } from "vue-i18n";

import TimerControls from "@/components/TimerControls.vue";

const { t } = useI18n();
</script>

<template>
  <section class="timer-view">
    <h1>{{ t("nav.timer") }}</h1>
    <p>{{ t("timer.viewHint") }}</p>
    <TimerControls />
  </section>
</template>

<style scoped>
.timer-view {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  max-width: 28rem;
}

h1 {
  margin: 0;
  font-size: 1.1rem;
}

p {
  margin: 0;
  color: var(--jp-muted);
}
</style>
