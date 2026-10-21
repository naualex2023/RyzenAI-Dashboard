//! `/proc/meminfo` parser (kB fields → map).

use super::SysPaths;
use std::collections::HashMap;

pub fn raw(paths: &SysPaths) -> HashMap<String, u64> {
    let mut out = HashMap::new();
    if let Some(content) = paths.read("proc/meminfo") {
        for line in content.lines() {
            let line = line.trim();
            let colon = line.find(':');
            if let Some(idx) = colon {
                let key = line[..idx].trim().to_string();
                let rest = line[idx + 1..].trim().trim_end_matches("kB").trim();
                if let Ok(v) = rest.parse::<u64>() {
                    out.insert(key, v);
                }
            }
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
    fn parses_meminfo() {
        let p = SysPaths { root: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/sys")) };
        let m = raw(&p);
        assert_eq!(m.get("MemTotal").copied(), Some(16_000_000));
        assert_eq!(m.get("MemAvailable").copied(), Some(8_000_000));
        assert!(m.get("SwapTotal").is_some());
    }
}