<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import { useProfileStore } from "@/stores/profile";
import { useUiStore } from "@/stores/ui";
import type { AppearanceSetting, AppErrorDto, SurfacesSetting } from "@/types/dto";

const emit = defineEmits<{ close: [] }>();
const { t } = useI18n();
const profiles = useProfileStore();
const ui = useUiStore();
const { appearance, surfaces, monitors } = storeToRefs(ui);

const newName = ref("");
const busy = ref(false);

const themes: { id: AppearanceSetting["theme"]; labelKey: string }[] = [
  { id: "system", labelKey: "settings.themeSystem" },
  { id: "light", labelKey: "settings.themeLight" },
  { id: "dark", labelKey: "settings.themeDark" },
];

const audienceOptions = computed(() => monitors.value);
const speakerOptions = computed(() =>
  monitors.value.filter((item) => item.id !== surfaces.value.audience_monitor_id),
);

function errorMessage(err: unknown): string {
  if (err && typeof err === "object" && "message" in err) {
    return (err as AppErrorDto).message;
  }
  return t("errors.invokeFailed");
}

async function createProfile(): Promise<void> {
  if (!newName.value.trim()) {
    return;
  }
  busy.value = true;
  try {
    await profiles.create(newName.value);
    if (profiles.current) {
      await ui.hydrate(profiles.current.id);
    }
    newName.value = "";
  } catch (err) {
    ui.showToast(errorMessage(err));
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
    ui.showToast(errorMessage(err));
  } finally {
    busy.value = false;
  }
}

async function changeTheme(theme: AppearanceSetting["theme"]): Promise<void> {
  if (!profiles.current) {
    return;
  }
  try {
    await ui.setAppearance(profiles.current.id, { ...appearance.value, theme });
  } catch (err) {
    ui.showToast(errorMessage(err));
  }
}

async function saveSurfaces(next: SurfacesSetting): Promise<void> {
  if (!profiles.current) {
    return;
  }
  try {
    await ui.setSurfaces(profiles.current.id, next);
  } catch (err) {
    ui.showToast(errorMessage(err));
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
            v-model="newName"
            :placeholder="t('settings.profileName')"
            :aria-label="t('settings.profileName')"
          />
          <button type="button" class="primary" :disabled="busy" @click="createProfile">
            {{ t("settings.create") }}
          </button>
        </div>
      </section>

      <section>
        <h2>{{ t("settings.theme") }}</h2>
        <div class="row">
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
      </section>

      <button type="button" class="primary" @click="emit('close')">
        {{ t("settings.close") }}
      </button>
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

select,
input[type="text"],
input:not([type]) {
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
  border-radius: 4px;
  padding: var(--jp-pad);
}

.primary:disabled {
  opacity: 0.6;
}
</style>
