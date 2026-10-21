import type { Snapshot, ProcInfo } from "../types";
import { fmtBytes, fmtPct } from "../fmt";

function ProcTable({ title, rows }: { title: string; rows: ProcInfo[] }) {
  return (
    <div className="proc-half">
      <span className="tag">{title}</span>
      <table className="proc-table">
        <tbody>
          {rows.map((p) => (
            <tr key={p.pid}>
              <td className="proc-cpu">{fmtPct(p.cpu_pct)}</td>
              <td className="proc-rss">{fmtBytes(p.rss_kb * 1024)}</td>
              <td className="proc-name">{p.name}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function ProcCard({ snap }: { snap: Snapshot }) {
  const p = snap.processes;
  return (
    <section className="card" aria-label="processes">
      <h2>Processes</h2>
      {p.by_cpu.length === 0 && p.by_rss.length === 0 && <span className="muted">nothing found</span>}
      <div className="proc-cols">
        <ProcTable title="by cpu" rows={p.by_cpu} />
        <ProcTable title="by rss" rows={p.by_rss} />
      </div>
    </section>
  );
}
