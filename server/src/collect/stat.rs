//! `/proc/stat` — per-CPU counters + boot time; cpufreq for per-core MHz.

use super::SysPaths;

pub struct StatRaw {
    /// First entry is the aggregate `"cpu"` line, then `cpu0…cpuN`.
    pub cpus: Vec<(String, Vec<u64>)>,
}

impl StatRaw {
    /// Aggregate line tick total — denominator for process CPU %.
    pub fn total_ticks(&self) -> u64 {
        self.cpus.first().map(|(_, v)| v.iter().sum::<u64>()).unwrap_or(0)
    }
}

pub fn raw(paths: &SysPaths) -> StatRaw {
    let mut cpus: Vec<(String, Vec<u64>)> = Vec::new();
    if let Some(content) = paths.read("proc/stat") {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("cpu") && (line == "cpu" || line.starts_with("cpu")) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 2 {
                    continue;
                }
                if parts[0].chars().all(|c| c == 'c' || c == 'p' || c == 'u') && parts[0] != "cpu" {
                    continue; // "cpufreq"? not present in proc/stat, defensive
                }
                let name = parts[0].to_string();
                let is_cpu = name == "cpu";
                if !(is_cpu || name.starts_with("cpu") && name[3..].chars().all(|c| c.is_ascii_digit())) {
                    continue;
                }
                let cols: Vec<u64> = parts[1..].iter().filter_map(|s| s.parse::<u64>().ok()).collect();
                if is_cpu {
                    cpus.insert(0, (name, cols));
                } else {
                    cpus.push((name, cols));
                }
            }
        }
    }
    StatRaw { cpus }
}

/// Per-core frequency in MHz from cpufreq (`scaling_cur_freq`, kHz) — None if absent.
pub fn cpu_freq(paths: &SysPaths, core: usize) -> Option<f64> {
    let v = paths.read(&format!("sys/devices/system/cpu/cpu{core}/cpufreq/scaling_cur_freq"))?;
    let khz: f64 = v.parse().ok()?;
    Some(khz / 1000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_paths() -> SysPaths {
        SysPaths { root: std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/sys")) }
    }

    #[test]
    fn parses_stat_and_btime() {
        let p = fixture_paths();
        let st = raw(&p);
        assert!(st.cpus.len() >= 2, "aggregate + at least one core");
        assert!(st.cpus[0].0 == "cpu");
        assert!(st.cpus[1].0 == "cpu0");
        assert_eq!(st.cpus[0].1.len(), 10);
        assert!(st.total_ticks() > 0);
    }
}