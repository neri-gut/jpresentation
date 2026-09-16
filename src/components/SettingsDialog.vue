<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useI18n } from "vue-i18n";

import { errorMessage } from "@/composables/errors";
import { invokeCommand } from "@/composables/invoke";
import { useProfileStore } from "@/stores/profile";
import { useUiStore } from "@/stores/ui";
import type {
  AppearanceSetting,
  ContentLanguageDto,
  MeetingScheduleSetting,
  SurfacesSetting,
} from "@/types/dto";

const emit = defineEmits<{ close: [] }>();
const { t } = useI18n();
const profiles = useProfileStore();
const ui = useUiStore();
const { appearance, surfaces, schedule, monitors } = storeToRefs(ui);

const newName = ref("");
const renameName = ref(profiles.current?.name ?? "");
const busy = ref(false);
const confirmDelete = ref(false);
const contentFilter = ref("");
const languages = ref<ContentLanguageDto[]>([]);

const themes: { id: AppearanceSetting["theme"]; labelKey: string }[] = [
  { id: "system", labelKey: "settings.themeSystem" },
  { id: "light", labelKey: "settings.themeLight" },
  { id: "dark", labelKey: "settings.themeDark" },
];

const accents: AppearanceSetting["accent"][] = ["blue", "teal", "violet", "amber"];

const densities: { id: AppearanceSetting["density"]; labelKey: string }[] = [
  { id: "compact", labelKey: "settings.densityCompact" },
  { id: "comfortable", labelKey: "settings.densityComfortable" },
];

const weekdays = [1, 2, 3, 4, 5, 6, 7] as const;

const audienceOptions = computed(() => monitors.value);
const speakerOptions = computed(() =>
  monitors.value.filter((item) => item.id !== surfaces.value.audience_monitor_id),
);

const filteredLanguages = computed(() => {
  const query = contentFilter.value.trim().toLowerCase();
  const current = profiles.current?.content_locale;
  const list = query
    ? languages.value.filter(
        (item) =>
          item.name.toLowerCase().includes(query) ||
          item.langwritten.toLowerCase().includes(query),
      )
    : languages.value;
  if (current && !list.some((item) => item.langwritten === current)) {
    const extra = languages.value.find((item) => item.langwritten === current);
    return extra ? [extra, ...list] : list;
  }
  return list;
});

watch(
  () => profiles.current?.id,
  () => {
    renameName.value = profiles.current?.name ?? "";
    confirmDelete.value = false;
  },
);

