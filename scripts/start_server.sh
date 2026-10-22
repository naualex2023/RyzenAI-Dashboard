#!/usr/bin/env bash
# start_server.sh — daemonize the RyzenAI Dashboard server so it frees the terminal.
#
# Usage: scripts/start_server.sh [command] [-- server-args...]
#   start      (default) build if needed, launch in background, write PID file
#   stop       graceful SIGTERM, then SIGKILL after a grace period
#   restart    stop + start
#   status     print PID / port / log tail
#   log        tail -f the server log (Ctrl-C to exit; server keeps running)
#
# Everything after "--" is passed verbatim to the binary, e.g.:
#   scripts/start_server.sh start -- --port 8080 --token secret
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVER_DIR="$ROOT/server"
BIN="$SERVER_DIR/target/release/ryzenai-dashboard"
LOG_DIR="$ROOT/logs"
LOG_FILE="$LOG_DIR/server.log"
PID_FILE="$LOG_DIR/ryzenai-dashboard.pid"
PORT="${RAD_PORT:-3010}"

mkdir -p "$LOG_DIR"

# ---- helpers ----------------------------------------------------------------

alive() {                       # true if a pidfile holds a live process
  [[ -f "$PID_FILE" ]] && kill -0 "$(cat "$PID_FILE")" 2>/dev/null
}

pid() { cat "$PID_FILE" 2>/dev/null || true; }

build() {
  if [[ ! -f "$BIN" ]]; then
    echo "[build] compiling release binary…"
    (cd "$SERVER_DIR" && cargo build --release)
  fi
}

start() {
  if alive; then
    echo "[start] already running (pid $(pid), http://localhost:$PORT)"
    return 0
  fi
  build
  echo "[start] launching $(basename "$BIN") → $LOG_FILE"
  nohup "$BIN" ${PASS[@]:+"${PASS[@]}"} >>"$LOG_FILE" 2>&1 &
  local p=$!
  echo "$p" >"$PID_FILE"
  sleep 0.3
  if ! kill -0 "$p" 2>/dev/null; then
    echo "[start] process died immediately — log tail:"
    tail -n 20 "$LOG_FILE"
    return 1
  fi
  echo "[start] running (pid $p) — http://localhost:$PORT (also on 0.0.0.0: LAN interface)"
}

stop() {
  if ! alive; then
    echo "[stop] not running"; rm -f "$PID_FILE"; return 0
  fi
  local p; p=$(pid)
  echo "[stop] stopping pid $p…"
  kill "$p" 2>/dev/null || true
  for i in {1..30}; do
    kill -0 "$p" 2>/dev/null || break
    sleep 0.2
  done
  if kill -0 "$p" 2>/dev/null; then
    echo "[stop] grace period expired — SIGKILL"
    kill -9 "$p" 2>/dev/null || true
  fi
  rm -f "$PID_FILE"
  echo "[stop] stopped"
}

status() {
  if alive; then
    echo "[status] running (pid $(pid)) on port $PORT"
    echo "[status] last log lines:"
    tail -n 5 "$LOG_FILE" 2>/dev/null || true
  else
    echo "[status] not running"
  fi
}

# ---- arg handling -----------------------------------------------------------

CMD="start"
declare -a PASS=()              # args after "--" go verbatim to the binary
declare -a RAW=("$@")
for ((i=0; i<${#RAW[@]}; i++)); do
  case "${RAW[i]}" in
    start|stop|restart|status|log|tail) CMD="${RAW[i]}" ;;
    --) PASS=("${RAW[@]:$((i+1))}"); break ;;
    -p|--port) PORT="${RAW[i+1]}"; i++ ;;   # script-level display port
    *) ;;
  esac
done

# Prefer an explicit --port passed to the binary over the script-level one.
for ((i=0; i<${#PASS[@]}; i++)); do
  if [[ "${PASS[i]}" == "--port" ]]; then
    PORT="${PASS[i+1]}"
    break
  fi
done

case "$CMD" in
  start)  start ;;
  stop)   stop ;;
  restart)
    stop
    start
    ;;
  status) status ;;
  log|tail)
    touch "$LOG_FILE"
    echo "[log] tail -f $LOG_FILE (server keeps running)"
    exec tail -n 40 -f "$LOG_FILE"
    ;;
  *) echo "usage: $0 [start|stop|restart|status|log] [-p PORT] [-- server-flags…]" && exit 1 ;;
esac
