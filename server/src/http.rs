//! HTTP API: JSON snapshot / history / SSE stream + embedded-or-dir UI.

use crate::{args::Config, state::App};
use axum::extract::{Query, Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use tokio_stream::StreamExt;
use axum::Router;
use std::collections::HashMap;
use std::sync::Arc;

pub fn router(app: Arc<App>) -> Router {
    Router::new()
        .route("/api/snapshot", get(snapshot))
        .route("/api/history", get(history))
        .route("/api/events", get(events_sse))
        .route("/api/sysdump", get(sysdump))
        .fallback(fallback_static)
        .with_state(app)
}

// ---------- auth ----------

fn auth_ok(cfg: &Config, headers: &HeaderMap, q_token: Option<&str>) -> bool {
    let Some(token) = cfg.token.as_ref() else { return true };
    if q_token == Some(token.as_str()) {
        return true;
    }
    headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .map(|h| h.trim_start_matches("Bearer ").trim() == token.as_str())
        .unwrap_or(false)
}

// ---------- /api/snapshot ----------

async fn snapshot(State(app): State<Arc<App>>, Query(q): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
    if !auth_ok(&app.cfg, &headers, q.get("token").map(String::as_str)) {
        return (StatusCode::UNAUTHORIZED, "unauthorized").into_response();
    }
    match app.snap.read() {
        Ok(guard) => (StatusCode::OK, axum::Json(guard.as_ref())).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "state poisoned").into_response(),
    }
}

// ---------- /api/history ----------

#[derive(serde::Serialize)]
struct HistoryResp {
    sec: u64,
    interval_ms: u64,
    timestamps: Vec<u64>,
    series: HashMap<String, Vec<Option<f64>>>,
}

async fn history(State(app): State<Arc<App>>, Query(q): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
    if !auth_ok(&app.cfg, &headers, q.get("token").map(String::as_str)) {
        return (StatusCode::UNAUTHORIZED, "unauthorized").into_response();
    }
    let sec: u64 = q.get("sec").and_then(|v| v.parse::<u64>().ok()).unwrap_or(300).clamp(10, 3600);
    match app.hist.lock() {
        Ok(h) => {
            let (timestamps, series) = h.slice(sec, app.cfg.interval_ms);
            let series: HashMap<String, Vec<Option<f64>>> = series
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().map(|x| if x.is_nan() { None } else { Some(x) }).collect()))
                .collect();
            let resp = HistoryResp { sec, interval_ms: app.cfg.interval_ms, timestamps, series };
            (StatusCode::OK, axum::Json(resp)).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "state poisoned").into_response(),
    }
}

// ---------- /api/events (SSE) ----------

async fn events_sse(State(app): State<Arc<App>>, headers: HeaderMap, Query(q): Query<HashMap<String, String>>) -> Response {
    if !auth_ok(&app.cfg, &headers, q.get("token").map(String::as_str)) {
        return (StatusCode::UNAUTHORIZED, "unauthorized").into_response();
    }
    let interval = std::time::Duration::from_millis(app.cfg.interval_ms.max(200));
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(4);
    let app2 = app.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(interval);
        loop {
            tick.tick().await;
            let snap = app2.snap.read().unwrap_or_else(|e| e.into_inner()).clone();
            let json = serde_json::to_string(snap.as_ref()).unwrap_or_default();
            if tx.send(format!("data: {json}\n\n")).await.is_err() {
                break;
            }
        }
    });
    let stream = tokio_stream::wrappers::ReceiverStream::from(rx)
        .map(|s| Ok::<_, std::convert::Infallible>(bytes::Bytes::from(s)));
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .body(axum::body::Body::from_stream(stream))
        .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "stream").into_response())
}

// ---------- /api/sysdump (diagnostics: raw procfs/sysfs file contents) ----------

pub const SYSDUMP_FILES: &[&str] = &[
    "proc/stat",
    "proc/meminfo",
    "proc/pressure/cpu",
    "proc/pressure/memory",
    "proc/pressure/io",
    "proc/diskstats",
    "proc/mounts",
    "proc/net/dev",
    "proc/uptime",
    "proc/loadavg",
    "proc/sys/kernel/hostname",
    "etc/os-release",
];

async fn sysdump(State(app): State<Arc<App>>, Query(q): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
    if !auth_ok(&app.cfg, &headers, q.get("token").map(String::as_str)) {
        return (StatusCode::UNAUTHORIZED, "unauthorized").into_response();
    }
    let mut files: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    for path in SYSDUMP_FILES {
        if let Some(c) = app.sys.read(path) {
            files.insert(path.to_string(), serde_json::Value::String(c));
        }
    }
    // hwmon chip discovery is dynamic — include the names for diagnostics.
    for d in app.sys.canonical_list("sys/class/hwmon") {
        if let Some(c) = app.sys.read(&format!("{d}/name")) {
            files.insert(format!("{d}/name"), serde_json::Value::String(c));
        }
    }
    (StatusCode::OK, axum::Json(serde_json::Value::Object(files))).into_response()
}

// ---------- static UI: --static dir or embedded ui_assets, SPA fallback ----------

async fn fallback_static(State(app): State<Arc<App>>, req: Request) -> Response {
    let rel = req.uri().path().trim_start_matches('/').to_string();
    if rel.starts_with("api/") {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    if rel.is_empty() {
        return index_resp(&app);
    }
    if let Some(dir) = &app.cfg.static_dir {
        if let Ok(b) = std::fs::read(dir.join(&rel)) {
            return file_resp(b, mime_for(&rel));
        }
    }
    embedded(&rel)
}

fn index_resp(app: &App) -> Response {
    if let Some(dir) = &app.cfg.static_dir {
        if let Ok(b) = std::fs::read(dir.join("index.html")) {
            return file_resp(b, mime_for("index.html"));
        }
    }
    embedded("index.html")
}

fn embedded(rel: &str) -> Response {
    use crate::ui_assets::ASSETS;
    if let Some(a) = ASSETS.iter().find(|a| a.path == rel) {
        return file_resp(a.bytes.to_vec(), a.mime);
    }
    // SPA fallback: unknown paths get the single-page app entry point.
    if let Some(a) = ASSETS.iter().find(|a| a.path == "index.html") {
        return file_resp(a.bytes.to_vec(), a.mime);
    }
    (StatusCode::NOT_FOUND, "not found").into_response()
}

fn file_resp(bytes: Vec<u8>, mime: &str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", mime)
        .body(axum::body::Body::from(bytes))
        .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "body").into_response())
}

fn mime_for(rel: &str) -> &'static str {
    let e = rel.rsplit('.').next().unwrap_or("");
    match e {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css",
        "js" | "mjs" => "text/javascript",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "ico" => "image/x-icon",
        "txt" => "text/plain; charset=utf-8",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}