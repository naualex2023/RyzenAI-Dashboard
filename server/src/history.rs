//! In-RAM ring buffers per numeric series + JSON leaf flattening.
//!
//! Every tick each numeric leaf of the snapshot (excluding volatile groups)
//! is appended to a per-series ring. `/api/history?sec=N` serves slices with
//! NaN padding where a series is younger than the window — the UI renders
//! gaps, memory stays bounded by `history_ticks`.

use std::collections::HashMap;

pub const MAX_SERIES: usize = 512;

pub struct Ring {
    data: Vec<f64>,
    cap: usize,
    head: usize,
    len: usize,
}

impl Ring {
    pub fn new(cap: usize) -> Self {
        Ring { data: Vec::new(), cap, head: 0, len: 0 }
    }

    pub fn push(&mut self, v: f64) {
        if self.data.len() < self.cap {
            self.data.push(v);
            self.len = self.data.len();
            if self.len == self.cap {
                self.head = 0;
            }
            return;
        }
        self.data[self.head] = v;
        self.head = (self.head + 1) % self.cap;
    }

    /// Newest `n` values, oldest first, NaN-padded at the front if needed.
    pub fn last(&self, n: usize) -> Vec<f64> {
        let n = n.min(self.len);
        let mut out: Vec<f64> = Vec::with_capacity(n);
        for i in (self.len - n)..self.len {
            let idx = (i + self.head) % self.len.max(1);
            out.push(self.data[idx]);
        }
        out
    }
}

pub struct History {
    pub ts: Vec<u64>,
    pub ts_head: usize,
    pub ts_len: usize,
    pub cap: usize,
    pub series: HashMap<String, Ring>,
    pub dropped: bool,
}

impl History {
    pub fn new(cap: usize) -> Self {
        History { ts: Vec::new(), ts_head: 0, ts_len: 0, cap, series: HashMap::new(), dropped: false }
    }

    pub fn push_tick(&mut self, ts_ms: u64, entries: &[(String, f64)]) {
        if self.ts.len() < self.cap {
            self.ts.push(ts_ms);
            self.ts_len = self.ts.len();
            if self.ts_len == self.cap {
                self.ts_head = 0;
            }
        } else {
            self.ts[self.ts_head] = ts_ms;
            self.ts_head = (self.ts_head + 1) % self.cap;
        }
        for (name, v) in entries {
            let ring = match self.series.get_mut(name) {
                Some(r) => r,
                None => {
                    if self.series.len() >= MAX_SERIES {
                        self.dropped = true;
                        continue;
                    }
                    self.series.insert(name.clone(), Ring::new(self.cap));
                    self.series.get_mut(name).unwrap()
                }
            };
            ring.push(*v);
        }
    }

    /// `sec`-seconds window: aligned timestamps + per-series values
    /// (oldest → newest, front-padded with NaN for younger series).
    pub fn slice(&self, sec: u64, interval_ms: u64) -> (Vec<u64>, HashMap<String, Vec<f64>>) {
        let n = (((sec * 1000) / interval_ms.max(1)) as usize).min(self.ts_len).max(1);
        let mut ts_out: Vec<u64> = Vec::with_capacity(n);
        for i in (self.ts_len - n)..self.ts_len {
            ts_out.push(self.ts[(i + self.ts_head) % self.ts_len]);
        }
        let mut out = HashMap::new();
        for (name, ring) in &self.series {
            let vals = ring.last(n);
            if vals.len() == n {
                out.insert(name.clone(), vals);
            } else {
                let mut padded: Vec<f64> = vec![f64::NAN; n - vals.len()];
                padded.extend(vals.iter().copied());
                out.insert(name.clone(), padded);
            }
        }
        (ts_out, out)
    }
}

/// Flatten a snapshot `Value` to (dotted-path, f64) pairs. Arrays of objects
/// are keyed by their `id`/`name` field when present (stable across rescans),
/// otherwise by index. Non-numeric leaves are skipped.
/// `deny_prefixes` filters out volatile groups (processes, system, meta…).
pub fn flatten(value: &serde_json::Value, deny_prefixes: &[&str]) -> Vec<(String, f64)> {
    let mut out = Vec::new();
    let root = match value {
        serde_json::Value::Object(m) => m,
        _ => return out,
    };
    for (k, v) in root {
        if deny_prefixes.iter().any(|p| *p == k.as_str()) {
            continue;
        }
        walk(v, k, &mut out);
    }
    out
}

fn walk(v: &serde_json::Value, path: &str, out: &mut Vec<(String, f64)>) {
    match v {
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if f.is_finite() {
                    out.push((path.to_string(), f));
                }
            }
        }
        serde_json::Value::Object(m) => {
            for (k, v) in m {
                walk(v, &format!("{path}.{k}"), out);
            }
        }
        serde_json::Value::Array(a) => {
            for (i, item) in a.iter().enumerate() {
                let key = match item {
                    serde_json::Value::Object(m) => m
                        .get("id")
                        .and_then(|x| x.as_str())
                        .or_else(|| m.get("name").and_then(|x| x.as_str()))
                        .map(sanitize)
                        .unwrap_or_else(|| i.to_string()),
                    _ => i.to_string(),
                };
                walk(item, &format!("{path}.{key}"), out);
            }
        }
        _ => {}
    }
}

fn sanitize(s: &str) -> String {
    if s.is_empty() {
        return s.to_string();
    }
    // Series keys keep dotted ids (`hwmon3.temp1`) verbatim; only path
    // separators and whitespace are flattened.
    s.replace('/', "_")
        .replace('\\', "_")
        .replace(' ', "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ring_wraps() {
        let mut r = Ring::new(4);
        for i in 0..9 {
            r.push(i as f64);
        }
        assert_eq!(r.last(4), vec![5.0, 6.0, 7.0, 8.0]);
    }

    #[test]
    fn history_aligns_and_pads() {
        let mut h = History::new(4);
        h.push_tick(0, &[]);
        h.push_tick(1000, &[("a".into(), 1.0)]);
        h.push_tick(2000, &[("a".into(), 2.0)]);
        let (ts, out) = h.slice(4, 1000);
        assert_eq!(ts, vec![0, 1000, 2000]);
        let a = out.get("a").unwrap();
        assert!(a[0].is_nan());
        assert_eq!(a[1], 1.0);
        assert_eq!(a[2], 2.0);
    }

    #[test]
    fn flatten_keys_by_id() {
        let v = json!({
            "temps": [
                {"id": "hwmon3.temp1", "temp_c": 30.85},
                {"id": "hwmon5.temp1", "temp_c": 42.0}
            ],
            "cpu": {"usage_pct": 7.5}
        });
        let flat = flatten(&v, &["processes", "system", "meta"]);
        assert!(flat.contains(&("temps.hwmon3.temp1.temp_c".to_string(), 30.85)));
        assert!(flat.contains(&("cpu.usage_pct".to_string(), 7.5)));
    }
}