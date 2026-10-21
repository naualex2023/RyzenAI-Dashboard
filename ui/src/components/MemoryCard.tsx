import type { Snapshot } from "../types";
import { fmtBytes, fmtPct, psiLabel, psiClass } from "../fmt";
import { Bar } from "./Bar";

export function MemoryCard({ snap }: { snap: Snapshot }) {
  const m = snap.memory;
  return (
    <section className="card" aria-label="memory">
      <h2>Memory</h2>
      <div className="row-big">
        <span className="big">{fmtPct(m.used_pct)}</span>
        <span className="muted">{fmtBytes(m.used_b)} / {fmtBytes(m.total_b)}</span>
        <span className={`chip ${psiClass(m.psi.some_60)}`}>PSI {psiLabel(m.psi)}</span>
      </div>
      <Bar pct={m.used_pct} />
      <dl className="kv">
        <dt>available</dt><dd>{fmtBytes(m.available_b)}</dd>
        <dt>cached</dt><dd>{fmtBytes(m.cached_b)}</dd>
        <dt>buffers</dt><dd>{fmtBytes(m.buffers_b)}</dd>
        <dt>swap</dt><dd>{m.swap_total_b > 0 ? `${fmtBytes(m.swap_used_b)} / ${fmtBytes(m.swap_total_b)}` : "—"}</dd>
      </dl>
    </section>
  );
}
