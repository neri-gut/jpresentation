/**
 * Formats remaining milliseconds as `mm:ss`, or overtime as `+m:ss`.
 */
export function formatRemaining(remainingMs: number): string {
  if (remainingMs < 0) {
    const total = Math.floor(-remainingMs / 1000);
    const minutes = Math.floor(total / 60);
    const seconds = total % 60;
    return `+${minutes}:${String(seconds).padStart(2, "0")}`;
  }
  const total = Math.floor(remainingMs / 1000);
  const minutes = Math.floor(total / 60);
  const seconds = total % 60;
  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}
