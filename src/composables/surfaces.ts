import type { MonitorDto, SurfacesSetting } from "@/types/dto";

/**
 * True when the saved speaker monitor cannot be used (missing, or the same as audience).
 * Matches `speaker_placement` in Rust: empty id is preview, not missing.
 */
export function speakerMonitorMissing(
  surfaces: SurfacesSetting,
  monitors: MonitorDto[],
): boolean {
  if (!surfaces.use_speaker) {
    return false;
  }
  const id = surfaces.speaker_monitor_id;
  if (!id) {
    return false;
  }
  const exists = monitors.some((item) => item.id === id);
  const sameAsAudience = surfaces.audience_monitor_id === id;
  return !exists || sameAsAudience;
}
