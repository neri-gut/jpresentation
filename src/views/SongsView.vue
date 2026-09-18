<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import { errorMessage } from "@/composables/errors";
import { useHymnalStore } from "@/stores/hymnal";
import { useUiStore } from "@/stores/ui";
import { useWeekStore } from "@/stores/week";
import type { HymnalSongDto } from "@/types/dto";

const { t } = useI18n();
const hymnal = useHymnalStore();
const ui = useUiStore();
const week = useWeekStore();

const searchQuery = ref("");
const downloadingSingle = ref<number | null>(null);

const filteredSongs = computed(() => {
  const query = searchQuery.value.trim().toLowerCase();
  if (!query) {
    return hymnal.songs;
  }
  return hymnal.songs.filter((song) => {
    return (
      song.track.toString() === query ||
      song.title.toLowerCase().includes(query)
    );
  });
});

const startTrack = computed({
  get: () => hymnal.slots.start ?? null,
  set: (val: number | null) => hymnal.setSlot("start", val ? Number(val) : null),
});

const middleTrack = computed({
  get: () => hymnal.slots.middle ?? null,
  set: (val: number | null) => hymnal.setSlot("middle", val ? Number(val) : null),
});

const endTrack = computed({
  get: () => hymnal.slots.end ?? null,
  set: (val: number | null) => hymnal.setSlot("end", val ? Number(val) : null),
});

function findSong(track: number | null | undefined): HymnalSongDto | undefined {
  if (!track) return undefined;
  return hymnal.songs.find((s) => s.track === track);
}

function songLabel(track: number | null | undefined): string {
  if (!track) return "—";
  const s = findSong(track);
  return s ? s.title : `${t("nav.songs")} ${track}`;
}

