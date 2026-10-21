//! `/proc/mounts` — per-device filesystem inventory + free space via statvfs(3).

use crate::collect::{FsInfo, SysPaths};
use std::ffi::CString;

pub struct RawMount {
    pub device: String,
    pub path: String,
    pub fstype: String,
}

pub fn raw(paths: &SysPaths) -> Vec<RawMount> {
    let mut out = Vec::new();
    if let Some(c) = paths.read("proc/mounts") {
        for line in c.lines() {
            // device, mount-point (may be %20-escaped), fstype, options…
            let f: Vec<&str> = line.split_whitespace().collect();
            if f.len() >= 3 {
                out.push(RawMount { device: f[0].to_string(), path: f[1].replace("%20", " "), fstype: f[2].to_string() });
            }
        }
    }
    out
}

/// Is `device` the disk itself or one of its partitions? (e.g. nvme0n1 vs nvme0n1p1)
fn dev_matches(disk: &str, device: &str) -> bool {
    let base = device.rsplit('/').next().unwrap_or(device);
    if base == disk {
        return true;
    }
    match base.strip_prefix(disk) {
        Some(sfx) => !sfx.is_empty() && sfx.chars().all(|c| c.is_ascii_digit() || c == 'p'),
        None => false,
    }
}

/// Filesystems mounted from `disk` (whole-disk or partition), deduped by path.
pub fn fs_for_disk(disk: &str, mounts: &[RawMount]) -> Vec<FsInfo> {
    let mut out: Vec<FsInfo> = Vec::new();
    for m in mounts {
        if !dev_matches(disk, &m.device) {
            continue;
        }
        if out.iter().any(|f: &FsInfo| f.path == m.path) {
            continue;
        }
        let (total_b, used_b, free_b) = usage_bytes(&m.path);
        let used_pct = if total_b == 0 { 0.0 } else { used_b as f64 / total_b as f64 * 100.0 };
        out.push(FsInfo { path: m.path.clone(), fstype: m.fstype.clone(), total_b, used_b, free_b, used_pct });
    }
    out
}

/// (total, used, free) bytes from statvfs. Missing/unreadable path → all zeros.
fn usage_bytes(path: &str) -> (u64, u64, u64) {
    let Ok(c) = CString::new(path) else { return (0, 0, 0) };
    unsafe {
        let mut st: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c.as_ptr(), &mut st) != 0 {
            return (0, 0, 0);
        }
        let fr = st.f_frsize as u64;
        let total = fr * st.f_blocks as u64;
        let free = fr * st.f_bavail as u64;
        let used = fr * (st.f_blocks as u64).saturating_sub(st.f_bfree as u64);
        (total, used, free)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::SysPaths;
    use std::path::PathBuf;

    fn fixture_paths() -> SysPaths {
        SysPaths { root: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/sys")) }
    }

    #[test]
    fn parses_mounts_fixture() {
        let p = fixture_paths();
        let ms = raw(&p);
        assert!(!ms.is_empty());
        assert!(ms.iter().any(|m| m.fstype == "ext4" || m.fstype == "btrfs" || m.fstype == "xfs"));
    }

    #[test]
    fn fs_for_disk_matches_partitions() {
        let p = fixture_paths();
        let ms = raw(&p);
        let fs = fs_for_disk("nvme0n1", &ms);
        assert!(fs.iter().any(|f| f.path != "/"), "root fs on nvme0n1 partition expected");
        assert!(fs.iter().all(|f| f.path.starts_with('/')));
    }

    #[test]
    fn dev_matches_rule() {
        assert!(dev_matches("nvme0n1", "/dev/nvme0n1"));
        assert!(dev_matches("nvme0n1", "/dev/nvme0n1p1"));
        assert!(!dev_matches("nvme0n1", "/dev/nvme0n2"));
        assert!(!dev_matches("nvme0n1", "tmpfs"));
    }

    #[test]
    fn usage_bytes_zero_for_missing() {
        assert_eq!(usage_bytes("/definitely/not/a/path/xyz"), (0, 0, 0));
    }
}