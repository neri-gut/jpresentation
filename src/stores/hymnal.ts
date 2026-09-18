import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { ref } from "vue";

import { invokeCommand } from "@/composables/invoke";
import type {
  HymnalBundleDto,
  HymnalProgressDto,
  HymnalSlotsDto,
  HymnalSongDto,
  HymnalTrackRequestDto,
  StageSnapshot,
} from "@/types/dto";

export const useHymnalStore = defineStore("hymnal", () => {
  const songs = ref<HymnalSongDto[]>([]);
  const slots = ref<HymnalSlotsDto>({ start: null, middle: null, end: null });
  const selectedTrack = ref<number | null>(null);
  const downloading = ref(false);
  const downloadProgress = ref<HymnalProgressDto | null>(null);
  const loading = ref(false);
  let unlisten: UnlistenFn | null = null;

  async function ensureListening(): Promise<void> {
    if (unlisten) {
      return;
    }
    unlisten = await listen<HymnalProgressDto>("hymnal://progress", (event) => {
      downloadProgress.value = event.payload;
    });
  }

  async function hydrate(): Promise<void> {
    loading.value = true;
    try {
      await ensureListening();
      const bundle = await invokeCommand<HymnalBundleDto>("hymnal_get");
      songs.value = bundle.songs;
      if (bundle.slots.start || bundle.slots.middle || bundle.slots.end) {
        slots.value = bundle.slots;
      }
      if (!selectedTrack.value && songs.value.length > 0) {
        selectedTrack.value = songs.value[0]?.track ?? 1;
      }
    } finally {
      loading.value = false;
    }
  }

  async function refreshCatalog(): Promise<void> {
    loading.value = true;
    try {
      const bundle = await invokeCommand<HymnalBundleDto>("hymnal_refresh");
      songs.value = bundle.songs;
    } finally {
      loading.value = false;
    }
  }

  async function downloadSong(track: number): Promise<HymnalSongDto> {
    const updated = await invokeCommand<HymnalSongDto, HymnalTrackRequestDto>(
      "hymnal_download_song",
      { track },
    );
    const index = songs.value.findIndex((s) => s.track === track);
    if (index !== -1) {
      songs.value[index] = updated;
    }
    return updated;
  }

  async function downloadAll(): Promise<void> {
    downloading.value = true;
    downloadProgress.value = null;
    try {
      await ensureListening();
      const result = await invokeCommand<HymnalSongDto[]>("hymnal_download_all");
      songs.value = result;
    } finally {
      downloading.value = false;
      downloadProgress.value = null;
    }
  }

  async function cancelDownload(): Promise<void> {
    await invokeCommand<void>("hymnal_cancel");
    downloading.value = false;
    downloadProgress.value = null;
  }

  async function playSong(track: number): Promise<StageSnapshot> {
    selectedTrack.value = track;
    const snap = await invokeCommand<StageSnapshot, HymnalTrackRequestDto>(
      "hymnal_play",
      { track },
    );
    const index = songs.value.findIndex((s) => s.track === track);
    if (index !== -1 && songs.value[index]) {
      songs.value[index].status = "ready";
    }
    return snap;
  }

  function setSlot(slotName: "start" | "middle" | "end", track: number | null): void {
    slots.value[slotName] = track;
  }

  return {
    songs,
    slots,
    selectedTrack,
    downloading,
    downloadProgress,
    loading,
    hydrate,
    refreshCatalog,
    downloadSong,
    downloadAll,
    cancelDownload,
    playSong,
    setSlot,
  };
});