async function handlePlay(track: number): Promise<void> {
  try {
    await hymnal.playSong(track);
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
}

async function handleDownloadSingle(track: number): Promise<void> {
  downloadingSingle.value = track;
  try {
    await hymnal.downloadSong(track);
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  } finally {
    downloadingSingle.value = null;
  }
}

async function handleDownloadAll(): Promise<void> {
  try {
    await hymnal.downloadAll();
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
}

async function handleCancelDownload(): Promise<void> {
  try {
    await hymnal.cancelDownload();
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
}

async function handleRefreshCatalog(): Promise<void> {
  try {
    await hymnal.refreshCatalog();
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
}

function syncSlotsFromWeek(): void {
  const meeting =
    week.selectedMeeting === "weekend" ? week.bundle.weekend : week.bundle.midweek;
  const songItems = (meeting.media ?? []).filter((m) => m.media_kind === "song");
  const tracks: number[] = [];
  for (const item of songItems) {
    if ("track" in item.media_ref) {
      tracks.push(item.media_ref.track);
    }
  }
  if (tracks.length >= 1 && hymnal.slots.start == null) {
    hymnal.setSlot("start", tracks[0] ?? null);
  }
  if (tracks.length >= 2 && hymnal.slots.middle == null) {
    hymnal.setSlot("middle", tracks[1] ?? null);
  }
  if (tracks.length >= 3 && hymnal.slots.end == null) {
    hymnal.setSlot("end", tracks[2] ?? null);
  }
}

watch(
  () => week.bundle,
  () => {
    syncSlotsFromWeek();
  },
  { deep: true },
);

onMounted(async () => {
  try {
    await hymnal.hydrate();
    syncSlotsFromWeek();
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
});
</script>

<template>
  <div class="songs-view">
    <!-- Left Column: Catalog List -->
    <section class="panel-left">
      <header class="search-header">
        <input
          v-model="searchQuery"
          type="search"
          class="search-input"
          :placeholder="t('songs.search')"
        />
        <button
          type="button"
          class="refresh-btn"
          :title="t('songs.refreshCatalog')"
          :disabled="hymnal.loading"
          @click="handleRefreshCatalog"
        >
          ↻
        </button>
      </header>

      <div class="song-list" role="listbox">
        <div
          v-for="song in filteredSongs"
          :key="song.track"
          class="song-row"
          :class="{
            selected: hymnal.selectedTrack === song.track,
            ready: song.status === 'ready',
          }"
          role="option"
          :aria-selected="hymnal.selectedTrack === song.track"
          @click="hymnal.selectedTrack = song.track"
          @dblclick="handlePlay(song.track)"
        >
          <span class="track-number">{{ song.track }}.</span>
          <span class="song-title">{{ song.title.replace(/^\d+\.\s*/, '') }}</span>
          <span v-if="song.duration_formatted" class="duration">
            ({{ song.duration_formatted }})
          </span>

          <div class="actions">
            <button
              v-if="song.status === 'ready'"
              type="button"
              class="icon-btn play-btn"
              :title="t('songs.playSong')"
              @click.stop="handlePlay(song.track)"
            >
              ▶
            </button>
            <button
              v-else
              type="button"
              class="icon-btn dl-btn"
              :title="t('songs.downloadSong')"
              :disabled="downloadingSingle === song.track || hymnal.downloading"
              @click.stop="handleDownloadSingle(song.track)"
            >
              {{ downloadingSingle === song.track ? "⏳" : "⬇" }}
            </button>
          </div>
        </div>

        <p v-if="filteredSongs.length === 0" class="empty-msg">
          {{ t("songs.empty") }}
        </p>
      </div>
    </section>

    <!-- Right Column: Meeting Slots & Actions (JMulti-01 layout) -->
    <section class="panel-right">
      <div class="slots-box">
        <fieldset class="slot-fieldset">
          <legend>{{ t("songs.slotStart") }}</legend>
          <div class="slot-row">
            <input
              v-model.number="startTrack"
              type="number"
              min="1"
              max="163"
              class="slot-spinner"
            />
            <button
              type="button"
              class="slot-action-btn"
              :disabled="!startTrack"
              @click="startTrack && handlePlay(startTrack)"
            >
              {{ songLabel(startTrack) }}
            </button>
          </div>
        </fieldset>

        <fieldset class="slot-fieldset">
          <legend>{{ t("songs.slotMiddle") }}</legend>
          <div class="slot-row">
            <input
              v-model.number="middleTrack"
              type="number"
              min="1"
              max="163"
              class="slot-spinner"
            />
            <button
              type="button"
              class="slot-action-btn"
              :disabled="!middleTrack"
              @click="middleTrack && handlePlay(middleTrack)"
            >
              {{ songLabel(middleTrack) }}
            </button>
          </div>
        </fieldset>

        <fieldset class="slot-fieldset">
          <legend>{{ t("songs.slotEnd") }}</legend>
          <div class="slot-row">
            <input
              v-model.number="endTrack"
              type="number"
              min="1"
              max="163"
              class="slot-spinner"
            />
            <button
              type="button"
              class="slot-action-btn"
              :disabled="!endTrack"
              @click="endTrack && handlePlay(endTrack)"
            >
              {{ songLabel(endTrack) }}
            </button>
          </div>
        </fieldset>
      </div>

      <!-- Bottom Downloads Section -->
      <div class="download-section">
        <div v-if="hymnal.downloading && hymnal.downloadProgress" class="progress-box">
          <div class="progress-info">
            <span class="progress-label">
              {{
                t("songs.downloading", {
                  done: hymnal.downloadProgress.done,
                  total: hymnal.downloadProgress.total,
                  label: hymnal.downloadProgress.label,
                })
              }}
            </span>
            <button type="button" class="cancel-btn" @click="handleCancelDownload">
              {{ t("songs.cancel") }}
            </button>
          </div>
          <progress
            class="progress-bar"
            :value="hymnal.downloadProgress.done"
            :max="hymnal.downloadProgress.total"
          />
        </div>

        <button
          v-else
          type="button"
          class="download-all-btn"
          :disabled="hymnal.downloading"
          @click="handleDownloadAll"
        >
          ⬇ {{ t("songs.downloadHymnal") }}
        </button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.songs-view {
  display: grid;
  grid-template-columns: 1fr 340px;
  gap: 1rem;
  height: 100%;
  min-height: 0;
}

.panel-left {
  display: flex;
  flex-direction: column;
  background: var(--jp-bg-elev);
  border: 1px solid var(--jp-border);
  border-radius: var(--jp-radius);
  padding: 0.6rem;
  min-height: 0;
  overflow: hidden;
}

.search-header {
  display: flex;
  gap: 0.4rem;
  margin-bottom: 0.5rem;
}

.search-input {
  flex: 1;
  padding: var(--jp-pad);
  border: 1px solid var(--jp-border);
  border-radius: 4px;
  background: var(--jp-bg);
  color: var(--jp-fg);
}

.refresh-btn {
  border: 1px solid var(--jp-border);
  background: var(--jp-bg);
  border-radius: 4px;
  padding: 0 0.6rem;
  font-size: 1.1rem;
  cursor: pointer;
}

.refresh-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--jp-accent) 15%, transparent);
}

.song-list {
  flex: 1;
  overflow-y: auto;
  border: 1px solid var(--jp-border);
  border-radius: 4px;
  background: var(--jp-bg);
}

.song-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.35rem 0.6rem;
  border-bottom: 1px solid color-mix(in srgb, var(--jp-border) 40%, transparent);
  cursor: pointer;
  user-select: none;
  font-size: 13px;
}

