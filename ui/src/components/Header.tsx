import type { Snapshot } from "../types";
import { fmtUptime } from "../fmt";

export type Conn = "connecting" | "live" | "offline";

export function Header({ snap, conn }:
  { snap: Snapshot | null; conn: Conn }) {
  const sys = snap?.system;
  return (
    <header className="hdr">
      <div className="hdr-title">
        <h1>RyzenAI Dashboard</h1>
        <span className={`conn conn-${conn}`} title="SSE /api/events">
          {conn === "live" ? "● live" : conn === "offline" ? "● offline" : "● connecting"}
        </span>
      </div>
      {sys && (
        <div className="hdr-info">
          <span className="host">{sys.hostname}</span>
          <span>{sys.distro}</span>
          <span>{sys.kernel}</span>
          <span>{sys.cpu_model} ×{sys.n_cpu}</span>
          <span>up {fmtUptime(sys.uptime_s)}</span>
          <span>load {sys.loadavg.a.toFixed(2)} {sys.loadavg.b.toFixed(2)} {sys.loadavg.c.toFixed(2)}</span>
          {snap && <span className="muted">v{snap.meta.version} · {snap.meta.interval_ms} ms</span>}
        </div>
      )}
    </header>
  );
}
