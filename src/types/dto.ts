/** Opaque profile identifier (UUID string). */
export type ProfileId = string;

/** Congregation profile as returned by Rust. `content_locale` is a JW langwritten code. */
export interface ProfileDto {
  id: ProfileId;
  name: string;
  ui_locale: string;
  content_locale: string;
  media_provider: string;
  created_at: string;
  updated_at: string;
}

/** Payload for `profile_create`. */
export interface CreateProfileDto {
  name: string;
}

/** Payload for `profile_select`. */
export interface SelectProfileDto {
  id: ProfileId;
}

/** Payload for `profile_update`. */
export interface UpdateProfileDto {
  id: ProfileId;
  name?: string;
  ui_locale?: string;
  content_locale?: string;
}

/** Payload for `profile_duplicate`. */
export interface DuplicateProfileDto {
  id: ProfileId;
}

/** Payload for `profile_delete`. */
export interface DeleteProfileDto {
  id: ProfileId;
}

/** JW content language from the embedded seed. */
export interface ContentLanguageDto {
  langwritten: string;
  name: string;
  locale: string;
  direction: string;
  script: string;
}

/** Lookup key for `settings_get`. */
export interface SettingKeyDto {
  profile_id: string;
  key: string;
}

/** One settings cell. `value_json` is a JSON object string for a named key. */
export interface SettingDto {
  profile_id: string;
  key: string;
  value_json: string;
}

/** Console appearance stored per profile. */
export interface AppearanceSetting {
  theme: "system" | "light" | "dark";
  accent: "blue" | "teal" | "violet" | "amber";
  density: "comfortable" | "compact";
}

/** Operator right-panel chrome stored per profile. */
export interface PanelSetting {
  collapsed: boolean;
  width: number;
}

/** How the speaker surface paints the stage. */
export type SpeakerMode = "mirror" | "hud_only";

/** Per-profile assignment of audience/speaker surfaces. */
export interface SurfacesSetting {
  audience_monitor_id: string | null;
  speaker_monitor_id: string | null;
  use_speaker: boolean;
  speaker_mode: SpeakerMode;
}

/** Midweek and weekend meeting times in the machine local timezone. */
export interface MeetingScheduleSetting {
  midweek_weekday: number;
  midweek_time: string;
  weekend_weekday: number;
  weekend_time: string;
}

/** A physical display as shown in Settings. */
export interface MonitorDto {
  id: string;
  name: string;
  width: number;
  height: number;
  is_primary: boolean;
  is_operator: boolean;
  position_x: number;
  position_y: number;
}

/** IPC error payload. The audience webview is not closed when this appears. */
export interface AppErrorDto {
  code: string;
  message: string;
  rev: number;
}

/** Stage kind. Change 001 only has `none`. */
export type StageKind = "none";

/** Versioned stage snapshot. */
export interface StageSnapshot {
  rev: number;
  kind: StageKind;
}

/** Assignment clock. `armed` is ready or paused. */
export type ClockState = "idle" | "armed" | "running";

/** HUD color from remaining vs assigned. */
export type ClockHue = "green" | "amber" | "red";

/** Versioned clock snapshot owned by Rust. */
export interface ClockSnapshot {
  rev: number;
  state: ClockState;
  title: string | null;
  assigned_ms: number;
  elapsed_ms: number;
  remaining_ms: number;
  overtime_ms: number;
  progress_pct: number;
  hue: ClockHue;
}

/** Payload for `clock_arm`. */
export interface ClockArmDto {
  title: string;
  minutes: number;
}

/** Combined output returned by `output_get`. */
export interface OutputBundleDto {
  stage: StageSnapshot;
  clock: ClockSnapshot;
  speaker_mode: SpeakerMode;
}

/** Speaker HUD flags. `message` is empty until a later change. */
export interface SpeakerUiDto {
  mode: SpeakerMode;
  message: string;
}

/** Idle clock used before the first snapshot. */
export function idleClock(): ClockSnapshot {
  return {
    rev: 0,
    state: "idle",
    title: null,
    assigned_ms: 0,
    elapsed_ms: 0,
    remaining_ms: 0,
    overtime_ms: 0,
    progress_pct: 0,
    hue: "green",
  };
}
