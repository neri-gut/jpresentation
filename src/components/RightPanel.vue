<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";

import TimerControls from "@/components/TimerControls.vue";
import { formatRemaining } from "@/composables/clockDisplay";
import { errorMessage } from "@/composables/errors";
import { useExplorerStore } from "@/stores/explorer";
import { useOutputStore } from "@/stores/output";
import { useProfileStore } from "@/stores/profile";
import { useTimerStore } from "@/stores/timer";
import { useUiStore } from "@/stores/ui";
import { useWeekStore } from "@/stores/week";

const { t } = useI18n();
const ui = useUiStore();
const profiles = useProfileStore();
const timer = useTimerStore();
const explorer = useExplorerStore();
const output = useOutputStore();
const week = useWeekStore();
const { panel } = storeToRefs(ui);

const outlineParts = computed(() => {
  if (week.bundle.midweek.parts.length) {
    return week.bundle.midweek.parts;
  }
  return week.bundle.weekend.parts;
});

const placeholders = [
  { id: "songs", labelKey: "panel.songs" },
  { id: "bible", labelKey: "panel.bible" },
] as const;

const width = computed(() => (panel.value.collapsed ? 44 : panel.value.width));
const collapsedTime = computed(() => formatRemaining(timer.clock.remaining_ms));
const live = computed(() => output.stage.kind !== "none");
const canShow = computed(() => explorer.cue !== null);
const stageTitle = computed(() => {
  if (live.value) {
    return output.stage.name ?? t("panel.live");
  }
  return explorer.cue?.name ?? t("panel.cueEmpty");
});

onMounted(() => {
  void week.hydrate().catch((err) => {
    ui.showToast(errorMessage(err, t));
  });
});

async function toggle(): Promise<void> {
  if (!profiles.current) {
    return;
  }
  try {
    await ui.setPanel(profiles.current.id, {
      ...panel.value,
      collapsed: !panel.value.collapsed,
    });
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
}

async function showOnScreens(): Promise<void> {
  const cue = explorer.cue;
  if (!cue) {
    return;
  }
  try {
    await explorer.openOnStage(cue.path);
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
}

async function hideScreens(): Promise<void> {
  try {
    await explorer.closeStage();
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
}
</script>

<template>
  <aside class="panel" :class="{ collapsed: panel.collapsed }" :style="{ width: `${width}px` }">
    <button
      type="button"
      class="collapse"
      :title="panel.collapsed ? t('panel.expand') : t('panel.collapse')"
      @click="toggle"
    >
      {{ panel.collapsed ? "«" : "»" }}
    </button>
    <section class="block stage">
      <h2>{{ t("panel.media") }}</h2>
      <template v-if="!panel.collapsed">
        <p class="caption">{{ stageTitle }}</p>
        <p class="state">{{ live ? t("panel.live") : t("panel.cued") }}</p>
        <img
          v-if="explorer.cue?.previewUrl"
          class="thumb"
          :src="explorer.cue.previewUrl"
          alt=""
        />
        <p v-else-if="explorer.cue?.kind === 'video'" class="empty">{{ t("media.kind.video") }}</p>
        <div class="row">
          <button type="button" class="primary" :disabled="!canShow" @click="showOnScreens">
            {{ t("panel.show") }}
          </button>
          <button type="button" :disabled="!live" @click="hideScreens">
            {{ t("panel.hide") }}
          </button>
        </div>
      </template>
    </section>
    <section v-for="block in placeholders" :key="block.id" class="block">
      <h2>{{ t(block.labelKey) }}</h2>
      <p v-if="!panel.collapsed">{{ t("panel.placeholder") }}</p>
    </section>
    <section class="block timer">
      <h2>{{ t("panel.timer") }}</h2>
      <TimerControls v-if="!panel.collapsed" :parts="outlineParts" />
      <p v-else class="collapsed-time" :data-hue="timer.clock.hue">{{ collapsedTime }}</p>
    </section>
  </aside>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  border-left: 1px solid var(--jp-border);
  background: var(--jp-bg-elev);
  padding: 0.45rem;
  overflow: auto;
}

.collapsed .block h2 {
  writing-mode: vertical-rl;
  transform: rotate(180deg);
  font-size: 11px;
}

.collapsed .block p:not(.collapsed-time) {
  display: none;
}

.collapse {
  align-self: flex-end;
  border: 1px solid var(--jp-border);
  background: var(--jp-bg);
  border-radius: 4px;
  padding: 0.1rem 0.35rem;
}

.block {
  border: 1px solid var(--jp-border);
  border-radius: var(--jp-radius);
  padding: 0.45rem;
}

.block h2 {
  margin: 0 0 0.25rem;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--jp-muted);
}

.block p,
.caption,
.state,
.empty {
  margin: 0;
  color: var(--jp-muted);
  font-size: 12px;
}

.thumb {
  display: block;
  width: 100%;
  max-height: 7rem;
  object-fit: contain;
  margin: 0.35rem 0;
  background: #111;
}

.row {
  display: flex;
  gap: 0.35rem;
  margin-top: 0.35rem;
}

button {
  border: 1px solid var(--jp-border);
  background: var(--jp-bg);
  border-radius: 4px;
  padding: var(--jp-pad);
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

.collapsed-time {
  margin: 0.35rem 0 0;
  writing-mode: vertical-rl;
  transform: rotate(180deg);
  font-variant-numeric: tabular-nums;
  font-family: var(--jp-mono);
  font-weight: 650;
  color: var(--jp-fg);
}

.collapsed-time[data-hue="green"] {
  color: var(--jp-ok);
}

.collapsed-time[data-hue="amber"] {
  color: var(--jp-warn);
}

.collapsed-time[data-hue="red"] {
  color: var(--jp-danger);
}
</style>
