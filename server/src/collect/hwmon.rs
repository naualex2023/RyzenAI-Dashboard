//! `/sys/class/hwmon` — generic discovery + curated classification.

use crate::mapping::classify;
use super::{SysPaths, TempSensor};

pub fn scan(paths: &SysPaths) -> Vec<TempSensor> {
    let mut out = Vec::new();
    for idx in 0..64 {
        let base = format!("sys/class/hwmon/hwmon{idx}");
        if !paths.exists(&base) {
            break;
        }
        let chip = paths.read(&format!("{base}/name")).unwrap_or_default();
        if chip.is_empty() {
            continue;
        }
        let device = paths.canonical(&format!("{base}/device"));
        for f in list_files(paths, &base) {
            let file = f.rsplit('/').next().unwrap_or("").to_string();
            if !file.starts_with("temp") || !file.ends_with("_input") {
                continue;
            }
            let Some(millideg) = read_f(paths, &f) else { continue };
            let temp_c = millideg / 1000.0;
            if temp_c <= 0.0 || temp_c > 150.0 {
                continue;
            }
            let label = paths
                .read(&format!("{base}/{}_label", file.trim_end_matches("_input")))
                .unwrap_or_else(|| label_of_chip(&chip));
            let role = classify(&chip, &label);
            if role.as_str() == "misc" && chip_has_no_other_role(&chip) {
                continue; // noise: battery/PSU-style temp channels
            }
            out.push(TempSensor { id: format!("{base}.{file}"), role: role.as_str(), source: chip.clone(), label, device: device.clone(), temp_c: round2(temp_c) });
        }
    }
    out
}

fn chip_has_no_other_role(chip: &str) -> bool {
    !["k10temp", "amdgpu", "nvme", "npu", "soc", "pch"].iter().any(|k| chip.contains(k))
}

fn label_of_chip(chip: &str) -> String {
    chip.to_string()
}

/// Optional GPU power (amdgpu hwmon `power1_input` [µW] or `power1_average` [µW]).
pub fn gpu_power(paths: &SysPaths) -> Option<f64> {
    for idx in 0..64 {
        let base = format!("sys/class/hwmon/hwmon{idx}");
        if !paths.exists(&base) {
            break;
        }
        if paths.read(&format!("{base}/name")).unwrap_or_default() != "amdgpu" {
            continue;
        }
        for suffix in ["input", "average"] {
            let f = format!("{base}/power1_{suffix}");
            if let Some(v) = read_f(paths, &f) {
                if v > 0.0 {
                    return Some(v / 1_000_000.0); // µW → W
                }
            }
        }
    }
    None
}

/// (no helper) — mod.rs matches per-disk temps via `TempSensor.device`.

fn list_files(paths: &SysPaths, base: &str) -> Vec<String> {
    // SysPaths is text-read based; enumerate via a conventional range instead.
    let mut out = Vec::new();
    for t in 1..32 {
        for suffix in ["input", "label"] {
            let f = format!("{base}/temp{t}_{suffix}");
            if paths.exists(&f) {
                out.push(f);
            }
        }
    }
    out
}

fn read_f(paths: &SysPaths, f: &str) -> Option<f64> {
    paths.read(f).and_then(|v| v.trim().parse::<f64>().ok())
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::SysPaths;
    use std::path::PathBuf;

    #[test]
    fn scans_hwmon_fixture() {
        let p = SysPaths { root: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/sys")) };
        let t = scan(&p);
        assert!(t.iter().any(|s| s.role == "cpu"), "k10temp → cpu");
        assert!(t.iter().any(|s| s.role == "gpu"), "amdgpu → gpu");
        assert!(t.iter().any(|s| s.role == "ssd"), "nvme → ssd");
        assert!(gpu_power(&p).unwrap_or(0.0) > 0.0, "amdgpu power1_input present");
        for s in t.iter() {
            assert!(s.temp_c > 0.0 && s.temp_c <= 150.0);
        }
    }
}