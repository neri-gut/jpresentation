/**
 * Heat color along green → amber → red. `t` is elapsed fraction 0..1.
 * Overtime (`t >= 1`) is solid red.
 */
export function clockHeatColor(t: number): string {
  const x = Math.min(1, Math.max(0, t));
  if (x >= 1) {
    return "#f04438";
  }
  if (x < 0.5) {
    return lerpHex("#12b76a", "#f5a524", x / 0.5);
  }
  return lerpHex("#f5a524", "#f04438", (x - 0.5) / 0.5);
}

function lerpHex(a: string, b: string, t: number): string {
  const A = hexRgb(a);
  const B = hexRgb(b);
  const r = Math.round(A[0] + (B[0] - A[0]) * t);
  const g = Math.round(A[1] + (B[1] - A[1]) * t);
  const bl = Math.round(A[2] + (B[2] - A[2]) * t);
  return `rgb(${r}, ${g}, ${bl})`;
}

function hexRgb(hex: string): [number, number, number] {
  const n = hex.replace("#", "");
  return [
    Number.parseInt(n.slice(0, 2), 16),
    Number.parseInt(n.slice(2, 4), 16),
    Number.parseInt(n.slice(4, 6), 16),
  ];
}
