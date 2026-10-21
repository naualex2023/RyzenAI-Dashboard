//! `/proc/net/dev` + sysfs link info — per-interface counters.

use super::SysPaths;

pub struct RawNet {
    pub name: String,
    pub rx: u64,
    pub tx: u64,
    pub rxp: u64,
    pub txp: u64,
    pub speed: Option<f64>,
    pub operstate: String,
}

fn is_ignored(name: &str) -> bool {
    name == "lo" || name.starts_with("veth") || name.starts_with("docker") || name.starts_with("br-") || name.starts_with("virbr")
}

pub fn raw(paths: &SysPaths) -> Vec<RawNet> {
    let mut out = Vec::new();
    if let Some(content) = paths.read("proc/net/dev") {
        for (i, line) in content.lines().enumerate() {
            if i < 2 {
                continue; // header lines
            }
            let line = line.trim();
            let (name, rest) = match line.split_once(':') {
                Some(x) => (x.0.trim().to_string(), x.1),
                None => {
                    let mut parts = line.split_whitespace();
                    let name = parts.next().unwrap_or("").to_string();
                    (name, parts.next().unwrap_or(""))
                }
            };
            if name.is_empty() || is_ignored(&name) {
                continue;
            }
            let cols: Vec<u64> = rest.split_whitespace().filter_map(|s| s.parse::<u64>().ok()).collect();
            if cols.len() < 16 {
                continue;
            }
            // -1 means "unknown" (virtual/bridge links) → no link speed reported
            let speed = paths
                .read(&format!("sys/class/net/{name}/speed"))
                .and_then(|v| v.parse::<f64>().ok())
                .filter(|v| *v > 0.0);
            let operstate = paths.read(&format!("sys/class/net/{name}/operstate")).unwrap_or_else(|| "unknown".into());
            out.push(RawNet { name, rx: cols[0], tx: cols[8], rxp: cols[1], txp: cols[9], speed, operstate });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::SysPaths;
    use std::path::PathBuf;

    #[test]
    fn parses_net_dev() {
        let p = SysPaths { root: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/sys")) };
        let nets = raw(&p);
        assert!(!nets.is_empty());
        assert!(!nets.iter().any(|n| n.name == "lo" || n.name == "docker0" || n.name.starts_with("veth") || n.name.starts_with("br-")), "virtual interfaces filtered");
        let w = nets.iter().find(|n| n.name == "wlp2s0").unwrap();
        assert_eq!(w.operstate, "up");
        let t = nets.iter().find(|n| n.name == "tailscale0").unwrap();
        assert!(t.speed.is_none(), "speed=-1 must become None");
    }
}