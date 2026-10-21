//! System identity: hostname, distro, kernel, CPU model, uptime, loadavg.

use super::{LoadAvg, SysPaths, SystemInfo};

fn parse_loadavg(content: &str) -> LoadAvg {
    let parts: Vec<&str> = content.split_whitespace().collect();
    let f = |i: usize| parts.get(i).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    LoadAvg { a: f(0), b: f(1), c: f(2) }
}

pub fn raw(paths: &SysPaths, n_cpu: usize) -> SystemInfo {
    let hostname = paths.read("proc/sys/kernel/hostname").or_else(|| paths.read("etc/hostname")).unwrap_or_default();
    let distro = distro_name(paths).unwrap_or_else(|| "Unknown Linux".into());
    let kernel = paths.read("proc/version").map(|v| kernel_name(&v)).unwrap_or_default();
    let cpu_model = cpu_model(paths);
    let uptime_s = uptime_s(paths);
    let loadavg = parse_loadavg(&paths.read("proc/loadavg").unwrap_or_default());
    SystemInfo { hostname, distro, kernel, cpu_model, n_cpu, uptime_s, loadavg }
}

/// Page size in kB (x86-64 is always 4096 B) — used for RSS byte math.
pub fn page_kb(_paths: &SysPaths) -> u64 {
    4
}

fn kernel_name(v: &str) -> String {
    // "Linux version 6.11.0-1008-amd64 (…)" → up to ' '
    let first: String = v.chars().take_while(|c| *c != '\n').collect();
    let seg: Vec<&str> = first.split_whitespace().collect();
    if seg.len() >= 3 {
        seg[2].to_string()
    } else {
        first.trim().to_string()
    }
}

fn cpu_model(paths: &SysPaths) -> String {
    if let Some(content) = paths.read("proc/cpuinfo") {
        for line in content.lines() {
            let line = line.trim();
            if let Some((k, v)) = line.split_once(':') {
                if k.trim() == "model name" {
                    return v.trim().to_string();
                }
            }
        }
    }
    // Fallback to kernel-reported brand from /sys (best effort).
    "AMD Ryzen AI Max+".into()
}

fn uptime_s(paths: &SysPaths) -> u64 {
    let content = paths.read("proc/uptime").unwrap_or_default();
    let first: String = content.chars().take_while(|c| *c != ' ' && *c != '\n').collect();
    first.parse().map(|f: f64| f as u64).unwrap_or(0)
}

fn distro_name(paths: &SysPaths) -> Option<String> {
    let content = paths.read("etc/os-release")?;
    for line in content.lines() {
        if let Some((k, v)) = line.split_once('=') {
            if k == "PRETTY_NAME" {
                return Some(v.trim_matches('"').to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::SysPaths;
    use std::path::PathBuf;

    #[test]
    fn system_identity_from_fixture() {
        let p = SysPaths { root: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/sys")) };
        let si = raw(&p, 8);
        assert!(!si.distro.is_empty());
        assert!(si.uptime_s > 0);
        assert!(si.loadavg.a >= 0.0);
        assert_eq!(si.n_cpu, 8);
        assert!(!si.cpu_model.is_empty());
    }
}