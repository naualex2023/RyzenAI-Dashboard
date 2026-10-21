//! `/proc/diskstats` — per-device counters, filtered to real block devices.

use super::SysPaths;

pub struct RawDisk {
    pub name: String,
    pub kind: &'static str,
    pub rotational: bool,
}

fn is_virtual(name: &str) -> bool {
    let v = ["loop", "dm-", "ram", "zram", "nbd", "sr", "null", "swqm", "veritas", "snapshot", "fd", "mtd", "vg"];
    v.iter().any(|p| name.starts_with(p))
}

pub fn raw(paths: &SysPaths) -> Vec<RawDisk> {
    let mut out = Vec::new();
    if let Some(content) = paths.read("proc/diskstats") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 9 {
                continue;
            }
            let name = parts[2].to_string();
            if is_virtual(&name) || paths.exists(&format!("sys/class/block/{name}/partition")) {
                continue;
            }
            let rotational = paths
                .read(&format!("sys/class/block/{name}/queue/rotational"))
                .map(|v| v.trim() != "0")
                .unwrap_or(false);
            let kind: &'static str = if name.starts_with("nvme") {
                "nvme"
            } else if name.starts_with("mmcblk") {
                "emmc"
            } else if name.starts_with("vd") {
                "virt"
            } else if rotational {
                "hdd"
            } else {
                "sata"
            };
            out.push(RawDisk { name, kind, rotational });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::SysPaths;
    use std::path::PathBuf;

    #[test]
    fn filters_partitions_and_virtual() {
        let p = SysPaths { root: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/sys")) };
        let disks = raw(&p);
        let names: Vec<&str> = disks.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"nvme0n1"));
        assert!(!names.contains(&"nvme0n1p1"), "partition must be filtered");
        assert!(!names.contains(&"loop0"));
        assert_eq!(disks.iter().find(|d| d.name == "nvme0n1").unwrap().kind, "nvme");
        assert!(!disks.iter().find(|d| d.name == "nvme0n1").unwrap().rotational);
    }
}