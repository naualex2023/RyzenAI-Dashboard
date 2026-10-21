import type { Snapshot } from "../types";
import { fmtPct, fmtBytes, fmtMhz, fmtTemp, tempClass } from "../fmt";
import { Bar } from "./Bar";

export function AcceleratorCard({ snap }: { snap: Snapshot }) {
  const g = snap.gpu;
  const n = snap.npu;
  if (!g && !n) return null;
  return (
    <section className="card" aria-label="accelerators">
      <h2>Accelerators</h2>
      {g && (
        <div className="accel">
          <div className="disk-head">
            <span className="disk-name">{g.card}</span>
            <span className="tag">gpu</span>
            {g.temps.map((t) => (
              <span key={t.label} className={`chip ${tempClass(t.temp_c)}`}>{fmtTemp(t.temp_c)} {t.label}</span>
            ))}
          </div>
          <dl className="kv">
            <dt>busy</dt><dd>{fmtPct(g.busy_pct)}</dd>
            <dt>vram</dt><dd>{g.vram_total_b != null ? `${fmtBytes(g.vram_used_b)} / ${fmtBytes(g.vram_total_b)}` : "—"}</dd>
            <dt>power</dt><dd>{g.power_w != null ? `${g.power_w.toFixed(1)} W` : "—"}</dd>
          </dl>
          {g.busy_pct != null && <Bar pct={g.busy_pct} />}
        </div>
      )}
      {n && (
        <div className="accel">
          <div className="disk-head">
            <span className="disk-name">{n.source}</span>
            <span className="tag">npu</span>
          </div>
          <dl className="kv">
            <dt>freq</dt><dd>{fmtMhz(n.freq_mhz)}</dd>
            <dt>power</dt><dd>{n.power_w != null ? `${n.power_w.toFixed(1)} W` : "—"}</dd>
          </dl>
        </div>
      )}
    </section>
  );
}
