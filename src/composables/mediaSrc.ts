import { convertFileSrc } from "@tauri-apps/api/core";

import type { StageSnapshot } from "@/types/dto";

/**
 * Image sources stay on the asset protocol. Video uses the loopback server:
 * WebKitGTK stops MP4 playback when the asset protocol answers a range with a short slice.
 */
export function stageMediaSrc(stage: StageSnapshot, mediaOrigin: string): string {
  const path = stage.path;
  if (!path) {
    return "";
  }
  if (stage.kind === "video" && mediaOrigin) {
    return `${mediaOrigin}/media?path=${encodeURIComponent(path)}`;
  }
  return convertFileSrc(path);
}
