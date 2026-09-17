<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import { errorMessage } from "@/composables/errors";
import { useTimerStore } from "@/stores/timer";
import { useUiStore } from "@/stores/ui";
import { useWeekStore } from "@/stores/week";
import type { MeetingKind, MeetingPart, MeetingWeek } from "@/types/dto";

const { t } = useI18n();
const week = useWeekStore();
const timer = useTimerStore();
const ui = useUiStore();

const selectedPartId = ref<string | null>(null);
const draftTitle = ref("");
const draftMinutes = ref(10);
const draftTone = ref("other");
const templateId = ref("sys:midweek");
const newTemplateName = ref("");

const meetings = computed(() => [
  { key: "midweek" as const, week: week.bundle.midweek, labelKey: "settings.midweek" },
  { key: "weekend" as const, week: week.bundle.weekend, labelKey: "settings.weekend" },
]);

const selectedMeeting = computed({
  get: () => week.selectedMeeting,
  set: (value: MeetingKind) => {
    week.selectedMeeting = value;
  },
});

const activeWeek = computed(() =>
  selectedMeeting.value === "midweek" ? week.bundle.midweek : week.bundle.weekend,
);

function visibleParts(meeting: MeetingWeek): MeetingPart[] {
  return meeting.parts.filter((part) => part.tone !== "song");
}

const selectedTemplate = computed(() =>
  week.templates.find((item) => item.id === templateId.value),
);

