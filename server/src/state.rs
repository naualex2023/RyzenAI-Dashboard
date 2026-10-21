//! Shared app state: latest snapshot, history rings, and the sysfs root.

use crate::args::Config;
use crate::collect::{Snapshot, SysPaths};
use crate::history::History;
use std::sync::{Arc, Mutex, RwLock};

pub struct App {
    pub cfg: Config,
    pub sys: SysPaths,
    pub snap: RwLock<Arc<Snapshot>>,
    pub hist: Mutex<History>,
}

impl App {
    pub fn new(cfg: &Config) -> Self {
        App {
            cfg: cfg.clone(),
            sys: SysPaths { root: cfg.sys_root.clone() },
            snap: RwLock::new(Arc::new(Snapshot::default())),
            hist: Mutex::new(History::new(cfg.history_ticks)),
        }
    }
}

/// Collector thread: full scan per tick, publish snapshot + history series.
pub fn run_collector(app: Arc<App>) {
    let interval = std::time::Duration::from_millis(app.cfg.interval_ms.max(200));
    let version = env!("CARGO_PKG_VERSION").to_string();
    let mut delta = crate::collect::Delta::default();
    loop {
        let ts_ms = now_ms();
        let snapshot = crate::collect::scan(&app.sys, &mut delta, &version, app.cfg.interval_ms, ts_ms);
        if let Ok(mut w) = app.snap.write() {
            *w = Arc::new(snapshot.clone());
        }
        if let Ok(json) = serde_json::to_value(&snapshot) {
            if let Ok(mut h) = app.hist.lock() {
                h.push_tick(ts_ms, &crate::history::flatten(&json, HISTORY_DENY));
            }
        }
        std::thread::sleep(interval);
    }
}

/// Groups excluded from history series (volatile / non-telemetry data).
pub const HISTORY_DENY: &[&str] = &["processes", "system", "meta"];

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}