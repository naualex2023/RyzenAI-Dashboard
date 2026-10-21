import type { Snapshot, HistoryResp } from "./types";

// ---------- token: from `?token=` on first load, then localStorage ----------

const TOKEN_KEY = "rad-token";

export function getToken(): string {
  const q = new URLSearchParams(window.location.search).get("token");
  if (q) localStorage.setItem(TOKEN_KEY, q);
  return localStorage.getItem(TOKEN_KEY) ?? "";
}

export function setToken(t: string): void {
  localStorage.setItem(TOKEN_KEY, t.trim());
}

export function clearToken(): void {
  localStorage.removeItem(TOKEN_KEY);
}

const qs = () => {
  const t = getToken();
  return t ? `?token=${encodeURIComponent(t)}` : "";
};

export const snapshotUrl = () => `/api/snapshot${qs()}`;
export const eventsUrl = () => `/api/events${qs()}`;
export const historyUrl = (sec: number) => `/api/history?sec=${sec}${qs()}`;

export async function fetchSnapshot(): Promise<Snapshot> {
  const r = await fetch(snapshotUrl(), { cache: "no-store" });
  if (r.status === 401) throw new Unauthorized();
  if (!r.ok) throw new Error(`snapshot: ${r.status}`);
  return r.json();
}

export async function fetchHistory(sec: number): Promise<HistoryResp> {
  const r = await fetch(historyUrl(sec), { cache: "no-store" });
  if (r.status === 401) throw new Unauthorized();
  if (!r.ok) throw new Error(`history: ${r.status}`);
  return r.json();
}

export async function fetchSysdump(): Promise<Record<string, string>> {
  const r = await fetch(`/api/sysdump${qs()}`, { cache: "no-store" });
  if (r.status === 401) throw new Unauthorized();
  if (!r.ok) throw new Error(`sysdump: ${r.status}`);
  const files: Record<string, string> = await r.json();
  return files;
}

export class Unauthorized extends Error {
  constructor() { super("unauthorized"); }
}

export const isUnauthorized = (e: unknown): boolean => e instanceof Unauthorized;
