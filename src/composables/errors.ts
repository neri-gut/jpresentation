import type { AppErrorDto } from "@/types/dto";

const CODE_KEYS: Record<string, string> = {
  LastProfile: "errors.lastProfile",
  UnknownContentLanguage: "errors.unknownContentLanguage",
  MonitorMissing: "errors.monitorMissing",
  ClockNotArmed: "errors.clockNotArmed",
  ClockNotRunning: "errors.clockNotRunning",
  UnreadablePub: "errors.unreadablePub",
  CatalogNotFound: "errors.catalogNotFound",
  Network: "errors.network",
  Cancelled: "errors.cancelled",
  NotFound: "errors.notFound",
  Busy: "errors.busy",
};

/**
 * Maps an IPC error to an i18n string. Unknown codes fall back to `errors.invokeFailed`.
 */
export function errorMessage(err: unknown, t: (key: string) => string): string {
  if (err && typeof err === "object" && "code" in err) {
    const dto = err as AppErrorDto;
    const key = CODE_KEYS[String(dto.code)];
    if (key) {
      return t(key);
    }
  }
  return t("errors.invokeFailed");
}
