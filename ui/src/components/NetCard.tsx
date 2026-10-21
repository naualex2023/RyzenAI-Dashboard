import type { Snapshot } from "../types";
import { fmtKbps } from "../fmt";

export function NetCard({ snap }: { snap: Snapshot }) {
  return (
    <section className="card" aria-label="network">
      <h2>Network</h2>
      {snap.network.length === 0 && <span className="muted">no interfaces found</span>}
      {snap.network.map((n) => (
        <div className="net" key={n.name}>
          <span className="disk-name">{n.name}</span>
          <span className={`tag ${n.operstate === "up" ? "" : "warm"}`}>{n.operstate}</span>
          {n.link_speed_mbps != null && <span className="tag">{Math.round(n.link_speed_mbps)} Mbps</span>}
          <span className="net-kv">↓ {fmtKbps(n.rx_kbps)} · {Math.round(n.rx_pkts_s)} p/s</span>
          <span className="net-kv">↑ {fmtKbps(n.tx_kbps)} · {Math.round(n.tx_pkts_s)} p/s</span>
        </div>
      ))}
    </section>
  );
}
