<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";

import { errorMessage } from "@/composables/errors";
import { useExplorerStore } from "@/stores/explorer";
import { useUiStore } from "@/stores/ui";
import { useWeekStore } from "@/stores/week";
import type { ExplorerEntryDto } from "@/types/dto";

const { t } = useI18n();
const explorer = useExplorerStore();
const week = useWeekStore();
const ui = useUiStore();

const addPath = ref("");

const videos = computed(() => explorer.files.filter((item) => item.kind === "video"));
const images = computed(() => explorer.files.filter((item) => item.kind === "image"));
const pubs = computed(() => explorer.files.filter((item) => item.kind === "jwpub"));
const selectedPath = computed(() => explorer.cue?.path ?? null);

onMounted(async () => {
  await run(async () => {
    await explorer.loadRoots();
    await explorer.list("");
  });
});

async function run(action: () => Promise<void>): Promise<void> {
  try {
    await action();
  } catch (err) {
    ui.showToast(errorMessage(err, t));
  }
}

function openFolder(path: string): Promise<void> {
  return run(() => explorer.list(path));
}

async function onFileClick(entry: ExplorerEntryDto): Promise<void> {
  if (entry.kind === "jwpub") {
    await run(() => explorer.openJwpub(entry.path));
    return;
  }
  await run(() => explorer.cueFile(entry));
}

function addFolder(): Promise<void> {
  const path = addPath.value.trim();
  if (!path) {
    return Promise.resolve();
  }
  return run(async () => {
    await explorer.addRoot(path);
    addPath.value = "";
    await explorer.loadRoots();
    await explorer.list("");
  });
}


</script>

<template>
  <section class="media">
    <header class="toolbar">
      <button type="button" :disabled="!explorer.listing.path" @click="openFolder(explorer.listing.parent ?? '')">
        {{ t("media.up") }}
      </button>
      <p class="crumb">{{ explorer.listing.path || t("media.roots") }}</p>
      <button type="button" :disabled="week.busy" @click="run(async () => { await week.downloadMedia(); if (explorer.listing.path) await explorer.list(explorer.listing.path); })">
        {{ t("media.downloadWeek") }}
      </button>
    </header>
    <div class="add">
      <button type="button" @click="run(() => explorer.pickRoot())">{{ t("media.addFolder") }}</button>
      <input v-model="addPath" type="text" :placeholder="t('media.addFolderHint')" />
      <button type="button" @click="addFolder">{{ t("media.addPath") }}</button>
    </div>
    <div class="body">
      <nav class="tree" :aria-label="t('media.roots')">
        <p class="tree-label">{{ t("media.places") }}</p>
        <button
          v-for="root in explorer.roots"
          :key="root.path"
          type="button"
          class="tree-item"
          :class="{ active: explorer.listing.path === root.path }"
          @click="openFolder(root.path)"
        >
          {{ root.name }}
        </button>
        <template v-if="explorer.dirs.length">
          <p class="tree-label">{{ t("media.folders") }}</p>
          <button
            v-for="dir in explorer.dirs"
            :key="dir.path"
            type="button"
            class="tree-item nested"
            :class="{ active: explorer.listing.path === dir.path }"
            @click="openFolder(dir.path)"
          >
            {{ dir.name }}
          </button>
        </template>
      </nav>
      <div class="grid">
        <p v-if="explorer.files.length === 0" class="empty">{{ t("media.cueHint") }}</p>
        <section v-if="videos.length" class="group">
          <h2>{{ t("media.videos") }}</h2>
          <div class="cards">
            <button
              v-for="entry in videos"
              :key="entry.path"
              type="button"
              class="card video"
              :class="{ selected: selectedPath === entry.path }"
              @click="onFileClick(entry)"
            >
              <span class="thumb video-thumb">▶</span>
              <span class="meta">
                <strong>{{ entry.name }}</strong>
              </span>
            </button>
          </div>
        </section>
        <section v-if="images.length" class="group">
          <h2>{{ t("media.images") }}</h2>
          <div class="cards">
            <button
              v-for="entry in images"
              :key="entry.path"
              type="button"
              class="card"
              :class="{ selected: selectedPath === entry.path }"
              @click="onFileClick(entry)"
            >
              <img
                v-if="explorer.thumbs[entry.path]"
                class="thumb"
                :src="explorer.thumbs[entry.path]"
                alt=""
              />
              <span v-else class="thumb" />
              <span class="meta">
                <strong>{{ entry.name }}</strong>
              </span>
            </button>
          </div>
        </section>
        <section v-if="pubs.length" class="group">
          <h2>{{ t("media.kind.jwpub") }}</h2>
          <button
            v-for="entry in pubs"
            :key="entry.path"
            type="button"
            class="pub"
            @click="onFileClick(entry)"
          >
            {{ entry.name }}
          </button>
        </section>
      </div>
    </div>
  </section>
</template>

<style scoped>
.media {
  display: flex;
  flex-direction: column;
  gap: 0.45rem;
  height: 100%;
  min-height: 0;
}

.toolbar,
.add {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  align-items: center;
}

.crumb {
  margin: 0;
  flex: 1;
  min-width: 8rem;
  font-size: 12px;
  color: var(--jp-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

button,
input {
  border: 1px solid var(--jp-border);
  background: var(--jp-bg);
  border-radius: 4px;
  padding: var(--jp-pad);
}

input {
  flex: 1;
  min-width: 10rem;
}

button:disabled {
  opacity: 0.55;
}

.body {
  display: grid;
  grid-template-columns: 11rem minmax(0, 1fr);
  gap: 0.5rem;
  min-height: 0;
  flex: 1;
}

.tree,
.grid {
  border: 1px solid var(--jp-border);
  border-radius: var(--jp-radius);
  background: var(--jp-bg-elev);
  overflow: auto;
  min-height: 12rem;
}

.tree {
  padding: 0.35rem;
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}

.tree-item {
  text-align: start;
  width: 100%;
}

.tree-item.active {
  background: color-mix(in srgb, var(--jp-accent) 16%, transparent);
}

.tree-item.nested {
  padding-left: 0.9rem;
}

.tree-label {
  margin: 0.35rem 0 0.15rem;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--jp-muted);
}

.grid {
  padding: 0.55rem;
}

.group {
  margin-bottom: 0.8rem;
}

h2 {
  margin: 0 0 0.4rem;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--jp-muted);
}

.cards {
  display: flex;
  flex-wrap: wrap;
  gap: 0.7rem;
}

.card {
  display: flex;
  gap: 0.45rem;
  align-items: center;
  width: min(22rem, 100%);
  text-align: start;
  padding: 0.3rem;
}

.card.selected {
  outline: 2px solid var(--jp-accent);
}

.thumb {
  width: 7.5rem;
  height: 4.4rem;
  object-fit: cover;
  background: #c5c5c5;
  flex: 0 0 7.5rem;
}

.video-thumb {
  display: grid;
  place-items: center;
  color: #fff;
  background: #3a3a3a;
  font-size: 1.2rem;
}

.meta {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  min-width: 0;
}

.meta strong {
  font-size: 12px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
}

.pub {
  display: block;
  margin-top: 0.2rem;
  text-align: start;
}

.empty {
  margin: 0;
  color: var(--jp-muted);
}
</style>
