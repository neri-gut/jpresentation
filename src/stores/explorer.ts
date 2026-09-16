import { defineStore } from "pinia";
import { ref } from "vue";

import { invokeCommand, isTauri } from "@/composables/invoke";
import type {
  ExplorerEntryDto,
  ExplorerListDto,
  ExplorerPathDto,
  ExplorerSetting,
  FilePreviewDto,
  StageSnapshot,
} from "@/types/dto";

/** File waiting in the right-panel stage. Not on the audience until Show. */
export interface StageCue {
  path: string;
  name: string;
  kind: "image" | "video";
  previewUrl: string | null;
}

/**
 * Operator file explorer. Rust lists disks; this store only caches the last listing.
 */
export const useExplorerStore = defineStore("explorer", () => {
  const listing = ref<ExplorerListDto>({
    path: "",
    parent: null,
    entries: [],
  });
  const roots = ref<ExplorerEntryDto[]>([]);
  const dirs = ref<ExplorerEntryDto[]>([]);
  const files = ref<ExplorerEntryDto[]>([]);
  const thumbs = ref<Record<string, string>>({});
  const cue = ref<StageCue | null>(null);
  const busy = ref(false);

  async function fetchList(path: string): Promise<ExplorerListDto> {
    const payload: ExplorerPathDto = { path };
    return invokeCommand<ExplorerListDto, ExplorerPathDto>("explorer_list", payload);
  }

  /** Lists virtual roots (left tree). */
  async function loadRoots(): Promise<void> {
    if (!isTauri()) {
      return;
    }
    const listed = await fetchList("");
    roots.value = listed.entries.filter((item) => item.kind === "dir");
  }

  /** Opens a folder into the thumbnail pane. Flattens `img/` and `vid/`. */
  async function list(path = ""): Promise<void> {
    if (!isTauri()) {
      return;
    }
    busy.value = true;
    try {
      if (path === "") {
        await loadRoots();
        listing.value = { path: "", parent: null, entries: roots.value };
        files.value = [];
        dirs.value = [];
        return;
      }
      const listed = await fetchList(path);
      listing.value = listed;
      const collected: ExplorerEntryDto[] = [];
      const nested: ExplorerEntryDto[] = [];
      for (const entry of listed.entries) {
        if (entry.kind === "dir" && (entry.name === "img" || entry.name === "vid")) {
          const inner = await fetchList(entry.path);
          collected.push(...inner.entries);
        } else if (entry.kind === "dir") {
          nested.push(entry);
        } else if (entry.kind === "image" || entry.kind === "video" || entry.kind === "jwpub") {
          collected.push(entry);
        }
      }
      dirs.value = nested;
      files.value = collected;
      void loadThumbs(collected);
    } finally {
      busy.value = false;
    }
  }

  async function loadThumbs(entries: ExplorerEntryDto[]): Promise<void> {
    for (const entry of entries) {
      if (entry.kind !== "image" || thumbs.value[entry.path]) {
        continue;
      }
      try {
        const shot = await preview(entry.path);
        if (shot) {
          thumbs.value = {
            ...thumbs.value,
            [entry.path]: `data:${shot.mime};base64,${shot.data_base64}`,
          };
        }
      } catch {
        /* skip a broken thumb; the grid still shows the name */
      }
    }
  }

  /** Queues a file on the right-panel stage. Does not project. */
  async function cueFile(entry: ExplorerEntryDto): Promise<void> {
    if (entry.kind !== "image" && entry.kind !== "video") {
      return;
    }
    let previewUrl: string | null = thumbs.value[entry.path] ?? null;
    if (entry.kind === "image" && !previewUrl) {
      const shot = await preview(entry.path);
      if (shot) {
        previewUrl = `data:${shot.mime};base64,${shot.data_base64}`;
        thumbs.value = { ...thumbs.value, [entry.path]: previewUrl };
      }
    }
    cue.value = {
      path: entry.path,
      name: entry.name,
      kind: entry.kind,
      previewUrl,
    };
  }

  /** Native folder dialog; adds the chosen directory as a root. */
  async function pickRoot(): Promise<void> {
    await invokeCommand<ExplorerSetting>("explorer_pick_root");
    await loadRoots();
  }

  /** Adds an absolute folder to the profile. */
  async function addRoot(path: string): Promise<ExplorerSetting> {
    const payload: ExplorerPathDto = { path };
    return invokeCommand<ExplorerSetting, ExplorerPathDto>("explorer_add_root", payload);
  }

  /** Removes a saved root. */
  async function removeRoot(path: string): Promise<ExplorerSetting> {
    const payload: ExplorerPathDto = { path };
    return invokeCommand<ExplorerSetting, ExplorerPathDto>(
      "explorer_remove_root",
      payload,
    );
  }

  /** Extracts media from a .jwpub and lists that folder. */
  async function openJwpub(path: string): Promise<void> {
    const payload: ExplorerPathDto = { path };
    const listed = await invokeCommand<ExplorerListDto, ExplorerPathDto>(
      "explorer_open_jwpub",
      payload,
    );
    listing.value = listed;
    dirs.value = listed.entries.filter((item) => item.kind === "dir");
    files.value = listed.entries.filter(
      (item) => item.kind === "image" || item.kind === "video",
    );
    void loadThumbs(files.value);
  }

  /** Image preview payload. */
  async function preview(path: string): Promise<FilePreviewDto | null> {
    const payload: ExplorerPathDto = { path };
    return invokeCommand<FilePreviewDto | null, ExplorerPathDto>(
      "explorer_preview",
      payload,
    );
  }

  /** Sends a local image or video to audience and speaker. */
  async function openOnStage(path: string): Promise<StageSnapshot> {
    const payload: ExplorerPathDto = { path };
    return invokeCommand<StageSnapshot, ExplorerPathDto>("stage_open", payload);
  }

  /** Clears the stage. */
  async function closeStage(): Promise<void> {
    await invokeCommand<StageSnapshot>("stage_close");
  }

  return {
    listing,
    roots,
    dirs,
    files,
    thumbs,
    cue,
    busy,
    list,
    loadRoots,
    cueFile,
    pickRoot,
    addRoot,
    removeRoot,
    openJwpub,
    preview,
    openOnStage,
    closeStage,
  };
});
