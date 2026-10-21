import { useEffect, useMemo, useState } from "react";
import type { Snapshot } from "../types";
import { fetchHistory, isUnauthorized } from "../api";
import { Spark } from "./Spark";
import type { HistoryResp } from "../types";

const RANGES = [60, 300, 900, 3600];

const GROUPS = ["cpu", "memory", "temps", "disks", "network", "gpu", "npu"] as const;

function groupOf(key: string): string {
  return GROUPS.find((g) => key.startsWith(`${g}.`)) ?? "other";
}

const DEFAULTS = ["cpu.usage_pct", "cpu.temp_c", "memory.used_pct", "cpu.psi.some_60", "cpu.loadavg.a"];

export function HistoryCard({ snap }:
  { snap: Snapshot | null }) {
  const [sec, setSec] = useState(300);
  const [hist, setHist] = useState<HistoryResp | null>(null);
  const [err, setErr] = useState("");
  const [sel, setSel] = useState<Set<string>>(new Set());
  const [openGroup, setOpenGroup] = useState<string>("cpu");

  // Refetch after the SSE tick so charts trail live data by ~1 interval.
  const tick = snap?.meta.ts_ms ?? 0;

  useEffect(() => {
    let alive = true;
    const load = () => {
      fetchHistory(sec)
        .then((h) => {
          if (!alive) return;
          setErr("");
          setHist(h);
          setSel((prev) => {
            const known = new Set(Object.keys(h.series));
            const next = new Set([...prev].filter((k) => known.has(k)));
            for (const d of DEFAULTS) if (known.has(d)) next.add(d);
            return next.size ? next : new Set([Object.keys(h.series)[0]].filter(Boolean) as string[]);
          });
        })
        .catch((e) => alive && setErr(isUnauthorized(e) ? "unauthorized — set token" : String(e.message)));
    };
    load();
    const id = window.setInterval(load, 10_000);
    return () => { alive = false; window.clearInterval(id); };
  }, [sec, tick === 0 ? 0 : Math.floor(tick / 10_000)]);

  const keys = useMemo(() => {
    if (!hist) return [];
    return Object.keys(hist.series).sort();
  }, [hist]);

  const groups = useMemo(() => {
    const m = new Map<string, string[]>();
    for (const k of keys) m.get(groupOf(k))?.push(k) ?? m.set(groupOf(k), [k]);
    return [...m.entries()].sort((a, b) => a[0].localeCompare(b[0]));
  }, [keys]);

  return (
    <section className="card" aria-label="history">
      <h2>History
        <span className="range">
          {RANGES.map((r) => (
            <button key={r} className={r === sec ? "on" : ""} onClick={() => setSec(r)}>{r}s</button>
          ))}
        </span>
      </h2>
      {err && <span className="tag warm">{err}</span>}
      {!hist && !err && <span className="muted">loading history…</span>}
      {hist && (
        <>
          <div className="hist-groups">
            {groups.map(([g, ks]) => (
              <div className="hist-group" key={g}>
                <button className="hist-group-btn" onClick={() => setOpenGroup(g)}>
                  <span className={g === openGroup ? "on" : ""}>{g} ({ks.length})</span>
                </button>
                {g === openGroup && (
                  <ul className="hist-keys">
                    {ks.map((k) => (
                      <li key={k}>
                        <label>
                          <input type="checkbox"
                            checked={sel.has(k)}
                            onChange={() => setSel((prev) => {
                              const n = new Set(prev);
                              if (n.has(k)) n.delete(k); else n.add(k);
                              return n;
                            })} />
                          {k}
                        </label>
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            ))}
          </div>
          <div className="hist-charts">
            {keys
              .filter((k) => sel.has(k))
              .map((k) => {
                const data = hist.series[k] ?? [];
                const cur = data.length ? data[data.length - 1] : null;
                return (
                  <div className="hist-chart" key={k}>
                    <div className="hist-title">
                      <span>{k}</span>
                      <span className="muted">{cur == null ? "—" : cur.toFixed(cur >= 100 ? 0 : 2)}</span>
                    </div>
                    <Spark data={data} height={40} />
                  </div>
                );
              })}
          </div>
        </>
      )}
    </section>
  );
}
