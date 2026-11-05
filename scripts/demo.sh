#!/usr/bin/env bash
# Demo: runs the server and a CPU monitor side by side in an mprocs TUI.
# Pick a pane on the left to read its logs. Quit with `q`.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SOCKET="${TMPDIR:-/tmp}/plain-metrics-demo.sock"
DATA_DIR="${TMPDIR:-/tmp}/plain-metrics-demo-data"
INTERVAL=5

# Used by the panes below: `demo.sh server` and `demo.sh cpu`.
case "${1:-}" in
  server)
    mkdir -p "$DATA_DIR"
    exec "$ROOT/target/debug/metrics" server --data-dir "$DATA_DIR" --socket "$SOCKET"
    ;;
  cpu)
    # Wait for the server to create its socket.
    until [[ -S "$SOCKET" ]]; do sleep 0.2; done
    while true; do
      # "CPU usage: 7.8% user, 5.5% sys, 86.6% idle" -> field 7 is the idle share.
      idle=$(top -l 1 -n 0 | rg 'CPU usage' | cut -d ' ' -f 7 | tr -d '%')
      usage=$(echo "100 - $idle" | bc)
      "$ROOT/target/debug/metrics" record --socket "$SOCKET" "cpu.usage=$usage" >/dev/null
      echo "sent cpu.usage=$usage%"
      sleep "$INTERVAL"
    done
    ;;
  data)
    # Follow every file in the data dir, including ones created later.
    mkdir -p "$DATA_DIR"
    trap 'kill 0' EXIT
    seen=$'\n'  # newline-separated list; macOS bash 3.2 has no associative arrays
    while true; do
      for f in "$DATA_DIR"/*; do
        [[ -f "$f" && "$seen" != *$'\n'"$f"$'\n'* ]] || continue
        seen+="$f"$'\n'
        # Prefix each line with the file name; -n +1 shows existing content too.
        tail -n +1 -F "$f" 2>/dev/null | sed -u "s|^|$(basename "$f"): |" &
      done
      sleep 1
    done
    ;;
  "")
    for tool in mprocs rg bc; do
      command -v "$tool" >/dev/null || { echo "missing: $tool" >&2; exit 1; }
    done
    cargo build --manifest-path "$ROOT/Cargo.toml"
    exec mprocs --names "server,cpu monitor,data files" "$0 server" "$0 cpu" "$0 data"
    ;;
  *)
    echo "usage: $0" >&2
    exit 2
    ;;
esac
