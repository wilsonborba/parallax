#!/usr/bin/env bash
set -euo pipefail

BIN_DIR="/usr/lib/parallax"

usage() {
  cat <<USAGE
Usage: parallax [command] [args]

Commands:
  ui                 Launch Parallax Host UI (default)
  host [args]        Run prlx-hostd directly with optional args
  doctor             Show installation paths and binary status
  help               Show this help
USAGE
}

cmd="${1:-ui}"
case "$cmd" in
  ui)
    exec "$BIN_DIR/prlx-host-ui"
    ;;
  host)
    shift || true
    exec "$BIN_DIR/prlx-hostd" "$@"
    ;;
  doctor)
    echo "BIN_DIR=$BIN_DIR"
    if [[ -x "$BIN_DIR/prlx-host-ui" ]]; then
      echo "prlx-host-ui: OK"
    else
      echo "prlx-host-ui: MISSING"
    fi
    if [[ -x "$BIN_DIR/prlx-hostd" ]]; then
      echo "prlx-hostd: OK"
    else
      echo "prlx-hostd: MISSING"
    fi
    ;;
  help|-h|--help)
    usage
    ;;
  *)
    echo "Unknown command: $cmd" >&2
    usage
    exit 1
    ;;
esac
