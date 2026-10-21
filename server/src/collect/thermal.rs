//! `/sys/class/thermal/thermal_zoneN` — legacy zones (deduped against hwmon).

use crate::mapping::classify;
use super::{SysPaths, TempSensor};

pub fn scan(paths: &SysPaths) -> Vec<TempSensor> {
    let mut out = Vec::new();
    for idx in 0..16 {
        let base = format!("sys/class/thermal/thermal_zone{idx}");
        if !paths.exists(&base) {
            break;
        }
        let Some(temp_c) = paths.read(&format!("{base}/temp")).and_then(|v| v.parse::<f64>().ok()) else { continue };
        // thermal zone temps are in °C (some kernels: millidegrees — heuristic)
        let temp_c = if temp_c > 1000.0 { temp_c / 1000.0 } else { temp_c };
        if temp_c <= 0.0 || temp_c > 150.0 {
            continue;
        }
        let zone_type = paths.read(&format!("{base}/type")).unwrap_or_default();
        if zone_type.is_empty() {
            continue;
        }
        let role = classify(&zone_type, "");
        if role.as_str() == "misc" {
            continue; // thermal zones are a superset of hwmon; keep only matched roles
        }
        out.push(TempSensor { id: format!("{base}"), role: role.as_str(), source: zone_type.clone(), label: zone_type, device: None, temp_c });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::SysPaths;
    use std::path::PathBuf;

    #[test]
    fn scans_thermal_zones() {
        let p = SysPaths { root: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/sys")) };
        let t = scan(&p);
        assert!(t.iter().any(|s| s.role == "ambient"), "acpitz zone → ambient");
        assert!(t.iter().all(|s| s.temp_c > 0.0 && s.temp_c <= 150.0));
    }
}