onMounted(async () => {
  try {
    languages.value = await invokeCommand<ContentLanguageDto[]>("content_languages_list");
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
});

async function createProfile(): Promise<void> {
  if (!newName.value.trim()) {
    return;
  }
  busy.value = true;
  try {
    await profiles.create(newName.value);
    if (profiles.current) {
      await ui.hydrate(profiles.current.id);
      renameName.value = profiles.current.name;
    }
    newName.value = "";
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  } finally {
    busy.value = false;
  }
}

async function selectProfile(id: string): Promise<void> {
  busy.value = true;
  try {
    await profiles.select(id);
    if (profiles.current) {
      await ui.hydrate(profiles.current.id);
    }
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  } finally {
    busy.value = false;
  }
}

async function renameProfile(): Promise<void> {
  if (!profiles.current || !renameName.value.trim()) {
    return;
  }
  busy.value = true;
  try {
    await profiles.update({ id: profiles.current.id, name: renameName.value });
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  } finally {
    busy.value = false;
  }
}

async function duplicateProfile(): Promise<void> {
  if (!profiles.current) {
    return;
  }
  busy.value = true;
  try {
    await profiles.duplicate(profiles.current.id);
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  } finally {
    busy.value = false;
  }
}

async function deleteProfile(): Promise<void> {
  if (!profiles.current) {
    return;
  }
  busy.value = true;
  try {
    const remaining = await profiles.remove(profiles.current.id);
    await ui.hydrate(remaining.id);
    confirmDelete.value = false;
  } catch (err) {
    ui.showToast(errorMessage(err, t));
    confirmDelete.value = false;
  } finally {
    busy.value = false;
  }
}

async function changeContent(event: Event): Promise<void> {
  if (!profiles.current) {
    return;
  }
  const value = (event.target as HTMLSelectElement).value;
  try {
    await profiles.update({ id: profiles.current.id, content_locale: value });
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
}

async function changeTheme(theme: AppearanceSetting["theme"]): Promise<void> {
  if (!profiles.current) {
    return;
  }
  try {
    await ui.setAppearance(profiles.current.id, { ...appearance.value, theme });
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
}

async function changeAccent(accent: AppearanceSetting["accent"]): Promise<void> {
  if (!profiles.current) {
    return;
  }
  try {
    await ui.setAppearance(profiles.current.id, { ...appearance.value, accent });
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
}

async function changeDensity(density: AppearanceSetting["density"]): Promise<void> {
  if (!profiles.current) {
    return;
  }
  try {
    await ui.setAppearance(profiles.current.id, { ...appearance.value, density });
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
}

async function saveSchedule(next: MeetingScheduleSetting): Promise<void> {
  if (!profiles.current) {
    return;
  }
  try {
    await ui.setSchedule(profiles.current.id, next);
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
}

function onWeekday(which: "midweek" | "weekend", event: Event): void {
  const weekday = Number((event.target as HTMLSelectElement).value);
  if (which === "midweek") {
    void saveSchedule({ ...schedule.value, midweek_weekday: weekday });
  } else {
    void saveSchedule({ ...schedule.value, weekend_weekday: weekday });
  }
}

function toHHmm(value: string): string {
  return value.slice(0, 5);
}

function onTime(which: "midweek" | "weekend", event: Event): void {
  const time = toHHmm((event.target as HTMLInputElement).value);
  if (which === "midweek") {
    void saveSchedule({ ...schedule.value, midweek_time: time });
  } else {
    void saveSchedule({ ...schedule.value, weekend_time: time });
  }
}

async function saveSurfaces(next: SurfacesSetting): Promise<void> {
  if (!profiles.current) {
    return;
  }
  try {
    await ui.setSurfaces(profiles.current.id, next);
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
}

function onAudienceChange(event: Event): void {
  const value = (event.target as HTMLSelectElement).value;
  void saveSurfaces({
    ...surfaces.value,
    audience_monitor_id: value === "" ? null : value,
  });
}

function onSpeakerChange(event: Event): void {
  const value = (event.target as HTMLSelectElement).value;
  void saveSurfaces({
    ...surfaces.value,
    speaker_monitor_id: value === "" ? null : value,
  });
}

function onUseSpeaker(event: Event): void {
  const checked = (event.target as HTMLInputElement).checked;
  void saveSurfaces({
    ...surfaces.value,
    use_speaker: checked,
    speaker_monitor_id: checked ? surfaces.value.speaker_monitor_id : null,
  });
}

async function identifyMonitors(): Promise<void> {
  try {
    await invokeCommand("monitors_identify");
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
}
</script>

<template>
  <div class="overlay" role="presentation" @click.self="emit('close')">
    <div class="dialog" role="dialog" :aria-label="t('settings.title')">
      <h1>{{ t("settings.title") }}</h1>

      <section>
        <h2>{{ t("settings.profiles") }}</h2>
        <select
          :value="profiles.current?.id ?? ''"
          :disabled="busy"
          @change="selectProfile(($event.target as HTMLSelectElement).value)"
        >
          <option v-for="profile in profiles.profiles" :key="profile.id" :value="profile.id">
            {{ profile.name }}
          </option>
        </select>
        <div class="row">
          <input
            v-model="renameName"
            :aria-label="t('settings.profileName')"
          />
          <button type="button" :disabled="busy" @click="renameProfile">
            {{ t("settings.rename") }}
          </button>
        </div>
        <div class="row">
          <input
            v-model="newName"
            :placeholder="t('settings.profileName')"
            :aria-label="t('settings.createProfile')"
          />
          <button type="button" class="primary" :disabled="busy" @click="createProfile">
            {{ t("settings.create") }}
          </button>
        </div>
        <div class="row">
          <button type="button" :disabled="busy" @click="duplicateProfile">
            {{ t("settings.duplicate") }}
          </button>
          <button type="button" class="danger" :disabled="busy" @click="confirmDelete = true">
            {{ t("settings.delete") }}
          </button>
        </div>
      </section>

      <section>
        <h2>{{ t("settings.contentLanguage") }}</h2>
        <input
          v-model="contentFilter"
          :placeholder="t('settings.contentFilter')"
          :aria-label="t('settings.contentFilter')"
        />
        <select
          :value="profiles.current?.content_locale ?? ''"
          :disabled="busy"
          @change="changeContent"
        >
          <option
            v-for="lang in filteredLanguages"
            :key="lang.langwritten"
            :value="lang.langwritten"
          >
            {{ lang.name }} ({{ lang.langwritten }})
          </option>
        </select>
      </section>

      <section>
        <h2>{{ t("settings.meetings") }}</h2>
        <label>
          {{ t("settings.midweek") }}
          <div class="row">
            <select :value="schedule.midweek_weekday" @change="onWeekday('midweek', $event)">
              <option v-for="day in weekdays" :key="`mw-${day}`" :value="day">
                {{ t(`weekday.${day}`) }}
              </option>
            </select>
            <input
              type="time"
              :value="schedule.midweek_time"
              @change="onTime('midweek', $event)"
            />
          </div>
        </label>
        <label>
          {{ t("settings.weekend") }}
          <div class="row">
            <select :value="schedule.weekend_weekday" @change="onWeekday('weekend', $event)">
              <option v-for="day in weekdays" :key="`we-${day}`" :value="day">
                {{ t(`weekday.${day}`) }}
              </option>
            </select>
            <input
              type="time"
              :value="schedule.weekend_time"
              @change="onTime('weekend', $event)"
            />
          </div>
        </label>
      </section>

      <section>
        <h2>{{ t("settings.theme") }}</h2>
        <div class="row wrap">
          <label v-for="theme in themes" :key="theme.id">
            <input
              type="radio"
              name="theme"
              :value="theme.id"
              :checked="appearance.theme === theme.id"
              @change="changeTheme(theme.id)"
            />
            {{ t(theme.labelKey) }}
          </label>
        </div>
        <h2>{{ t("settings.accent") }}</h2>
        <div class="row wrap">
          <button
            v-for="accent in accents"
            :key="accent"
            type="button"
            class="swatch"
            :data-accent="accent"
            :aria-pressed="appearance.accent === accent"
            :aria-label="accent"
            @click="changeAccent(accent)"
          />
        </div>
        <h2>{{ t("settings.density") }}</h2>
        <div class="row wrap">
          <label v-for="density in densities" :key="density.id">
            <input
              type="radio"
              name="density"
              :value="density.id"
              :checked="appearance.density === density.id"
              @change="changeDensity(density.id)"
            />
            {{ t(density.labelKey) }}
          </label>
        </div>
      </section>

      <section>
        <h2>{{ t("settings.monitors") }}</h2>
        <p v-if="monitors.length === 0">{{ t("settings.noMonitors") }}</p>
        <label>
          {{ t("settings.audienceMonitor") }}
          <select :value="surfaces.audience_monitor_id ?? ''" @change="onAudienceChange">
            <option value="">{{ t("settings.previewMonitor") }}</option>
            <option v-for="monitor in audienceOptions" :key="monitor.id" :value="monitor.id">
              {{ monitor.name }} ({{ monitor.width }}×{{ monitor.height }})
            </option>
          </select>
        </label>
        <label class="check">
          <input
            type="checkbox"
            :checked="surfaces.use_speaker"
            @change="onUseSpeaker"
          />
          {{ t("settings.useSpeaker") }}
        </label>
        <label>
          {{ t("settings.speakerMonitor") }}
          <select
            :value="surfaces.speaker_monitor_id ?? ''"
            :disabled="!surfaces.use_speaker"
            @change="onSpeakerChange"
          >
            <option value="">{{ t("settings.previewMonitor") }}</option>
            <option v-for="monitor in speakerOptions" :key="monitor.id" :value="monitor.id">
              {{ monitor.name }} ({{ monitor.width }}×{{ monitor.height }})
            </option>
          </select>
        </label>
        <button type="button" :disabled="monitors.length === 0" @click="identifyMonitors">
          {{ t("settings.identify") }}
        </button>
      </section>

      <button type="button" class="primary" @click="emit('close')">
        {{ t("settings.close") }}
      </button>
    </div>

    <div
      v-if="confirmDelete"
      class="overlay nested"
      role="presentation"
      @click.self="confirmDelete = false"
    >
      <div class="dialog small" role="alertdialog" :aria-label="t('settings.delete')">
        <p>{{ t("settings.confirmDelete", { name: profiles.current?.name ?? "" }) }}</p>
        <div class="row">
          <button type="button" :disabled="busy" @click="confirmDelete = false">
            {{ t("common.cancel") }}
          </button>
          <button type="button" class="danger" :disabled="busy" @click="deleteProfile">
            {{ t("common.ok") }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgb(10 12 16 / 45%);
  display: grid;
  place-items: center;
  z-index: 40;
}

.overlay.nested {
  z-index: 50;
}

.dialog {
  width: min(32rem, calc(100% - 2rem));
  max-height: calc(100% - 2rem);
  overflow: auto;
  background: var(--jp-bg-elev);
  color: var(--jp-fg);
  border: 1px solid var(--jp-border);
  border-radius: var(--jp-radius);
  box-shadow: var(--jp-shadow);
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.dialog.small {
  width: min(24rem, calc(100% - 2rem));
}

h1 {
  margin: 0;
  font-size: 1.1rem;
}

h2 {
  margin: 0 0 0.4rem;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--jp-muted);
}

section,
label {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.row,
.check {
  flex-direction: row;
  align-items: center;
  gap: 0.5rem;
}

.row {
  display: flex;
}

.row.wrap {
  flex-wrap: wrap;
}

select,
input[type="text"],
input[type="time"],
input:not([type]) {
  border: 1px solid var(--jp-border);
  background: var(--jp-bg);
  border-radius: 4px;
  padding: var(--jp-pad);
  min-width: 0;
  flex: 1;
}

button {
  border: 1px solid var(--jp-border);
  background: var(--jp-bg);
  border-radius: 4px;
  padding: var(--jp-pad);
}

.primary {
  align-self: flex-start;
  border: 0;
  background: var(--jp-accent);
  color: var(--jp-accent-fg);
}

.primary:disabled,
button:disabled {
  opacity: 0.6;
}

.danger {
  border-color: var(--jp-danger);
  color: var(--jp-danger);
}

.swatch {
  width: 1.6rem;
  height: 1.6rem;
  border-radius: 999px;
  padding: 0;
  border: 2px solid var(--jp-border);
}

.swatch[aria-pressed="true"] {
  border-color: var(--jp-fg);
}

.swatch[data-accent="blue"] {
  background: #1d4ed8;
}

.swatch[data-accent="teal"] {
  background: #0f766e;
}

.swatch[data-accent="violet"] {
  background: #6d28d9;
}

.swatch[data-accent="amber"] {
  background: #b45309;
}

p {
  margin: 0;
}
</style>
