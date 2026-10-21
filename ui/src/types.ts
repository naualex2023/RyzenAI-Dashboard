//! Types mirroring the serde structs of `server/src/collect/mod.rs` —
//! field names must stay in lockstep with the Rust definitions.

export interface LoadAvg { a: number; b: number; c: number }

export interface Psi {
  some_10: number; some_60: number; some_300: number;
  full_10: number; full_60: number; full_300: number;
}

export interface TempSensor {
  id: string;
  role: string;
  source: string;
  label: string;
  device: string | null;
  temp_c: number;
}

export interface SystemInfo {
  hostname: string;
  distro: string;
  kernel: string;
  cpu_model: string;
  n_cpu: number;
  uptime_s: number;
  loadavg: LoadAvg;
}

export interface CoreStat {
  core: number;
  usage_pct: number;
  freq_mhz: number | null;
}

export interface CpuInfo {
  usage_pct: number;
  temp_c: number | null;
  per_core: CoreStat[];
  loadavg: LoadAvg;
  psi: Psi;
}

export interface MemoryInfo {
  total_b: number;
  used_b: number;
  available_b: number;
  cached_b: number;
  buffers_b: number;
  swap_total_b: number;
  swap_used_b: number;
  used_pct: number;
  psi: Psi;
}

export interface FsInfo {
  path: string;
  fstype: string;
  total_b: number;
  used_b: number;
  free_b: number;
  used_pct: number;
}

export interface DiskTemp { label: string; temp_c: number }

export interface DiskInfo {
  name: string;
  kind: string;
  rotational: boolean;
  fs: FsInfo[];
  temps: DiskTemp[];
}

export interface GpuInfo {
  card: string;
  busy_pct: number | null;
  vram_used_b: number | null;
  vram_total_b: number | null;
  power_w: number | null;
  temps: DiskTemp[];
}

export interface NpuInfo {
  source: string;
  freq_mhz: number | null;
  power_w: number | null;
}

export interface NetInfo {
  name: string;
  rx_kbps: number;
  tx_kbps: number;
  rx_pkts_s: number;
  tx_pkts_s: number;
  link_speed_mbps: number | null;
  operstate: string;
}

export interface ProcInfo {
  pid: number;
  name: string;
  cpu_pct: number;
  rss_kb: number;
}

export interface Processes {
  by_cpu: ProcInfo[];
  by_rss: ProcInfo[];
}

export interface Meta {
  version: string;
  interval_ms: number;
  ts_ms: number;
}

export interface Snapshot {
  system: SystemInfo;
  temps: TempSensor[];
  cpu: CpuInfo;
  memory: MemoryInfo;
  disks: DiskInfo[];
  gpu: GpuInfo | null;
  npu: NpuInfo | null;
  network: NetInfo[];
  processes: Processes;
  meta: Meta;
}

export interface HistoryResp {
  sec: number;
  interval_ms: number;
  timestamps: number[];
  series: Record<string, (number | null)[]>;
}