onMounted(async () => {
  try {
    await week.hydrate();
    if (week.templates[0]) {
      templateId.value = week.templates[0].id;
    }
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

function fillDraft(part: MeetingPart): void {
  selectedPartId.value = part.id;
  week.selectedPartId = part.id;
  draftTitle.value = part.title;
  draftMinutes.value = part.minutes && part.minutes > 0 ? part.minutes : 10;
  draftTone.value = part.tone === "song" ? "other" : (part.tone ?? "other");
}

function armPart(part: MeetingPart): Promise<void> {
  fillDraft(part);
  const minutes = part.minutes && part.minutes > 0 ? part.minutes : 10;
  return run(() => timer.arm(part.title, minutes));
}

watch(
  () => week.selectedPartId,
  (id) => {
    if (!id || id === selectedPartId.value) {
      return;
    }
    const part = activeWeek.value.parts.find((item) => item.id === id);
    if (part) {
      fillDraft(part);
    }
  },
);

function isEmpty(meeting: MeetingWeek): boolean {
  return visibleParts(meeting).length === 0;
}

function persistParts(parts: MeetingPart[]): Promise<void> {
  return run(() => week.setParts(selectedMeeting.value, parts));
}

function addRow(): Promise<void> {
  const title = draftTitle.value.trim();
  if (!title) {
    return Promise.resolve();
  }
  const next: MeetingPart[] = [
    ...activeWeek.value.parts,
    {
      id: `user-${Date.now()}`,
      title,
      minutes: draftMinutes.value,
      tone: draftTone.value,
      items: [],
    },
  ];
  return persistParts(next);
}

function modifyRow(): Promise<void> {
  const id = selectedPartId.value;
  if (!id) {
    return addRow();
  }
  const title = draftTitle.value.trim();
  if (!title) {
    return Promise.resolve();
  }
  const next = activeWeek.value.parts.map((part) =>
    part.id === id
      ? { ...part, title, minutes: draftMinutes.value, tone: draftTone.value }
      : part,
  );
  return persistParts(next);
}

function deleteRow(): Promise<void> {
  const id = selectedPartId.value;
  if (!id) {
    return Promise.resolve();
  }
  const next = activeWeek.value.parts.filter((part) => part.id !== id);
  selectedPartId.value = null;
  return persistParts(next);
}

function applySelected(): Promise<void> {
  return run(() => week.applyTemplate(selectedMeeting.value, templateId.value));
}

function applyCircuit(): Promise<void> {
  const id =
    selectedMeeting.value === "weekend" ? "sys:circuit-weekend" : "sys:circuit-midweek";
  templateId.value = id;
  return run(() => week.applyTemplate(selectedMeeting.value, id));
}

function restore(): Promise<void> {
  return run(() => week.restore());
}

function saveAs(): Promise<void> {
  const name = newTemplateName.value.trim();
  if (!name) {
    return Promise.resolve();
  }
  const kind = selectedTemplate.value?.kind ?? selectedMeeting.value;
  const parts = activeWeek.value.parts.map((part) => ({
    ...part,
    items: [],
  }));
  return run(async () => {
    await week.saveTemplate(name, kind, parts);
    newTemplateName.value = "";
  });
}

function deleteSelectedTemplate(): Promise<void> {
  if (!selectedTemplate.value || selectedTemplate.value.source !== "user") {
    return Promise.resolve();
  }
  return run(async () => {
    await week.deleteTemplate(selectedTemplate.value!.id);
    templateId.value = week.templates[0]?.id ?? "sys:midweek";
  });
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
    <p class="monday">{{ t("timer.listHint") }}</p>

    <div class="actions">
      <button type="button" :disabled="week.busy" @click="applyCircuit">
        {{ t("timer.circuit") }}
      </button>
      <button type="button" :disabled="week.busy" @click="restore">
        {{ t("timer.restore") }}
      </button>
      <label class="field">
        <span>{{ t("timer.template") }}</span>
        <select v-model="templateId">
          <option v-for="item in week.templates" :key="item.id" :value="item.id">
            {{ item.name }}
          </option>
        </select>
      </label>
      <button type="button" :disabled="week.busy" @click="applySelected">
        {{ t("timer.apply") }}
      </button>
      <input
        v-model="newTemplateName"
        type="text"
        maxlength="80"
        :placeholder="t('timer.templateName')"
      />
      <button type="button" :disabled="week.busy || !newTemplateName.trim()" @click="saveAs">
        {{ t("timer.saveAs") }}
      </button>
      <button
        type="button"
        :disabled="week.busy || selectedTemplate?.source !== 'user'"
        @click="deleteSelectedTemplate"
      >
        {{ t("timer.deleteTemplate") }}
      </button>
    </div>

    <div class="outline">
      <section v-for="meeting in meetings" :key="meeting.key">
        <h2>
          <button
            type="button"
            class="meeting-tab"
            :class="{ active: selectedMeeting === meeting.key }"
            @click="selectedMeeting = meeting.key"
          >
            {{ t(meeting.labelKey) }}
          </button>
        </h2>
        <p v-if="isEmpty(meeting.week)" class="empty">{{ t("timer.noGuide") }}</p>
        <table v-else class="sheet">
          <thead>
            <tr>
              <th>{{ t("timer.colTitle") }}</th>
              <th>{{ t("timer.colMin") }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="part in visibleParts(meeting.week)"
              :key="part.id"
              :data-tone="part.tone ?? 'other'"
              :class="{
                current: timer.clock.title === part.title,
                picked: selectedPartId === part.id && selectedMeeting === meeting.key,
              }"
              @click="selectedMeeting = meeting.key; armPart(part)"
            >
              <td>{{ part.title }}</td>
              <td class="min">{{ part.minutes ?? "" }}</td>
            </tr>
          </tbody>
        </table>
      </section>

      <div class="editor">
        <select v-model="draftTone" :aria-label="t('timer.tone')">
          <option value="treasures">{{ t("timer.toneTreasures") }}</option>
          <option value="ayf">{{ t("timer.toneAyf") }}</option>
          <option value="living">{{ t("timer.toneLiving") }}</option>
          <option value="other">{{ t("timer.toneOther") }}</option>
        </select>
        <input
          v-model="draftTitle"
          type="text"
          maxlength="80"
          :placeholder="t('timer.title')"
        />
        <input
          v-model.number="draftMinutes"
          type="number"
          min="1"
          max="180"
          :aria-label="t('timer.minutes')"
        />
        <button type="button" @click="addRow">{{ t("timer.addRow") }}</button>
        <button type="button" @click="modifyRow">{{ t("timer.modifyRow") }}</button>
        <button type="button" :disabled="!selectedPartId" @click="deleteRow">
          {{ t("timer.deleteRow") }}
        </button>
      </div>
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

.toolbar,
.actions,
.editor {
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

.outline {
  overflow: auto;
  min-height: 0;
  flex: 1;
}

h2 {
  margin: 0.4rem 0;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--jp-muted);
}

.meeting-tab {
  text-transform: uppercase;
  letter-spacing: 0.04em;
  font-size: 12px;
}

.meeting-tab.active {
  border-color: var(--jp-accent);
  color: var(--jp-accent);
}

.sheet {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.sheet th,
.sheet td {
  border-bottom: 1px solid var(--jp-border);
  padding: 0.28rem 0.4rem;
  text-align: start;
}

.sheet th {
  color: var(--jp-muted);
  font-weight: 600;
  font-size: 11px;
  text-transform: uppercase;
}

.min {
  width: 3.5rem;
  font-variant-numeric: tabular-nums;
}

.sheet tbody tr {
  cursor: pointer;
}

.sheet tbody tr:hover {
  background: color-mix(in srgb, var(--jp-accent) 8%, transparent);
}

.sheet tbody tr.current,
.sheet tbody tr.picked {
  background: color-mix(in srgb, var(--jp-accent) 16%, transparent);
}

.sheet tr[data-tone="treasures"] td:first-child {
  color: #1d4ed8;
}

.sheet tr[data-tone="ayf"] td:first-child {
  color: #b45309;
}

.sheet tr[data-tone="living"] td:first-child {
  color: #9f1239;
}

.sheet tr[data-tone="song"] td:first-child {
  color: var(--jp-muted);
}

.field {
  display: flex;
  gap: 0.3rem;
  align-items: center;
  font-size: 12px;
  color: var(--jp-muted);
}

.editor input[type="text"] {
  flex: 1;
  min-width: 10rem;
}

.editor input[type="number"] {
  width: 3.6rem;
}

input,
select,
button {
  border: 1px solid var(--jp-border);
  background: var(--jp-bg);
  border-radius: 4px;
  padding: var(--jp-pad);
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
