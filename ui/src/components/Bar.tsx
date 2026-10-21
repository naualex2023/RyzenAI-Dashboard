import { clamp } from "../fmt";

export function Bar({ pct, warn = 80 }: { pct: number | null | undefined; warn?: number }) {
  const v = pct ?? 0;
  const cls = v >= warn ? "bar-fill hot" : v >= warn * 0.6 ? "bar-fill warm" : "bar-fill";
  return (
    <div className="bar">
      <div className={cls} style={{ width: `${clamp(v, 0, 100)}%` }} />
    </div>
  );
}
