import type { Snapshot } from "../types";
import { fmtPct, fmtBytes, fmtMhz, fmtTemp, tempClass, clamp } from "../fmt";
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
            {g.dpm_state && <span className="tag">{g.dpm_state}</span>}
            {g.perf_level && g.perf_level !== "auto" && <span className="tag warm">{g.perf_level}</span>}
            {g.fan_rpm != null && <span className="tag">{g.fan_rpm} rpm</span>}
            {g.temps.map((t) => (
              <span key={t.label} className={`chip ${tempClass(t.temp_c)}`}>{fmtTemp(t.temp_c)} {t.label}</span>
            ))}
          </div>
          <dl className="kv">
            <dt>busy</dt><dd>{fmtPct(g.busy_pct)}</dd>
            {g.mem_busy_pct != null && (<><dt>mem bus</dt><dd>{fmtPct(g.mem_busy_pct)}</dd></>)}
            <dt>vram</dt>
            <dd>{g.vram_total_b != null ? `${fmtBytes(g.vram_used_b)} / ${fmtBytes(g.vram_total_b)}` : "—"}</dd>
            {g.gtt_total_b != null && (<><dt>gtt</dt><dd>{fmtBytes(g.gtt_used_b)} / {fmtBytes(g.gtt_total_b)}</dd></>)}
            {(g.sclk_mhz != null || g.mclk_mhz != null) && (
              <><dt>clocks</dt><dd>{fmtMhz(g.sclk_mhz)} / {fmtMhz(g.mclk_mhz)}</dd></>
            )}
            <dt>power</dt><dd>{g.power_w != null ? `${g.power_w.toFixed(1)} W` : "—"}</dd>
          </dl>
          {g.busy_pct != null && <Bar pct={g.busy_pct} />}
          {g.vram_used_b != null && g.vram_total_b != null && g.vram_total_b > 0 && (
            <Bar pct={clamp((g.vram_used_b / g.vram_total_b) * 100, 0, 100)} />
          )}
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
