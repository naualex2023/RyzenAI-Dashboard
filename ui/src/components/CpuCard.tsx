import type { Snapshot } from "../types";
import { fmtPct, fmtTemp, fmtMhz, tempClass, psiLabel, psiClass } from "../fmt";
import { Bar } from "./Bar";

export function CpuCard({ snap }: { snap: Snapshot }) {
  const c = snap.cpu;
  const cores = c.per_core;
  return (
    <section className="card" aria-label="cpu">
      <h2>CPU</h2>
      <div className="row-big">
        <span className="big">{fmtPct(c.usage_pct)}</span>
        <span className={`chip ${tempClass(c.temp_c)}`}>{fmtTemp(c.temp_c)}</span>
        <span className={`chip ${psiClass(c.psi.some_60)}`}>PSI {psiLabel(c.psi)}</span>
      </div>
      <Bar pct={c.usage_pct} />
      <div className="cores">
        {cores.map((k) => (
          <div className="core" key={k.core}>
            <div className="core-bar"><div className="core-fill" style={{ width: `${Math.min(100, k.usage_pct)}%` }} /></div>
            <span className="core-lbl">c{k.core} {fmtPct(k.usage_pct)}</span>
            <span className="core-freq muted">{fmtMhz(k.freq_mhz)}</span>
          </div>
        ))}
      </div>
    </section>
  );
}
