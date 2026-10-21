//! `/sys/class/drm/cardN` — AMD iGPU busy % + VRAM totals (direct sysfs).

use super::SysPaths;

pub struct RawGpu {
    pub card: String,
    pub busy: Option<f64>,
    pub vram_used: Option<u64>,
    pub vram_total: Option<u64>,
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
        // amdgpu perf/VRAM accounting may live under the card root or its device dir
        let busy = paths
            .read(&format!("{card}/device/perf/gpu_busy_percent"))
            .or_else(|| paths.read(&format!("{card}/perf/gpu_busy_percent")))
            .and_then(|v| parse_first_num(&v))
            .map(|v| v.clamp(0.0, 100.0));
        let vram_total = paths
            .read(&format!("{card}/device/memory/vram/size"))
            .or_else(|| paths.read(&format!("{card}/memory/vram/size")))
            .and_then(|v| first_u64(&v));
        let vram_used = paths
            .read(&format!("{card}/device/memory/vram/used"))
            .or_else(|| paths.read(&format!("{card}/memory/vram/used")))
            .and_then(|v| first_u64(&v));
        return Some(RawGpu { card: format!("card{idx}"), busy, vram_used, vram_total });
    }
    None
}

fn parse_first_num(s: &str) -> Option<f64> {
    let first: String = s.chars().take_while(|c| *c != '\n' && *c != ' ').collect();
    first.trim().parse().ok()
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
        assert!(g.busy.is_none(), "no perf counter on this machine");
        assert!(g.vram_total.is_none(), "no vram accounting on this machine");
    }
}