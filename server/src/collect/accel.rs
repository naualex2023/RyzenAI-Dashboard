//! `/sys/class/accel` — NPU clocks (Ryzen AI Max+: accel_dev0).

use super::SysPaths;

pub struct RawAccel {
    pub source: String,
    pub freq_mhz: Option<f64>,
    pub power_w: Option<f64>,
}

pub fn scan(paths: &SysPaths) -> Option<RawAccel> {
    for idx in 0..8 {
        let base = format!("sys/class/accel/accel_dev{idx}");
        if !paths.exists(&base) {
            break;
        }
        let raw_source = paths
            .read(&format!("{base}/device/name"))
            .or_else(|| paths.read(&format!("{base}/name")))
            .unwrap_or_else(|| format!("accel_dev{idx}"));
        // device/name reads like `npu1:npu1_6` — keep the part after the colon.
        let source = match raw_source.trim().split_once(':') {
            Some((_, rest)) => rest.to_string(),
            None => raw_source.trim().to_string(),
        };
        let freq_mhz = clock_freq(paths, &base);
        let power_w = paths
            .read(&format!("{base}/power"))
            .and_then(|v| v.trim().parse::<f64>().ok())
            .map(|v| v / 1_000_000.0);
        return Some(RawAccel { source, freq_mhz, power_w });
    }
    None
}

/// `clocks/clock_freq` reads like `c: 1960000000` (Hz, decimal) or `c: 0x3f84000`.
fn clock_freq(paths: &SysPaths, base: &str) -> Option<f64> {
    let v = paths.read(&format!("{base}/clocks/clock_freq"))?;
    for line in v.lines() {
        let seg: Vec<&str> = line.split_whitespace().collect();
        if seg.len() >= 2 {
            // decimal first (Hz); hex form as fallback for `0x…` tokens
            if seg.len() >= 2 {
                let tok = seg[seg.len() - 1].trim_start_matches("0x").to_string();
                let hz = if let Ok(d) = tok.parse::<u64>() {
                    d
                } else {
                    u64::from_str_radix(&tok, 16).unwrap_or(0)
                };
                if hz > 0 {
                    return Some(hz as f64 / 1_000_000.0);
                }
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
    fn npu_absent_on_dev_fixture() {
        let p = SysPaths { root: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/sys")) };
        assert!(scan(&p).is_none(), "no accel device on this machine");
    }

    #[test]
    fn parses_synthetic_accel() {
        let tmp = std::env::temp_dir().join("ryzenai-accel-test");
        let _ = std::fs::remove_dir_all(&tmp);
        let base = tmp.join("sys/class/accel/accel_dev0");
        std::fs::create_dir_all(base.join("clocks")).unwrap();
        std::fs::create_dir_all(base.join("device")).unwrap();
        std::fs::write(base.join("device/name"), "npu1:npu1_6\n").unwrap();
        std::fs::write(base.join("clocks/clock_freq"), "c: 1960000000\n").unwrap();
        std::fs::write(base.join("power"), "3500000\n").unwrap();
        let p = SysPaths { root: tmp.clone() };
        let a = scan(&p).expect("accel found");
        assert_eq!(a.source, "npu1_6");
        assert!(a.freq_mhz.unwrap() > 0.0);
        assert!((a.power_w.unwrap() - 3.5).abs() < 0.01);
        let _ = std::fs::remove_dir_all(&tmp);
    }
}