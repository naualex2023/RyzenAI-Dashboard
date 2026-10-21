//! Direct sysfs/procfs collectors (no root, no external tools).
//!
//! Each submodule reads raw kernel files; this module orchestrates one full
//! scan per tick and computes deltas (usage %, rates, process CPU %).

pub mod accel;
pub mod diskstats;
pub mod drmgpu;
pub mod hwmon;
pub mod meminfo;
pub mod mounts;
pub mod net;
pub mod pressure;
pub mod procs;
pub mod stat;
pub mod sysinfo;
pub mod thermal;

use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Serialize, Default, Clone, Debug)]
pub struct TempSensor {
    pub id: String,
    pub role: &'static str,
    pub source: String,
    pub label: String,
    pub device: Option<PathBuf>,
    pub temp_c: f64,
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct LoadAvg {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct Psi {
    pub some_10: f64,
    pub some_60: f64,
    pub some_300: f64,
    pub full_10: f64,
    pub full_60: f64,
    pub full_300: f64,
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct SystemInfo {
    pub hostname: String,
    pub distro: String,
    pub kernel: String,
    pub cpu_model: String,
    pub n_cpu: usize,
    pub uptime_s: u64,
    pub loadavg: LoadAvg,
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct CoreStat {
    pub core: usize,
    pub usage_pct: f64,
    pub freq_mhz: Option<f64>,
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct CpuInfo {
    pub usage_pct: f64,
    pub temp_c: Option<f64>,
    pub per_core: Vec<CoreStat>,
    pub loadavg: LoadAvg,
    pub psi: Psi,
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct MemoryInfo {
    pub total_b: u64,
    pub used_b: u64,
    pub available_b: u64,
    pub cached_b: u64,
    pub buffers_b: u64,
    pub swap_total_b: u64,
    pub swap_used_b: u64,
    pub used_pct: f64,
    pub psi: Psi,
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct DiskTemp {
    pub label: String,
    pub temp_c: f64,
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct FsInfo {
    pub path: String,
    pub fstype: String,
    pub total_b: u64,
    pub used_b: u64,
    pub free_b: u64,
    pub used_pct: f64,
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct DiskInfo {
    pub name: String,
    pub kind: &'static str,
    pub rotational: bool,
    pub fs: Vec<FsInfo>,
    pub temps: Vec<DiskTemp>,
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct GpuInfo {
    pub card: String,
    pub busy_pct: Option<f64>,
    pub vram_used_b: Option<u64>,
    pub vram_total_b: Option<u64>,
    pub power_w: Option<f64>,
    pub temps: Vec<DiskTemp>,
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct NpuInfo {
    pub source: String,
    pub freq_mhz: Option<f64>,
    pub power_w: Option<f64>,
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct NetInfo {
    pub name: String,
    pub rx_kbps: f64,
    pub tx_kbps: f64,
    pub rx_pkts_s: f64,
    pub tx_pkts_s: f64,
    pub link_speed_mbps: Option<f64>,
    pub operstate: String,
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct ProcInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_pct: f64,
    pub rss_kb: u64,
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct Processes {
    pub by_cpu: Vec<ProcInfo>,
    pub by_rss: Vec<ProcInfo>,
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct Meta {
    pub version: String,
    pub interval_ms: u64,
    pub ts_ms: u64,
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct Snapshot {
    pub system: SystemInfo,
    pub temps: Vec<TempSensor>,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskInfo>,
    pub gpu: Option<GpuInfo>,
    pub npu: Option<NpuInfo>,
    pub network: Vec<NetInfo>,
    pub processes: Processes,
    pub meta: Meta,
}

/// Virtual FS root: `/` in production, `server/fixtures/sys` in tests.
pub struct SysPaths {
    pub root: PathBuf,
}

impl SysPaths {
    /// Resolved path: root `/` maps to the real system, otherwise rel joins under root.
    /// Absolute paths for `/` avoid CWD-relative resolution in production.
    fn path(&self, rel: &str) -> PathBuf {
        let rel = rel.trim_start_matches('/');
        if self.root.display().to_string() == "/" {
            PathBuf::from("/").join(rel)
        } else {
            self.root.join(rel)
        }
    }
    /// Read a sysfs/procfs file relative to the root. Missing → None.
    pub fn read(&self, rel: &str) -> Option<String> {
        std::fs::read_to_string(self.path(rel)).ok().map(|s| s.trim().to_string())
    }
    pub fn exists(&self, rel: &str) -> bool {
        self.path(rel).exists()
    }
    pub fn canonical(&self, rel: &str) -> Option<PathBuf> {
        std::fs::canonicalize(self.path(rel)).ok()
    }
    /// Directory listing of `rel`'s subentries as `{rel}/{name}` strings
    /// (paths stay relative to the root in dev/fixture mode too).
    pub fn canonical_list(&self, rel: &str) -> Vec<String> {
        let mut out = Vec::new();
        if let Ok(rd) = std::fs::read_dir(self.path(rel)) {
            for e in rd.filter_map(|e| e.ok()) {
                if let Some(f) = e.path().file_name().map(|f| f.to_string_lossy()) {
                    out.push(format!("{rel}/{f}"));
                }
            }
        }
        out.sort();
        out
    }
}

/// Previous-scan counters used for rate/usage computation.
pub struct Delta {
    pub ts: Option<Instant>,
    pub cpu: Vec<Vec<u64>>,
    pub cpu_total: Vec<u64>,
    pub net: HashMap<String, NetCounters>,
    pub procs: HashMap<u32, (u64, u64)>,
    pub sys_ticks: u64,
}

#[derive(Default, Clone, Debug)]
pub struct NetCounters {
    pub rx: u64,
    pub tx: u64,
    pub rxp: u64,
    pub txp: u64,
}

impl Default for Delta {
    fn default() -> Self {
        Delta {
            ts: None,
            cpu: Vec::new(),
            cpu_total: Vec::new(),
            net: HashMap::new(),
            procs: HashMap::new(),
            sys_ticks: 0,
        }
    }
}

pub fn scan(paths: &SysPaths, delta: &mut Delta, version: &str, interval_ms: u64, ts_ms: u64) -> Snapshot {
    let now = Instant::now();
    let elapsed_ms = delta.ts.map(|t| now.duration_since(t).as_millis() as f64).unwrap_or(0.0);
    let elapsed_s = elapsed_ms / 1000.0;

    // --- CPU ---
    let st = stat::raw(paths);
    let mut per_core: Vec<CoreStat> = Vec::new();
    for (i, (_, cur)) in st.cpus.iter().skip(1).enumerate() {
        let usage = match delta.cpu.get(i) {
            Some(prev) => pct(prev, cur),
            None => 0.0,
        };
        per_core.push(CoreStat { core: i, usage_pct: usage, freq_mhz: stat::cpu_freq(paths, i) });
    }
    let total_usage = pct(&delta.cpu_total, &st.cpus[0].1);
    delta.cpu = st.cpus.iter().skip(1).map(|(_, v)| v.clone()).collect();
    delta.cpu_total = st.cpus[0].1.clone();

    // --- Memory ---
    let mem = meminfo::raw(paths);
    let mem_kb = |k: &str| mem.get(k).copied().unwrap_or(0);
    let total_b = mem_kb("MemTotal") * 1024;
    let available_b = mem_kb("MemAvailable") * 1024;
    let cached_b = mem_kb("Cached") * 1024;
    let buffers_b = mem_kb("Buffers") * 1024;
    let swap_total_b = mem_kb("SwapTotal") * 1024;
    let swap_used_b = swap_total_b.saturating_sub(mem_kb("SwapFree") * 1024);
    let used_b = total_b.saturating_sub(available_b);
    let used_pct = if total_b > 0 { (used_b as f64 / total_b as f64) * 100.0 } else { 0.0 };
    let mem_psi = pressure::raw(paths, "memory").unwrap_or_default();
    let cpu_psi = pressure::raw(paths, "cpu").unwrap_or_default();

    // --- Temperature discovery (hwmon + thermal zones, deduped) ---
    let mut temps = hwmon::scan(paths);
    let tz = thermal::scan(paths);
    dedupe_temps(&mut temps, tz);

    // --- Disks: physical devices + filesystem free space + temperatures ---
    let disks_raw = diskstats::raw(paths);
    let mounts_raw = mounts::raw(paths);
    let mut disks: Vec<DiskInfo> = Vec::with_capacity(disks_raw.len());
    for d in disks_raw.iter() {
        let fs = mounts::fs_for_disk(&d.name, &mounts_raw);
        disks.push(DiskInfo { name: d.name.clone(), kind: d.kind, rotational: d.rotational, fs, temps: Vec::new() });
    }
    attach_disk_temps(paths, &mut disks, &temps);

    // --- Network ---
    let nets = net::raw(paths);
    let mut network: Vec<NetInfo> = Vec::with_capacity(nets.len());
    for n in nets.iter() {
        let prev = delta.net.get(&n.name);
        let cur = NetCounters { rx: n.rx, tx: n.tx, rxp: n.rxp, txp: n.txp };
        let (rx_kbps, tx_kbps, rx_pkts_s, tx_pkts_s) = net_rates(prev, &cur, elapsed_s);
        network.push(NetInfo { name: n.name.clone(), rx_kbps, tx_kbps, rx_pkts_s, tx_pkts_s, link_speed_mbps: n.speed, operstate: n.operstate.clone() });
        delta.net.insert(n.name.clone(), cur);
    }

    // --- GPU / NPU ---
    let gpu_raw = drmgpu::scan(paths);
    let gpu_power = hwmon::gpu_power(paths);
    let gpu_temps: Vec<DiskTemp> = temps
        .iter()
        .filter(|t| t.role == "gpu")
        .map(|t| DiskTemp { label: t.label.clone(), temp_c: t.temp_c })
        .collect();
    let gpu = gpu_raw.map(|g| GpuInfo {
        card: g.card,
        busy_pct: g.busy,
        vram_used_b: g.vram_used,
        vram_total_b: g.vram_total,
        power_w: gpu_power,
        temps: gpu_temps,
    });
    let npu = accel::scan(paths).map(|a| NpuInfo { source: a.source, freq_mhz: a.freq_mhz, power_w: a.power_w });

    // --- Processes ---
    let page_kb = sysinfo::page_kb(paths);
    let procs_raw = procs::raw(paths, page_kb);
    let sys_delta_ticks = st.total_ticks().saturating_sub(delta.sys_ticks);
    let mut plist: Vec<ProcInfo> = Vec::with_capacity(procs_raw.len());
    for p in procs_raw.iter() {
        let prev = delta.procs.get(&p.pid);
        let (prev_u, prev_s) = prev.copied().unwrap_or((p.utime, p.stime));
        let cur = p.utime + p.stime;
        let prev_t = prev_u + prev_s;
        let cpu_pct = if sys_delta_ticks > 0 {
            ((cur.saturating_sub(prev_t)) as f64 / sys_delta_ticks as f64 * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };
        plist.push(ProcInfo { pid: p.pid, name: p.name.clone(), cpu_pct, rss_kb: p.rss_kb });
    }
    delta.procs = procs_raw.iter().map(|p| (p.pid, (p.utime, p.stime))).collect();

    plist.sort_by(|a, b| b.cpu_pct.partial_cmp(&a.cpu_pct).unwrap_or(std::cmp::Ordering::Equal));
    let by_cpu: Vec<ProcInfo> = plist.iter().take(8).cloned().collect();
    plist.sort_by(|a, b| b.rss_kb.cmp(&a.rss_kb));
    let by_rss: Vec<ProcInfo> = plist.iter().take(8).cloned().collect();
    let processes = Processes { by_cpu, by_rss };

    // --- System ---
    let n_cpu = delta.cpu.len();
    let system = sysinfo::raw(paths, n_cpu);
    let cpu_temp_c = temps.iter().find(|t| t.role == "cpu").map(|t| t.temp_c);
    let loadavg = system.loadavg.clone();
    delta.sys_ticks = st.total_ticks();

    Snapshot {
        system,
        temps,
        cpu: CpuInfo { usage_pct: total_usage, temp_c: cpu_temp_c, per_core, loadavg, psi: cpu_psi },
        memory: MemoryInfo { total_b, used_b, available_b, cached_b, buffers_b, swap_total_b, swap_used_b, used_pct, psi: mem_psi },
        disks,
        gpu,
        npu,
        network,
        processes,
        meta: Meta { version: version.to_string(), interval_ms, ts_ms },
    }
}

fn busy_parts(cols: &[u64]) -> (u64, u64) {
    let total: u64 = cols.iter().sum();
    let busy = cols.get(0).copied().unwrap_or(0)
        + cols.get(1).copied().unwrap_or(0)
        + cols.get(2).copied().unwrap_or(0)
        + cols.get(5).copied().unwrap_or(0)
        + cols.get(6).copied().unwrap_or(0)
        + cols.get(7).copied().unwrap_or(0);
    (busy, total)
}

fn pct(prev: &Vec<u64>, cur: &Vec<u64>) -> f64 {
    if prev.len() != cur.len() || prev.is_empty() {
        return 0.0;
    }
    let (cb, ct) = busy_parts(cur);
    let (pb, pt) = busy_parts(prev);
    let d_busy = cb.saturating_sub(pb) as f64;
    let d_total = ct.saturating_sub(pt) as f64;
    if d_total <= 0.0 {
        return 0.0;
    }
    ((d_busy / d_total) * 100.0).clamp(0.0, 100.0)
}

fn net_rates(prev: Option<&NetCounters>, cur: &NetCounters, elapsed_s: f64) -> (f64, f64, f64, f64) {
    let p = prev.cloned().unwrap_or_default();
    let d = |a: u64, b: u64| (a.saturating_sub(b)) as f64;
    let e = elapsed_s.max(0.001);
    (d(cur.rx, p.rx) / 1024.0 / e, d(cur.tx, p.tx) / 1024.0 / e, d(cur.rxp, p.rxp) / e, d(cur.txp, p.txp) / e)
}

fn dedupe_temps(hw: &mut Vec<TempSensor>, tz: Vec<TempSensor>) {
    for t in tz {
        // Deduplicate identical sensors reported via both subsystems (e.g. hwmon
        // `acpitz_0` vs thermal zone `acpitz`): same id, or same role + |ΔT| < 0.5.
        let dup = hw.iter().any(|h| {
            h.id == t.id || (h.role == t.role && (h.temp_c - t.temp_c).abs() < 0.5)
        });
        if !dup {
            hw.push(t);
        }
    }
    hw.sort_by(|a, b| role_rank(a.role).cmp(&role_rank(b.role)).then(a.id.cmp(&b.id)));
}

fn role_rank(r: &str) -> u8 {
    match r {
        "ssd" => 0,
        "cpu" => 1,
        "gpu" => 2,
        "npu" => 3,
        "soc" => 4,
        "vrm" => 5,
        "chipset" => 6,
        "ambient" => 7,
        _ => 8,
    }
}

fn attach_disk_temps(paths: &SysPaths, disks: &mut [DiskInfo], temps: &[TempSensor]) {
    let ssd_temps: Vec<(TempSensor, PathBuf)> = temps
        .iter()
        .filter(|t| t.role == "ssd")
        .filter_map(|t| t.device.clone().map(|d| (t.clone(), d)))
        .collect();
    if ssd_temps.is_empty() {
        return;
    }
    for d in disks.iter_mut() {
        let disk_path = match paths.canonical(&format!("sys/class/block/{}", d.name)) {
            Some(p) => p,
            None => continue,
        };
        for (t, hw_dev) in ssd_temps.iter() {
            let matched = disk_path.starts_with(hw_dev.as_path())
                || hw_dev.file_name().and_then(|b| b.to_str()).map(|b| d.name.starts_with(b)).unwrap_or(false);
            if matched {
                d.temps.push(DiskTemp { label: t.label.clone(), temp_c: t.temp_c });
            }
        }
        d.temps.sort_by(|a, b| a.label.cmp(&b.label));
    }
}