.song-row:hover {
  background: color-mix(in srgb, var(--jp-accent) 10%, transparent);
}

.song-row.selected {
  background: color-mix(in srgb, var(--jp-accent) 22%, transparent);
  font-weight: 550;
}

.track-number {
  font-variant-numeric: tabular-nums;
  min-width: 2.2rem;
  color: var(--jp-muted);
}

.song-title {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.duration {
  color: var(--jp-muted);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  margin-left: 0.4rem;
}

.actions {
  display: flex;
  gap: 0.25rem;
}

.icon-btn {
  border: 1px solid var(--jp-border);
  background: var(--jp-bg-elev);
  border-radius: 4px;
  padding: 0.15rem 0.45rem;
  font-size: 11px;
  cursor: pointer;
}

.play-btn {
  color: var(--jp-ok);
}

.dl-btn {
  color: var(--jp-muted);
}

.icon-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--jp-accent) 20%, transparent);
}

.panel-right {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 1rem;
}

.slots-box {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.slot-fieldset {
  border: 1px solid var(--jp-border);
  border-radius: var(--jp-radius);
  background: var(--jp-bg-elev);
  padding: 0.6rem 0.75rem 0.75rem;
}

.slot-fieldset legend {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--jp-muted);
  padding: 0 0.3rem;
}

.slot-row {
  display: flex;
  gap: 0.5rem;
  align-items: stretch;
}

.slot-spinner {
  width: 4rem;
  padding: 0.4rem;
  border: 1px solid var(--jp-border);
  border-radius: 4px;
  background: var(--jp-bg);
  color: var(--jp-fg);
  font-variant-numeric: tabular-nums;
  font-weight: 600;
  font-size: 14px;
  text-align: center;
}

.slot-action-btn {
  flex: 1;
  padding: 0.5rem 0.7rem;
  border: 1px solid var(--jp-border);
  border-radius: 4px;
  background: var(--jp-bg);
  color: var(--jp-fg);
  font-size: 13px;
  font-weight: 550;
  cursor: pointer;
  text-align: start;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08);
}

.slot-action-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--jp-accent) 15%, transparent);
  border-color: var(--jp-accent);
}

.download-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.download-all-btn {
  padding: 0.65rem 1rem;
  border: 1px solid var(--jp-border);
  border-radius: var(--jp-radius);
  background: var(--jp-bg-elev);
  color: var(--jp-fg);
  font-weight: 600;
  font-size: 13px;
  cursor: pointer;
  box-shadow: var(--jp-shadow);
  text-align: center;
}

.download-all-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--jp-accent) 15%, transparent);
}

.progress-box {
  border: 1px solid var(--jp-border);
  border-radius: var(--jp-radius);
  background: var(--jp-bg-elev);
  padding: 0.6rem;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
}

.progress-label {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 220px;
}

.cancel-btn {
  border: 1px solid var(--jp-border);
  background: var(--jp-bg);
  border-radius: 4px;
  padding: 0.1rem 0.4rem;
  font-size: 11px;
  cursor: pointer;
}

.progress-bar {
  width: 100%;
  height: 6px;
  border-radius: 3px;
  accent-color: var(--jp-accent);
}

.empty-msg {
  padding: 1rem;
  text-align: center;
  color: var(--jp-muted);
  font-size: 13px;
}
</style>
