<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import ClockBar from "@/components/ClockBar.vue";
import { clockHeatColor } from "@/composables/clockColor";
import { formatRemaining } from "@/composables/clockDisplay";
import { errorMessage } from "@/composables/errors";
import { useTimerStore } from "@/stores/timer";
import { useUiStore } from "@/stores/ui";

const { t } = useI18n();
const timer = useTimerStore();
const ui = useUiStore();

const title = ref(t("timer.defaultTitle"));
const minutes = ref(10);

const clock = computed(() => timer.clock);
const display = computed(() => formatRemaining(clock.value.remaining_ms));
const canStart = computed(() => clock.value.state === "armed");
const canPause = computed(() => clock.value.state === "running");
const canFinish = computed(
  () => clock.value.state === "armed" || clock.value.state === "running",
);

const heat = computed(() => {
  if (clock.value.remaining_ms < 0 || clock.value.progress_pct >= 100) {
    return "#f04438";
  }
  if (!clock.value.assigned_ms) {
    return "var(--jp-ok)";
  }
  return clockHeatColor(clock.value.elapsed_ms / clock.value.assigned_ms);
});

async function run(action: () => Promise<void>): Promise<void> {
  try {
    await action();
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
}

function arm(): Promise<void> {
  return run(() => timer.arm(title.value, Number(minutes.value)));
}

function start(): Promise<void> {
  return run(() => timer.start());
}

function pause(): Promise<void> {
  return run(() => timer.pause());
}

function finish(): Promise<void> {
  return run(() => timer.finish());
}
</script>

<template>
  <div class="timer-controls">
    <p class="status">
      <span class="part">{{ clock.title ?? t("speaker.partPlaceholder") }}</span>
      <span class="time" :style="{ color: heat }">{{ display }}</span>
    </p>
    <ClockBar
      compact
      :progress-pct="clock.progress_pct"
      :remaining-ms="clock.remaining_ms"
    />
    <div class="row">
      <input
        v-model="title"
        type="text"
        maxlength="80"
        :aria-label="t('timer.title')"
        :placeholder="t('timer.title')"
      />
      <input
        v-model.number="minutes"
        type="number"
        min="1"
        max="180"
        :aria-label="t('timer.minutes')"
      />
      <button type="button" @click="arm">{{ t("timer.arm") }}</button>
    </div>
    <div class="row">
      <button type="button" class="primary" :disabled="!canStart" @click="start">
        {{ t("timer.start") }}
      </button>
      <button type="button" :disabled="!canPause" @click="pause">
        {{ t("timer.pause") }}
      </button>
      <button type="button" :disabled="!canFinish" @click="finish">
        {{ t("timer.finish") }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.timer-controls {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.status {
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
  margin: 0;
}

.part {
  font-size: 12px;
  color: var(--jp-muted);
}

.time {
  font-variant-numeric: tabular-nums;
  font-size: 1.35rem;
  font-weight: 650;
  font-family: var(--jp-mono);
}

.row {
  display: flex;
  gap: 0.35rem;
}

input,
button {
  border: 1px solid var(--jp-border);
  background: var(--jp-bg);
  border-radius: 4px;
  padding: var(--jp-pad);
  min-width: 0;
}

input[type="number"] {
  width: 3.6rem;
  flex: 0 0 3.6rem;
}

input[type="text"] {
  flex: 1;
}

.primary {
  border: 0;
  background: var(--jp-accent);
  color: var(--jp-accent-fg);
}

button:disabled {
  opacity: 0.55;
}
</style>
