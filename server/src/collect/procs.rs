//! `/proc/[pid]/[stat,status,cmdline]` — top processes (CPU % + RSS).

use super::SysPaths;

pub struct RawProc {
    pub pid: u32,
    pub name: String,
    pub utime: u64,
    pub stime: u64,
    pub rss_kb: u64,
}

pub fn raw(paths: &SysPaths, page_kb: u64) -> Vec<RawProc> {
    let mut out = Vec::new();
    let entries = paths.canonical_list("proc");
    for dir in entries {
        let name = dir.rsplit('/').next().unwrap_or("");
        if name.parse::<u32>().is_err() {
            continue;
        }
        let stat = paths.read(&format!("{dir}/stat")).unwrap_or_default();
        let fields = split_stat(&stat);
        if fields.len() < 24 {
            continue;
        }
        let pid: u32 = fields[0].parse().unwrap_or(0);
        let mut name = fields[1].trim_start_matches('(').trim_end_matches(')').replace(' ', "·");
        if name.is_empty() {
            name = "?".into();
        }
        let utime: u64 = fields[13].parse().unwrap_or(0);
        let stime: u64 = fields[14].parse().unwrap_or(0);
        let rss_pages: u64 = fields[23].parse().unwrap_or(0);
        let rss_kb = rss_pages * page_kb;
        if rss_kb == 0 && utime == 0 && stime == 0 {
            continue;
        }
        out.push(RawProc { pid, name, utime, stime, rss_kb });
    }
    out
}

/// `/proc/<pid>/stat` may contain spaces/commas inside field 2 — split safely.
fn split_stat(stat: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut cur = String::new();
    let mut depth = 0;
    for ch in stat.chars() {
        match ch {
            '(' => {
                depth += 1;
                cur.push(ch);
            }
            ')' => {
                depth -= 1;
                cur.push(ch);
            }
            ' ' | '\t' if depth == 0 => {
                if !cur.is_empty() {
                    fields.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(ch),
        }
    }
    if !cur.is_empty() {
        fields.push(cur);
    }
    fields
}

/// `SysPaths` has no directory listing; the caller gets relative subdirs like
/// `proc/42` from `SysPaths::canonical_list` (works on real /proc and fixtures).
#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::SysPaths;
    use std::path::PathBuf;

    #[test]
    fn splits_stat_correctly() {
        let stat = "1234 (proc a,b) S 1 1 1 0 -1 41 0 0 0 0 100 5 3 0 0 0 0 0 0 0 12";
        // field layout check: utime=idx13, stime=idx14, rss=idx23
        let f = split_stat(stat);
        assert_eq!(f[0], "1234");
        assert_eq!(f[1], "(proc a,b)");
        assert_eq!(f[13], "100");
        assert_eq!(f[14], "5");
        assert_eq!(f[23], "12");
    }

    #[test]
    fn parses_procs_from_fixture() {
        let p = SysPaths { root: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/sys")) };
        let ps = raw(&p, 4);
        assert!(ps.iter().any(|x| x.pid == 42));
        assert!(ps.iter().any(|x| x.pid == 43));
    }
}