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

const filter = ref("");
const selectedPath = computed(() => explorer.cue?.path ?? null);

const sep = computed(() =>
  explorer.listing.path.includes("\\") && !explorer.listing.path.startsWith("/") ? "\\" : "/",
);

const crumbs = computed(() => {
  const path = explorer.listing.path;
  const items: { name: string; path: string }[] = [{ name: t("media.places"), path: "" }];
  if (!path) {
    return items;
  }
  const root = explorer.roots.find(
    (r) => path === r.path || path.startsWith(r.path + sep.value),
  );
  if (!root) {
    items.push({ name: path, path });
    return items;
  }
  items.push({ name: root.name, path: root.path });
  if (path === root.path) {
    return items;
  }
  const rest = path.slice(root.path.length).split(/[/\\]/).filter(Boolean);
  let acc = root.path;
  for (const part of rest) {
    acc = acc.endsWith(sep.value) ? acc + part : acc + sep.value + part;
    items.push({ name: part, path: acc });
  }
  return items;
});

const folders = computed(() =>
  explorer.dirs.filter((item) => matches(item.name)),
);
const videos = computed(() =>
  explorer.files.filter((item) => item.kind === "video" && matches(item.name)),
);
const images = computed(() =>
  explorer.files.filter((item) => item.kind === "image" && matches(item.name)),
);
const pubs = computed(() =>
  explorer.files.filter((item) => item.kind === "jwpub" && matches(item.name)),
);
const empty = computed(
  () =>
    folders.value.length + videos.value.length + images.value.length + pubs.value.length === 0,
);

function matches(name: string): boolean {
  const q = filter.value.trim().toLowerCase();
  return !q || name.toLowerCase().includes(q);
}

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
  filter.value = "";
  return run(() => explorer.list(path));
}

async function onTileClick(entry: ExplorerEntryDto): Promise<void> {
  if (entry.kind === "dir") {
    await openFolder(entry.path);
    return;
  }
  if (entry.kind === "jwpub") {
    await run(() => explorer.openJwpub(entry.path));
    return;
  }
  await run(() => explorer.cueFile(entry));
}
</script>

<template>
  <section class="media">
    <header class="toolbar">
      <button type="button" :disabled="!explorer.listing.path" @click="openFolder(explorer.listing.parent ?? '')">
        {{ t("media.up") }}
      </button>
      <nav class="crumbs" :aria-label="t('media.roots')">
        <button
          v-for="(crumb, index) in crumbs"
          :key="crumb.path + index"
          type="button"
          class="crumb"
          :disabled="index === crumbs.length - 1 && !!explorer.listing.path"
          @click="openFolder(crumb.path)"
        >
          {{ crumb.name }}
        </button>
      </nav>
      <input
        v-model="filter"
        type="search"
        class="filter"
        :placeholder="t('media.filter')"
        :aria-label="t('media.filter')"
      />
      <button type="button" @click="run(() => explorer.pickRoot())">{{ t("media.addFolder") }}</button>
      <button
        type="button"
        :disabled="week.busy"
        @click="run(async () => { await week.downloadMedia(); if (explorer.listing.path) await explorer.list(explorer.listing.path); })"
      >
        {{ t("media.downloadWeek") }}
      </button>
    </header>
    <div class="body">
      <nav class="tree" :aria-label="t('media.places')">
        <p class="tree-label">{{ t("media.places") }}</p>
        <button
          v-for="root in explorer.roots"
          :key="root.path"
          type="button"
          class="tree-item"
          :class="{ active: explorer.listing.path === root.path || explorer.listing.path.startsWith(root.path + sep) }"
          @click="openFolder(root.path)"
        >
          {{ root.name }}
        </button>
      </nav>
      <div class="grid-wrap">
        <p v-if="empty" class="empty">{{ t("media.cueHint") }}</p>
        <div v-else class="grid">
          <button
            v-for="entry in folders"
            :key="entry.path"
            type="button"
            class="tile folder"
            @click="onTileClick(entry)"
          >
            <span class="thumb folder-thumb">{{ t("media.kind.dir") }}</span>
            <span class="cap">{{ entry.name }}</span>
          </button>
          <button
            v-for="entry in videos"
            :key="entry.path"
            type="button"
            class="tile"
            :class="{ selected: selectedPath === entry.path }"
            @click="onTileClick(entry)"
          >
            <span class="thumb video-thumb">▶</span>
            <span class="cap">{{ entry.name }}</span>
          </button>
          <button
            v-for="entry in images"
            :key="entry.path"
            type="button"
            class="tile"
            :class="{ selected: selectedPath === entry.path }"
            @click="onTileClick(entry)"
          >
            <img
              v-if="explorer.thumbs[entry.path]"
              class="thumb"
              :src="explorer.thumbs[entry.path]"
              alt=""
            />
            <span v-else class="thumb" />
            <span class="cap">{{ entry.name }}</span>
          </button>
          <button
            v-for="entry in pubs"
            :key="entry.path"
            type="button"
            class="tile"
            @click="onTileClick(entry)"
          >
            <span class="thumb pub-thumb">{{ t("media.kind.jwpub") }}</span>
            <span class="cap">{{ entry.name }}</span>
          </button>
        </div>
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

.toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  align-items: center;
}

.crumbs {
  display: flex;
  flex-wrap: wrap;
  gap: 0.15rem;
  flex: 1;
  min-width: 10rem;
  align-items: center;
}

.crumb {
  border: 0;
  background: transparent;
  padding: 0.15rem 0.35rem;
  border-radius: 4px;
  color: var(--jp-accent);
}

.crumb:disabled {
  color: var(--jp-fg);
  opacity: 1;
  font-weight: 650;
}

.crumb:not(:disabled):hover {
  background: color-mix(in srgb, var(--jp-accent) 12%, transparent);
}

.crumb:not(:last-child)::after {
  content: "/";
  margin-left: 0.25rem;
  color: var(--jp-muted);
  font-weight: 400;
}

.filter {
  width: 11rem;
}

button,
input {
  border: 1px solid var(--jp-border);
  background: var(--jp-bg);
  border-radius: 4px;
  padding: var(--jp-pad);
}

.filter {
  border: 1px solid var(--jp-border);
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
.grid-wrap {
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

.tree-label {
  margin: 0.15rem 0;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--jp-muted);
}

.grid-wrap {
  padding: 0.55rem;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(9.5rem, 1fr));
  gap: 0.65rem;
}

.tile {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  text-align: start;
  padding: 0.35rem;
  min-width: 0;
}

.tile.selected {
  outline: 2px solid var(--jp-accent);
}

.thumb {
  width: 100%;
  aspect-ratio: 1;
  object-fit: contain;
  background: #1a1a1a;
  border-radius: 4px;
}

.folder-thumb,
.video-thumb,
.pub-thumb {
  display: grid;
  place-items: center;
  color: #f4f4f4;
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.folder-thumb {
  background: #3d4a5c;
}

.video-thumb {
  background: #2a2a2a;
  font-size: 1.4rem;
}

.pub-thumb {
  background: #3a3228;
}

.cap {
  font-size: 11px;
  line-height: 1.25;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.empty {
  margin: 0;
  color: var(--jp-muted);
}
</style>
