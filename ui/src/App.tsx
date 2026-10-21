import { useCallback, useEffect, useState } from "react";
import type { Snapshot } from "./types";
import { fetchSnapshot, getToken, setToken, eventsUrl, isUnauthorized } from "./api";
import { Header, type Conn } from "./components/Header";
import { CpuCard } from "./components/CpuCard";
import { MemoryCard } from "./components/MemoryCard";
import { TempsCard } from "./components/TempsCard";
import { DisksCard } from "./components/DisksCard";
import { AcceleratorCard } from "./components/AcceleratorCard";
import { NetCard } from "./components/NetCard";
import { ProcCard } from "./components/ProcCard";
import { HistoryCard } from "./components/HistoryCard";
import { SysdumpCard } from "./components/SysdumpCard";

function TokenGate({ onDone }: { onDone: () => void }) {
  const [t, setT] = useState("");
  return (
    <div className="gate">
      <p>API requires a token.</p>
      <input placeholder="token" value={t} onChange={(e) => setT(e.target.value)} />
      <button className="btn" onClick={() => { setToken(t); onDone(); }}>connect</button>
    </div>
  );
}

export function App() {
  const [snap, setSnap] = useState<Snapshot | null>(null);
  const [conn, setConn] = useState<Conn>("connecting");
  const [needToken, setNeedToken] = useState(false);
  const [err, setErr] = useState("");

  const bootstrap = useCallback(() => {
    if (!needToken) {
      fetchSnapshot()
        .then(setSnap)
        .catch((e) => {
          if (isUnauthorized(e)) setNeedToken(true);
          else setErr(String(e.message));
        });
    }
    // EventSource reconnects by itself; onmessage overrides stale state.
    const es = new EventSource(eventsUrl());
    es.onopen = () => { setConn("live"); setNeedToken(false); setErr(""); };
    es.onerror = () => setConn("offline");
    es.onmessage = (m) => {
      try {
        setSnap(JSON.parse(m.data) as Snapshot);
        setConn("live");
        setNeedToken(false);
      } catch {
        setConn("offline");
      }
    };
    return () => es.close();
  }, []);

  // Reconnect on token change (TokenGate onDone) — key off token itself.
  const [token, setTk] = useState(getToken());
  useEffect(() => bootstrap(), [token]);

  if (needToken) return <TokenGate onDone={() => { setNeedToken(false); setTk(getToken()); }} />;

  return (
    <div className="dash">
      <Header snap={snap} conn={conn} />
      {err && <div className="tag warm api-err">{err}</div>}
      {snap && (
        <main className="grid">
          <CpuCard snap={snap} />
          <MemoryCard snap={snap} />
          <TempsCard snap={snap} />
          <AcceleratorCard snap={snap} />
          <DisksCard snap={snap} />
          <NetCard snap={snap} />
          <ProcCard snap={snap} />
          <HistoryCard snap={snap} />
          <SysdumpCard />
        </main>
      )}
    </div>
  );
}
