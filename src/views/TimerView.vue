<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";

import TimerControls from "@/components/TimerControls.vue";
import { errorMessage } from "@/composables/errors";
import { useTimerStore } from "@/stores/timer";
import { useUiStore } from "@/stores/ui";
import { useWeekStore } from "@/stores/week";
import type { MeetingPart, MeetingWeek } from "@/types/dto";

const { t } = useI18n();
const week = useWeekStore();
const timer = useTimerStore();
const ui = useUiStore();

const meetings = computed(() => [
  { key: "midweek" as const, week: week.bundle.midweek, labelKey: "settings.midweek" },
  { key: "weekend" as const, week: week.bundle.weekend, labelKey: "settings.weekend" },
]);

onMounted(async () => {
  try {
    await week.hydrate();
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
});

async function run(action: () => Promise<void>): Promise<void> {
  try {
    await action();
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
}

async function armPart(part: MeetingPart): Promise<void> {
  const minutes = part.minutes && part.minutes > 0 ? part.minutes : 10;
  await run(() => timer.arm(part.title, minutes));
}

function isEmpty(meeting: MeetingWeek): boolean {
  return meeting.parts.length === 0;
}
</script>

<template>
  <section class="timer-view">
    <header class="toolbar">
      <h1>{{ t("nav.timer") }}</h1>
      <div class="tabs">
        <button
          type="button"
          :class="{ active: week.which === 'this' }"
          :disabled="week.busy"
          @click="run(() => week.setWhich('this'))"
        >
          {{ t("media.thisWeek") }}
        </button>
        <button
          type="button"
          :class="{ active: week.which === 'next' }"
          :disabled="week.busy"
          @click="run(() => week.setWhich('next'))"
        >
          {{ t("media.nextWeek") }}
        </button>
      </div>
      <button type="button" class="primary" :disabled="week.busy" @click="run(() => week.fetchWeek())">
        {{ t("timer.fetchGuide") }}
      </button>
    </header>
    <p v-if="week.bundle.monday" class="monday">{{ week.bundle.monday }}</p>
    <p v-if="week.progress" class="progress">
      {{ week.progress.phase }} {{ week.progress.done }}/{{ week.progress.total }}
    </p>

    <div class="layout">
      <div class="outline">
        <section v-for="meeting in meetings" :key="meeting.key">
          <h2>{{ t(meeting.labelKey) }}</h2>
          <p v-if="isEmpty(meeting.week)" class="empty">{{ t("timer.noGuide") }}</p>
          <button
            v-for="part in meeting.week.parts"
            :key="part.id"
            type="button"
            class="part"
            @click="armPart(part)"
          >
            <span>{{ part.title }}</span>
            <span v-if="part.minutes">{{ t("media.minutes", { n: part.minutes }) }}</span>
          </button>
        </section>
      </div>
      <TimerControls />
    </div>
  </section>
</template>

<style scoped>
.timer-view {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  height: 100%;
  min-height: 0;
}

.toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
  align-items: center;
}

h1 {
  margin: 0;
  font-size: 1.1rem;
}

.tabs {
  display: flex;
  gap: 0.3rem;
}

.monday,
.progress,
.empty {
  margin: 0;
  color: var(--jp-muted);
}

.layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(16rem, 22rem);
  gap: 0.8rem;
  min-height: 0;
  flex: 1;
}

.outline {
  overflow: auto;
}

h2 {
  margin: 0.4rem 0;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--jp-muted);
}

.part,
button {
  border: 1px solid var(--jp-border);
  background: var(--jp-bg);
  border-radius: 4px;
  padding: var(--jp-pad);
}

.part {
  display: flex;
  justify-content: space-between;
  width: 100%;
  margin-top: 0.25rem;
  text-align: start;
}

.primary,
button.active {
  border: 0;
  background: var(--jp-accent);
  color: var(--jp-accent-fg);
}

button:disabled {
  opacity: 0.55;
}
</style>
