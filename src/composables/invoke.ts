import { invoke } from "@tauri-apps/api/core";

import type { AppErrorDto } from "@/types/dto";

/** True when the page is running inside a Tauri webview. */
export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * Typed IPC helper. Every command takes a single `payload` DTO when it needs input.
 * Failures are rethrown as `AppErrorDto` so the console can toast without touching the stage.
 */
export async function invokeCommand<T, P = undefined>(
  command: string,
  payload?: P,
): Promise<T> {
  try {
    if (payload === undefined) {
      return await invoke<T>(command);
    }
    return await invoke<T>(command, { payload });
  } catch (err) {
    throw normalizeError(err);
  }
}

function normalizeError(err: unknown): AppErrorDto {
  if (err && typeof err === "object" && "code" in err && "message" in err) {
    const dto = err as AppErrorDto;
    return {
      code: String(dto.code),
      message: String(dto.message),
      rev: typeof dto.rev === "number" ? dto.rev : 0,
    };
  }
  if (typeof err === "string") {
    return { code: "Invariant", message: err, rev: 0 };
  }
  return { code: "Invariant", message: "invoke failed", rev: 0 };
}
