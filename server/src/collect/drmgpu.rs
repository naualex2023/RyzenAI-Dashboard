//! `/sys/class/drm/cardN` — AMD GPU busy %, VRAM/GTT accounting, clocks, DPM
//! state (direct sysfs; no external tools).

use super::SysPaths;

pub struct RawGpu {
    pub card: String,
    pub busy: Option<f64>,
    pub mem_busy: Option<f64>,
    pub vram_used: Option<u64>,
    pub vram_total: Option<u64>,
    pub vis_vram_used: Option<u64>,
    pub vis_vram_total: Option<u64>,
    pub gtt_used: Option<u64>,
    pub gtt_total: Option<u64>,
    pub sclk_mhz: Option<f64>,
    pub mclk_mhz: Option<f64>,
    pub dpm_state: Option<String>,
    pub perf_level: Option<String>,
    pub fan_rpm: Option<u64>,
}

pub fn scan(paths: &SysPaths) -> Option<RawGpu> {
    // Card numbering is not contiguous (card0 may be absent while card1 exists,
    // and vice versa) — scan all candidates instead of breaking on the first gap.
    for idx in 0..8 {
        let card = format!("sys/class/drm/card{idx}");
        if !paths.exists(&card) {
            continue;
        }
        let dev = paths.canonical(&format!("{card}/device"));
        if dev.is_none() {
            continue;
        }
        let vendor = paths
            .read(&format!("{card}/device/vendor"))
            .or_else(|| paths.read(&format!("{card}/vendor")))
            .map(|v| v.trim().to_lowercase())
            .unwrap_or_default();
        if !vendor.contains("1002") {
            continue; // not an AMD card
        }
        let p = |rel: &str| format!("{card}/device/{rel}");
        let read_num =
            |cands: Vec<String>| -> Option<u64> { read_first(paths, &cands).and_then(|v| first_u64(&v)) };
        // Load percentages: amdgpu keeps them under device/ (optionally perf/).
        let busy = read_first(
            paths,
            &[p("gpu_busy_percent"), p("perf/gpu_busy_percent"), format!("{card}/perf/gpu_busy_percent")],
        )
        .and_then(|v| parse_first_num(&v))
        .map(|v| v.clamp(0.0, 100.0));
        let mem_busy = read_first(paths, &[p("mem_busy_percent"), p("perf/mem_busy_percent")])
            .and_then(|v| parse_first_num(&v))
            .map(|v| v.clamp(0.0, 100.0));
        // Memory: amdgpu mem_info_* (bytes); generic DRM memory/ tree as fallback.
        let vram_used = read_num(vec![p("mem_info_vram_used"), p("memory/vram/used")]);
        let vram_total = read_num(vec![p("mem_info_vram_total"), p("memory/vram/size")]);
        let vis_vram_used = read_num(vec![p("mem_info_vis_vram_used"), p("memory/vis_vram/used")]);
        let vis_vram_total = read_num(vec![p("mem_info_vis_vram_total"), p("memory/vis_vram/size")]);
        let gtt_used = read_num(vec![p("mem_info_gtt_used"), p("memory/gtt/used")]);
        let gtt_total = read_num(vec![p("mem_info_gtt_total"), p("memory/gtt/size")]);
        // Current DPM level: the `*`-marked line of pp_dpm_sclk/pp_dpm_mclk.
        let sclk_mhz = paths.read(&p("pp_dpm_sclk")).and_then(|v| dpm_current_mhz(&v));
        let mclk_mhz = paths.read(&p("pp_dpm_mclk")).and_then(|v| dpm_current_mhz(&v));
        let dpm_state = read_first(paths, &[p("power_dpm_state")]);
        let perf_level = read_first(paths, &[p("power_dpm_force_performance_level")]);
        // Card-local hwmon (may differ from the global chip order): fan RPM.
        let fan_rpm = (0..4).find_map(|n| {
            paths.read(&p(&format!("hwmon/hwmon{n}/fan1_input"))).and_then(|v| first_u64(&v))
        });
        return Some(RawGpu {
            card: format!("card{idx}"),
            busy,
            mem_busy,
            vram_used,
            vram_total,
            vis_vram_used,
            vis_vram_total,
            gtt_used,
            gtt_total,
            sclk_mhz,
            mclk_mhz,
            dpm_state,
            perf_level,
            fan_rpm,
        });
    }
    None
}

fn parse_first_num(s: &str) -> Option<f64> {
    let first: String = s.chars().take_while(|c| *c != '\n' && *c != ' ').collect();
    first.trim().parse().ok()
}

/// First non-empty read among candidate sysfs paths (trimmed).
fn read_first(paths: &SysPaths, candidates: &[String]) -> Option<String> {
    for c in candidates {
        if let Some(v) = paths.read(c) {
            let t = v.trim().to_string();
            if !t.is_empty() {
                return Some(t);
            }
        }
    }
    None
}

/// Current level from a `pp_dpm_*` listing: the `*`-marked line, e.g.
/// `0: 200Mhz\n1: 1200Mhz *` → 1200.
fn dpm_current_mhz(s: &str) -> Option<f64> {
    for line in s.lines().filter(|l| l.contains('*')) {
        for tok in line.split_whitespace() {
            let low = tok.to_lowercase();
            let num = low.strip_suffix("mhz").or_else(|| low.strip_suffix('m')).unwrap_or(&low);
            if let Ok(v) = num.parse::<f64>() {
                return Some(v);
            }
        }
    }
    None
}

fn first_u64(s: &str) -> Option<u64> {
    for line in s.lines() {
        if let Ok(v) = line.trim().parse::<u64>() {
            return Some(v);
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
    fn reads_gpu_from_fixture() {
        let p = SysPaths { root: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/sys")) };
        let g = scan(&p).expect("amd card present");
        assert_eq!(g.card, "card1");
        assert_eq!(g.busy, Some(37.0));
        assert_eq!(g.mem_busy, Some(12.0));
        assert_eq!(g.vram_used, Some(1024 * 1024 * 1024));
        assert_eq!(g.vram_total, Some(5 * 1024 * 1024 * 1024));
        assert_eq!(g.gtt_total, Some(8 * 1024 * 1024 * 1024));
        assert_eq!(g.sclk_mhz, Some(1200.0));
        assert_eq!(g.mclk_mhz, Some(640.0));
        assert_eq!(g.dpm_state.as_deref(), Some("performance"));
        assert_eq!(g.perf_level.as_deref(), Some("auto"));
        assert!(g.fan_rpm.is_none(), "no fan on this (integrated) fixture GPU");
    }

    #[test]
    fn dpm_parser_takes_starred_level() {
        assert_eq!(dpm_current_mhz("0: 200Mhz *\n1: 1200Mhz\n"), Some(200.0));
        assert_eq!(dpm_current_mhz("0: 640MHz *"), Some(640.0));
        assert_eq!(dpm_current_mhz("0: 200Mhz\n"), None);
    }
}