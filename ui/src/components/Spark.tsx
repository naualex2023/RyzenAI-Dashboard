//! Minimal dependency-free sparkline: one SVG polyline, gaps on null points.

import { useId } from "react";

export function Spark({ data, height = 34, color = "var(--spark)" }:
  { data: (number | null)[]; height?: number; color?: string }) {
  const w = 240;
  const n = data.length;
  const vals = data.filter((x): x is number => x != null);
  if (n === 0 || vals.length === 0) return <svg className="spark" viewBox={`0 0 ${w} ${height}`} />;

  const min = Math.min(...vals);
  const max = Math.max(...vals);
  const span = max - min || 1;
  const pad = 2;
  const y = (v: number) => height - pad - ((v - min) / span) * (height - 2 * pad);
  const x = (i: number) => (n <= 1 ? w / 2 : (i / (n - 1)) * (w - 2)) + 1;

  const d: string[] = [];
  let seg: string[] = [];
  for (let i = 0; i < n; i++) {
    const v = data[i];
    if (v == null) {
      if (seg.length > 1) d.push(`M${seg.join(" L")}`);
      seg = [];
      continue;
    }
    seg.push(`${x(i).toFixed(2)},${y(v).toFixed(2)} `);
  }
  if (seg.length > 1) d.push(`M${seg.join(" L")}`);

  const key = useId();
  return (
    <svg className="spark" viewBox={`0 0 ${w} ${height}`} key={key} preserveAspectRatio="none">
      {d.map((p, i) => (
        <path key={i} d={p} fill="none" stroke={color} strokeWidth="1.4"
          strokeLinecap="round" strokeLinejoin="round" vectorEffect="non-scaling-stroke" />
      ))}
    </svg>
  );
}
