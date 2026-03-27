#!/usr/bin/env bash
set -euo pipefail

REPO_URL="${PARALLAX_REPO_URL:-https://github.com/asodya/parallax.git}"
REPO_BRANCH="${PARALLAX_REPO_BRANCH:-main}"
WORK_DIR="${PARALLAX_WORK_DIR:-$HOME/.local/share/parallax-installer}"
SRC_DIR="$WORK_DIR/src"
MODE="deb"
RUN_AFTER_INSTALL=0
SUDO=""

say() {
  printf '[parallax-install] %s\n' "$*"
}

fail() {
  printf '[parallax-install] ERROR: %s\n' "$*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1
}

usage() {
  cat <<'USAGE'
Parallax installer (Debian/Ubuntu only)

Usage:
  bash install.sh [--mode deb|cargo|source] [--run]

Modes:
  deb     Clone/update repo, build .deb and install with apt (default).
  cargo   Clone/update repo, cargo-install binaries, and install launcher/icon for current user.
  source  Alias of deb (kept for backward compatibility).

Examples:
  curl -fsSL https://parallax.asodya.com/assets/install.sh | bash
  curl -fsSL https://parallax.asodya.com/assets/install.sh | bash -s -- --mode cargo --run

Environment overrides:
  PARALLAX_REPO_URL
  PARALLAX_REPO_BRANCH
  PARALLAX_WORK_DIR
USAGE
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --mode)
        MODE="${2:-}"
        shift 2
        ;;
      --run)
        RUN_AFTER_INSTALL=1
        shift
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        fail "Invalid argument: $1"
        ;;
    esac
  done

  case "$MODE" in
    deb|cargo|source) ;;
    *) fail "Invalid mode: $MODE (use deb, cargo, or source)." ;;
  esac
}

check_debian_like() {
  if [[ -f /etc/os-release ]]; then
    # shellcheck disable=SC1091
    source /etc/os-release
    case "${ID:-}" in
      debian|ubuntu|linuxmint|pop|zorin) return ;;
    esac
    case "${ID_LIKE:-}" in
      *debian*) return ;;
    esac
  fi
  fail "This installer supports Debian/Ubuntu and Debian-like distributions only."
}

resolve_sudo() {
  if [[ "${EUID:-$(id -u)}" -eq 0 ]]; then
    SUDO=""
    return
  fi
  if need_cmd sudo; then
    SUDO="sudo"
    return
  fi
  fail "sudo not found. Run as root or install sudo."
}

install_base_apt_deps() {
  local pkgs=(ca-certificates curl git)
  say "Installing base dependencies (apt)..."
  $SUDO apt-get update
  $SUDO apt-get install -y "${pkgs[@]}"
}

sync_repo() {
  mkdir -p "$WORK_DIR"
  if [[ -d "$SRC_DIR/.git" ]]; then
    say "Updating repository in $SRC_DIR..."
    git -C "$SRC_DIR" fetch origin "$REPO_BRANCH" --depth 1
    git -C "$SRC_DIR" checkout -q "$REPO_BRANCH"
    git -C "$SRC_DIR" reset --hard "origin/$REPO_BRANCH"
  else
    say "Cloning repository into $SRC_DIR..."
    git clone --depth 1 --branch "$REPO_BRANCH" "$REPO_URL" "$SRC_DIR"
  fi
}

ensure_repo_layout() {
  local required_file
  case "$MODE" in
    deb|source) required_file="packaging/build-deb.sh" ;;
    cargo) required_file="host/Cargo.toml" ;;
    *) fail "Unsupported mode while validating repository layout: $MODE" ;;
  esac

  [[ -f "$SRC_DIR/$required_file" ]] || fail "Required file not found on branch '$REPO_BRANCH': $required_file"
}

stop_existing_hostd() {
  if pgrep -x prlx-hostd >/dev/null 2>&1; then
    say "Stopping existing prlx-hostd processes..."
    pkill -x prlx-hostd >/dev/null 2>&1 || true
    sleep 1
    pkill -9 -x prlx-hostd >/dev/null 2>&1 || true
  fi
}

