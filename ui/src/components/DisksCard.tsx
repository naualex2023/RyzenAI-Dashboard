import type { Snapshot } from "../types";
import { fmtBytes, fmtPct, fmtTemp, tempClass } from "../fmt";
import { Bar } from "./Bar";

export function DisksCard({ snap }: { snap: Snapshot }) {
  return (
    <section className="card" aria-label="disks">
      <h2>Disks</h2>
      {snap.disks.length === 0 && <span className="muted">no block devices found</span>}
      {snap.disks.map((d) => (
        <div className="disk" key={d.name}>
          <div className="disk-head">
            <span className="disk-name">{d.name}</span>
            <span className="tag">{d.kind}</span>
            {d.rotational && <span className="tag warm">rotational</span>}
            {d.temps.map((t) => (
              <span key={t.label} className={`chip ${tempClass(t.temp_c)}`}>{fmtTemp(t.temp_c)} {t.label}</span>
            ))}
          </div>
          {d.fs.map((f) => (
            <div className="fs" key={f.path}>
              <span className="fs-path">{f.path}</span>
              <span className="fs-type muted">{f.fstype}</span>
              <span className="fs-usage">{fmtBytes(f.used_b)} / {fmtBytes(f.total_b)} ({fmtPct(f.used_pct)})</span>
              <div className="fs-bar"><Bar pct={f.used_pct} /></div>
            </div>
          ))}
        </div>
      ))}
    </section>
  );
}
