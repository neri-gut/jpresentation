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

/** What the audience is showing. */
export type StageKind = "none" | "image" | "video";

/** Versioned stage snapshot. */
export interface StageSnapshot {
  rev: number;
  kind: StageKind;
  name?: string | null;
  mime?: string | null;
  path?: string | null;
}

export type ExplorerKind = "dir" | "image" | "video" | "jwpub";

export interface ExplorerEntryDto {
  name: string;
  path: string;
  kind: ExplorerKind;
}

export interface ExplorerListDto {
  path: string;
  parent: string | null;
  entries: ExplorerEntryDto[];
}

export interface ExplorerSetting {
  roots: string[];
}

export interface ExplorerPathDto {
  path: string;
}

export interface FilePreviewDto {
  mime: string;
  data_base64: string;
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

/** Which rolling week the Multimedia tab is showing. */
export type WeekWhich = "this" | "next";

export type MeetingKind = "midweek" | "weekend";
export type MediaKind = "image" | "video" | "song";
export type MediaStatus =
  | "embedded"
  | "ready"
  | "pending"
  | "pending_hymnal"
  | "failed";

export type MediaRef =
  | { kind: "embedded"; path: string }
  | {
      kind: "catalog";
      key_symbol: string;
      track: number;
      lang_meps: number;
      issue_tag: number;
      mime: string;
    };

export interface MediaItem {
  id: string;
  title: string;
  media_kind: MediaKind;
  status: MediaStatus;
  mime: string;
  media_ref: MediaRef;
  cache_path: string | null;
}

export interface MeetingPart {
  id: string;
  title: string;
  minutes: number | null;
  tone?: string;
  items: MediaItem[];
}

export interface MeetingWeek {
  monday: string;
  kind: MeetingKind;
  title: string;
  langwritten: string;
  pub_symbol: string;
  issue: string;
  parts: MeetingPart[];
  media?: MediaItem[];
}

export interface WeekBundleDto {
  monday: string;
  langwritten: string;
  midweek: MeetingWeek;
  weekend: MeetingWeek;
}

export interface WeekScopeDto {
  which: WeekWhich;
}

export interface WeekPreviewRequestDto {
  which: WeekWhich;
  item_id: string;
}

export type TemplateSource = "system" | "user";
export type TemplateKind = "midweek" | "weekend" | "event";

export interface EventTemplateDto {
  id: string;
  name: string;
  source: TemplateSource;
  kind: TemplateKind;
  parts: MeetingPart[];
}

export interface SaveTemplateDto {
  id?: string | null;
  name: string;
  kind: TemplateKind;
  parts: MeetingPart[];
}

export interface TemplateIdDto {
  id: string;
}

export interface ApplyTemplateDto {
  which: WeekWhich;
  meeting: MeetingKind;
  template_id: string;
}

export interface SetPartsDto {
  which: WeekWhich;
  meeting: MeetingKind;
  parts: MeetingPart[];
}

export interface WeekPreviewDto {
  mime: string;
  data_base64: string;
}

export interface WeekProgressDto {
  phase: string;
  done: number;
  total: number;
  label: string;
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