cleanup_legacy_user_install() {
  local user_bin user_share
  user_bin="${XDG_BIN_HOME:-$HOME/.local/bin}"
  user_share="${XDG_DATA_HOME:-$HOME/.local/share}"

  say "Cleaning old user-level Parallax artifacts to avoid version conflicts..."
  rm -f "$user_bin/parallax"
  rm -f "$user_share/applications/parallax.desktop"
  rm -f "$user_share/icons/hicolor/scalable/apps/parallax.svg"
  rm -rf "$user_share/parallax"
}

install_via_deb() {
  local arch deb_path
  arch="$(dpkg --print-architecture)"

  stop_existing_hostd
  cleanup_legacy_user_install

  say "Building .deb package..."
  bash "$SRC_DIR/packaging/build-deb.sh"

  deb_path="$(ls -t "$SRC_DIR"/dist/parallax-host_*_"$arch".deb 2>/dev/null | head -n 1 || true)"
  [[ -n "$deb_path" ]] || fail ".deb package not found in $SRC_DIR/dist"

  say "Installing package: $deb_path"
  $SUDO apt install -y "$deb_path"
}

ensure_rust_for_cargo_mode() {
  if need_cmd cargo && need_cmd rustc; then
    return
  fi
  say "Rust not found. Installing rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
}

load_cargo_env() {
  if [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck disable=SC1090
    source "$HOME/.cargo/env"
  fi
  need_cmd cargo || fail "cargo not found after Rust installation."
}

install_via_cargo() {
  local app_home local_bin desktop_dir icon_dir
  app_home="${XDG_DATA_HOME:-$HOME/.local/share}/parallax"
  local_bin="${XDG_BIN_HOME:-$HOME/.local/bin}"
  desktop_dir="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
  icon_dir="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor/scalable/apps"

  stop_existing_hostd

  say "Installing build dependencies (apt) for cargo mode..."
  $SUDO apt-get install -y \
    build-essential pkg-config clang libclang-dev \
    libx11-dev libxext-dev libxfixes-dev libxrender-dev \
    libavcodec-dev libavformat-dev libavutil-dev libswscale-dev \
    desktop-file-utils

  ensure_rust_for_cargo_mode
  load_cargo_env

  say "Building and installing binaries with cargo..."
  cargo install --path "$SRC_DIR/host" --force --root "$app_home/cargo"

  say "Installing launcher and desktop entry for current user..."
  mkdir -p "$app_home/bin" "$local_bin" "$desktop_dir" "$icon_dir"
  install -m 0755 "$app_home/cargo/bin/prlx-hostd" "$app_home/bin/prlx-hostd"
  install -m 0755 "$app_home/cargo/bin/prlx-host-ui" "$app_home/bin/prlx-host-ui"
  install -m 0755 "$SRC_DIR/packaging/templates/parallax-wrapper.sh" "$local_bin/parallax"
  install -m 0644 "$SRC_DIR/packaging/templates/parallax.desktop" "$desktop_dir/parallax.desktop"
  install -m 0644 "$SRC_DIR/packaging/assets/parallax.svg" "$icon_dir/parallax.svg"

  if need_cmd update-desktop-database; then
    update-desktop-database "$desktop_dir" >/dev/null 2>&1 || true
  fi
}

print_result() {
  local cmd_hint
  cmd_hint="parallax"
  say "Installation completed in mode: $MODE"
  printf '\n'
  printf 'Next commands:\n'
  printf '  %s\n' "$cmd_hint"
  printf '  %s host --help\n' "$cmd_hint"
  printf '  %s doctor\n' "$cmd_hint"
  printf '\n'
}

maybe_run_ui() {
  if [[ "$RUN_AFTER_INSTALL" -eq 1 ]]; then
    say "Launching Parallax UI..."
    exec parallax
  fi
}

main() {
  parse_args "$@"
  check_debian_like
  resolve_sudo
  install_base_apt_deps
  sync_repo
  ensure_repo_layout

  if [[ "$MODE" == "source" ]]; then
    say "Mode 'source' is now an alias of 'deb' to keep installs consistent."
    MODE="deb"
  fi

  case "$MODE" in
    deb) install_via_deb ;;
    cargo) install_via_cargo ;;
  esac

  print_result
  maybe_run_ui
}

main "$@"
