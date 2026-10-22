mod args;
mod collect;
mod history;
mod http;
mod mapping;
mod state;
mod ui_assets {
    include!(concat!(env!("OUT_DIR"), "/ui_assets.rs"));
}

use std::sync::Arc;

fn main() {
    let cfg = args::parse();
    let app = Arc::new(state::App::new(&cfg));

    let collector_app = app.clone();
    std::thread::Builder::new()
        .name("collector".into())
        .spawn(move || state::run_collector(collector_app))
        .expect("spawn collector thread");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    rt.block_on(async move {
        let addr = format!("{}:{}", cfg.bind_addr, cfg.port);
        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("error: bind {addr}: {e}");
                std::process::exit(1);
            }
        };
        let shown = if cfg.bind_addr == "0.0.0.0" {
            format!(
                "http://0.0.0.0:{0}  (all interfaces: http://localhost:{0}, http://<lan-ip>:{0})",
                cfg.port
            )
        } else {
            format!("http://{addr}")
        };
        println!(
            "ryzenai-dashboard v{}: listening on {shown} (ui: {})",
            env!("CARGO_PKG_VERSION"),
            if cfg.static_dir.is_some() {
                "filesystem dir".to_string()
            } else {
                "embedded".to_string()
            }
        );
        let router = http::router(app);
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .expect("server");
    });
}

async fn shutdown_signal() {
    use tokio::signal::unix::{signal, SignalKind};
    if let Ok(mut s) = signal(SignalKind::terminate()) {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = s.recv() => {}
        }
        return;
    }
    tokio::signal::ctrl_c().await.ok();
}