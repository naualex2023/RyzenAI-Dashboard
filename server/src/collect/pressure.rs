//! `/proc/pressure/{cpu,memory,io}` — PSI averages (10s/60s/300s windows).

use super::{Psi, SysPaths};

pub fn raw(paths: &SysPaths, which: &str) -> Option<Psi> {
    let content = paths.read(&format!("proc/pressure/{which}"))?;
    let mut psi = Psi::default();
    for line in content.lines() {
        let line = line.trim();
        let kind = if line.starts_with("some ") {
            "some"
        } else if line.starts_with("full ") {
            "full"
        } else {
            continue;
        };
        let rest = line.split_once(' ').map(|(_, r)| r).unwrap_or("");
        for part in rest.split_whitespace() {
            let Some((key, val)) = part.split_once('=') else { continue };
            let Ok(val) = val.parse::<f64>() else { continue };
            match kind {
                "some" => match key {
                    "avg10" => psi.some_10 = val,
                    "avg60" => psi.some_60 = val,
                    "avg300" => psi.some_300 = val,
                    _ => {}
                },
                "full" => match key {
                    "avg10" => psi.full_10 = val,
                    "avg60" => psi.full_60 = val,
                    "avg300" => psi.full_300 = val,
                    _ => {}
                },
                _ => {}
            }
        }
    }
    Some(psi)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::SysPaths;
    use std::path::PathBuf;

    #[test]
    fn parses_psi() {
        let p = SysPaths { root: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/sys")) };
        let psi = raw(&p, "cpu").unwrap();
        assert!(psi.some_10 > 0.0);
        assert!(psi.full_300 >= 0.0);
    }
}