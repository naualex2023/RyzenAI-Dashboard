//! Value formatting helpers — all tolerant of `null` (serde `Option`).

export function fmtBytes(b: number | null | undefined): string {
  if (b == null) return "—";
  const units = ["B", "KB", "MB", "GB", "TB", "PB"];
  let v = b;
  let u = 0;
  while (v >= 1000 && u < units.length - 1) { v /= 1000; u += 1; }
  return u === 0 ? `${v.toFixed(0)} ${units[u]}` : `${(v).toFixed(u >= 3 ? 2 : 1)} ${units[u]}`;
}

export function fmtKbps(kbps: number): string {
  if (Math.abs(kbps) >= 1_000_000) return `${(kbps / 1_000_000).toFixed(2)} TB/s`;
  if (Math.abs(kbps) >= 1000) return `${(kbps / 1000).toFixed(1)} MB/s`;
  return `${Math.round(kbps)} KB/s`;
}

export function fmtPct(x: number | null | undefined): string {
  return x == null ? "—" : `${x.toFixed(1)}%`;
}

export function fmtTemp(x: number | null | undefined): string {
  return x == null ? "—" : `${x.toFixed(1)}°C`;
}

export function fmtMhz(x: number | null | undefined): string {
  return x == null ? "—" : `${Math.round(x)} MHz`;
}

export function fmtUptime(s: number): string {
  const d = Math.floor(s / 86400);
  const h = Math.floor((s % 86400) / 3600);
  const m = Math.floor((s % 3600) / 60);
  return d > 0 ? `${d}d ${h}h ${m}m` : h > 0 ? `${h}h ${m}m` : `${m}m ${Math.floor(s % 60)}s`;
}

export function psiClass(x: number): "ok" | "warn" | "hot" {
  return x < 2 ? "ok" : x < 10 ? "warn" : "hot";
}

export function tempClass(c: number | null | undefined): "ok" | "warn" | "hot" {
  if (c == null) return "ok";
  return c < 50 ? "ok" : c < 75 ? "warn" : "hot";
}

export function psiLabel(p: { some_60: number; full_60: number }): string {
  return `some ${(p.some_60).toFixed(1)}% / full ${(p.full_60).toFixed(1)}%`;
}

export function clamp(x: number, lo: number, hi: number): number {
  return Math.min(hi, Math.max(lo, x));
}
