import type { Snapshot } from "../types";
import { fmtTemp, tempClass } from "../fmt";

const ROLE_ORDER = ["cpu", "gpu", "ssd", "vrm", "soc", "chipset", "ambient", "misc"];

export function TempsCard({ snap }: { snap: Snapshot }) {
  const sorted = [...snap.temps].sort((a, b) => {
    const ra = ROLE_ORDER.indexOf(a.role), rb = ROLE_ORDER.indexOf(b.role);
    return (ra < 0 ? 99 : ra) - (rb < 0 ? 99 : rb) || a.id.localeCompare(b.id);
  });
  return (
    <section className="card" aria-label="temps">
      <h2>Temps</h2>
      {sorted.length === 0 && <span className="muted">no sensors found</span>}
      <div className="temps-grid">
        {sorted.map((t) => (
          <div className="temp" key={t.id}>
            <span className={`temp-val chip ${tempClass(t.temp_c)}`}>{fmtTemp(t.temp_c)}</span>
            <span className="temp-lbl">{t.label || t.source}</span>
            <span className="temp-role muted">{t.role} · {t.id}</span>
          </div>
        ))}
      </div>
    </section>
  );
}
