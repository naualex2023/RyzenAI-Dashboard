# RyzenAI Dashboard

A self-contained, dependency-free system dashboard for Ryzen AI–class Linux machines.
A single Rust binary (Axum + Tokio) reads `/proc` and `/sys` directly — no root,
no external tools, no daemons — and serves both the telemetry API and the
embedded React UI from one process.

## Features

- **CPU** — aggregate + per-core usage, per-core frequency, Tctl-class temp, load average, PSI
- **Memory** — used/available/cached/buffers/swap, memory PSI
- **Disks** — whole-device + partitions (nvme suffix-aware matching), per-filesystem usage via `statvfs`, SMART-style temps from hwmon, optional `--smartctl` probing
- **GPU / NPU** — AMD DRM busy %, VRAM, power, temps (`/sys/class/drm`); accelerator probes (`accel`)
- **Temps** — unified sensor grid with role classification (cpu / gpu / ssd / vrm / soc / chipset / ambient / misc), deduped across hwmon + thermal zones
- **Network** — per-interface rx/tx rates + packet rates, link speed, operstate
- **Processes** — top by CPU % and RSS (per-pid `/proc/[pid]` deltas)
- **Pressure stall info** — cpu/memory/io PSI everywhere relevant
- **Sysdump** — raw procfs/sysfs file contents for diagnostics
- **History** — in-memory ring buffers, flattened series keys, SVG sparklines in the UI
- **Auth** — optional single-token auth (`?token=` query or `Authorization: Bearer`)
- **Zero-config deploy** — the UI (`ui/dist`) is embedded into the binary at compile time; the target machine needs nothing but `cargo build`

## Requirements

- Linux (any recent kernel; reads only `/proc`, `/sys`, `/etc`)
- Rust (stable toolchain)
- Optional for development: Node.js ≥ 20 + npm (only needed to rebuild the UI; `ui/dist` is committed)

## Build

```sh
git clone <repo-url> && cd RyzenAI-Dashboard
cargo build --release -C server        # embeds ui/dist automatically
```

Build order matters: `server/build.rs` scans `../ui/dist` at compile time and
embeds every asset (`crate::ui_assets::ASSETS`) into the binary. The compiled
`ui/dist` in this repo is committed, so **no Node toolchain is required** to
build or run the server. To change the UI:

```sh
cd ui && npm install && npm run build   # tsc --noEmit && vite build → ui/dist
cargo build --release                   # pick up the new dist
```

## Run

```sh
./target/release/ryzenai-dashboard              # 0.0.0.0:3010, embedded UI
```

By default the server listens on **all interfaces** (`0.0.0.0`, which includes
`127.0.0.1`): it is reachable via `http://localhost:3010` locally and via
`http://<lan-ip>:3010` from other machines. Use `--bind` to restrict to a
specific address (e.g. `--bind 127.0.0.1` for localhost-only).

### Background run (scripts/start_server.sh)

`scripts/start_server.sh` daemonizes the server so it does not occupy a
terminal; the binary is rebuilt automatically whenever sources or the embedded
UI are newer than the existing binary (so a plain `restart` after `git pull`
picks up the fresh code), output goes to `logs/server.log`,
the PID is stored in `logs/ryzenai-dashboard.pid`:

```sh
scripts/start_server.sh                 # start (build if needed) + free the terminal
scripts/start_server.sh status          # pid / port / last log lines
scripts/start_server.sh stop            # graceful SIGTERM, SIGKILL after grace
scripts/start_server.sh restart
scripts/start_server.sh log             # tail -f logs/server.log (Ctrl-C exits)
scripts/start_server.sh -- --port 8080 --token secret   # args after "--" go to the binary
```

### Options

```
--bind <addr>         listen address (default: 0.0.0.0 — all interfaces, includes 127.0.0.1)
--port <n>            TCP port (default: 3010)
--interval <ms>       collection interval (default: 2000)
--history <ticks>     ring buffer length per series (default: 900)
--sys-root <path>     sysfs/procfs root, dev use only (default: /)
--static <dir>        serve UI from a directory instead of the embedded UI
--token <t>           require a Bearer token on /api/* (query token or Authorization header)
--smartctl            enable optional smartctl probing (needs privileges)
```

With `--token T`, all `/api/*` routes require the token; the UI accepts it via
`?token=T` in the URL (first load), stores it in `localStorage`, and shows an
input form when the API responds `401`.

## API

| Endpoint | Description |
|---|---|
| `GET /api/snapshot` | one full `Snapshot` (system, temps, cpu, memory, disks, gpu, npu, network, processes, meta) |
| `GET /api/events` | SSE stream — one `data: <Snapshot JSON>` event per collection tick |
| `GET /api/history?sec=N` | `{sec, interval_ms, timestamps, series}` — flattened ring-buffer series (N clamped 10…3600) |
| `GET /api/sysdump` | `{ "<procfs path>": "<content>" }` for the diagnostic file list + hwmon chip names |

Everything else is served as static UI assets (embedded or `--static`), with a
SPA fallback to `index.html`.

## Tests

The server test-suite runs against a **synthetic fixture tree**, not the live
system:

```sh
scripts/gen_fixtures.sh          # deterministic /proc + /sys + /etc snapshot
cd server && cargo test          # 23 tests, all green
```

`scripts/gen_fixtures.sh` is the single source of truth for fixture values
(16 GB RAM, 8 CPUs, `nvme0n1`, hwmon0–3 with k10temp/nvme/amdgpu/acpitz, AMD
card1, `wlp2s0` + `tailscale0`, PSI entries, `devbox` hostname, …), mirrored
from the test assertions.

## Development notes

- **Types must stay in lockstep**: `ui/src/types.ts` mirrors the serde structs
  in `server/src/collect/mod.rs`. Change Rust — change TypeScript.
- **History series keys** are flattened snapshot paths (`server/src/history.rs`):
  arrays are indexed by their `id` field (`temps.hwmon3.temp1.temp_c`), else
  `name` (`disks.nvme0n1.*`, `network.wlp2s0.rx_kbps`), else numeric index
  (`cpu.per_core.0.usage_pct`); `null` (Rust `Option::None`) values are absent
  from series.
- `ui/README.md` documents the frontend build, sections, and API contract in detail.

## Architecture

```
ui/            React + Vite + TS frontend (committed dist, embedded by build.rs)
server/
  src/
    main.rs    arg parsing, app state, collector thread, HTTP serve
    args.rs    CLI options
    state.rs   shared App: latest snapshot + history rings + collector loop
    http.rs    /api/* routes, SSE, embedded/static UI serving with SPA fallback
    history.rs ring buffers + series flattening
    mapping.rs sensor role classification
    collect/   procfs/sysfs readers: stat, meminfo, mounts, diskstats, hwmon,
               thermal, net, procs, pressure, sysinfo, drmgpu, accel
  fixtures/    synthetic sys tree for tests (generated by scripts/gen_fixtures.sh)
scripts/       gen_fixtures.sh (deterministic test fixtures)
deploy/, docs/ (reserved)
```

## License

MIT — see `LICENSE`.
