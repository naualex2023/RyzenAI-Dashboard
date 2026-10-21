import { useState } from "react";
import { fetchSysdump, isUnauthorized } from "../api";

export function SysdumpCard() {
  const [files, setFiles] = useState<Record<string, string> | null>(null);
  const [err, setErr] = useState("");

  const load = () => {
    fetchSysdump()
      .then(setFiles)
      .catch((e) => setErr(isUnauthorized(e) ? "unauthorized — set token" : String(e.message)));
  };

  return (
    <section className="card" aria-label="sysdump">
      <h2>Sysdump <button className="btn" onClick={load}>fetch</button></h2>
      {err && <span className="tag warm">{err}</span>}
      {files && (
        <div className="sysdump">
          {Object.keys(files).sort().map((k) => (
            <details key={k}>
              <summary>{k}</summary>
              <pre>{files[k]}</pre>
            </details>
          ))}
        </div>
      )}
    </section>
  );
}
