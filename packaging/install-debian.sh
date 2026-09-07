#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

APP_HOME="${XDG_DATA_HOME:-$HOME/.local/share}/parallax"
BIN_DIR="$APP_HOME/bin"
LOCAL_BIN="${XDG_BIN_HOME:-$HOME/.local/bin}"
DESKTOP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
ICON_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor/scalable/apps"
SYSTEMD_USER_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
SKIP_DEPS=0

need_cmd() {
  command -v "$1" >/dev/null 2>&1
}

say() {
  printf '[parallax-install] %s\n' "$*"
}

fail() {
  printf '[parallax-install] ERROR: %s\n' "$*" >&2
  exit 1
}

install_apt_packages() {
  local pkgs=(
    build-essential
    pkg-config
    curl
    ca-certificates
    clang
    libclang-dev
    libx11-dev
    libxext-dev
    libxfixes-dev
    libxrender-dev
    libavcodec-dev
    libavformat-dev
    libavutil-dev
    libswscale-dev
  )

  if ! need_cmd apt-get; then
    fail "This installer targets Debian/Ubuntu (apt-get not found)."
  fi

  local sudo_cmd=""
  if [[ "${EUID:-$(id -u)}" -ne 0 ]]; then
    if need_cmd sudo; then
      sudo_cmd="sudo"
    else
      fail "sudo not found. Run as root or install sudo."
    fi
  fi

  say "Installing system dependencies..."
  $sudo_cmd apt-get update
  $sudo_cmd apt-get install -y "${pkgs[@]}"
}

ensure_rust() {
  if need_cmd cargo && need_cmd rustc; then
    return
  fi

  say "Rust not found. Installing rustup + stable toolchain..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
}

load_cargo_env() {
  if [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck disable=SC1090
    source "$HOME/.cargo/env"
  fi
  need_cmd cargo || fail "cargo not found after Rust installation."
}

build_binaries() {
  say "Compilando prlx-hostd e prlx-host-ui (release)..."
  cargo build \
    --manifest-path "$REPO_ROOT/host/Cargo.toml" \
    --release \
    --bin prlx-hostd \
    --bin prlx-host-ui
}

install_files() {
  say "Installing user files..."
  mkdir -p "$BIN_DIR" "$LOCAL_BIN" "$DESKTOP_DIR" "$ICON_DIR" "$SYSTEMD_USER_DIR"

  install -m 0755 "$REPO_ROOT/host/target/release/prlx-hostd" "$BIN_DIR/prlx-hostd"
  install -m 0755 "$REPO_ROOT/host/target/release/prlx-host-ui" "$BIN_DIR/prlx-host-ui"
  install -m 0755 "$REPO_ROOT/packaging/templates/parallax-wrapper.sh" "$LOCAL_BIN/parallax"
  install -m 0644 "$REPO_ROOT/packaging/prlx-hostd.service" "$SYSTEMD_USER_DIR/prlx-hostd.service"
  install -m 0644 "$REPO_ROOT/packaging/templates/parallax.desktop" "$DESKTOP_DIR/parallax.desktop"
  install -m 0644 "$REPO_ROOT/packaging/assets/parallax.svg" "$ICON_DIR/parallax.svg"

  if need_cmd update-desktop-database; then
    update-desktop-database "$DESKTOP_DIR" >/dev/null 2>&1 || true
  fi

  if need_cmd systemctl; then
    systemctl --user daemon-reload >/dev/null 2>&1 || true
  fi
}

check_debian() {
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
  fail "Unsupported distribution for this installer (Debian-like only)."
}

print_success() {
  say "Installation complete."
  printf '\n'
  printf 'Installed command: %s\n' "$LOCAL_BIN/parallax"
  printf 'Desktop entry:    %s\n' "$DESKTOP_DIR/parallax.desktop"
  printf 'Icon:             %s\n' "$ICON_DIR/parallax.svg"
  printf 'User service:     %s\n' "$SYSTEMD_USER_DIR/prlx-hostd.service"
  printf '\n'
  printf 'Usage:\n'
  printf '  parallax            # opens the UI\n'
  printf '  parallax host --help\n'
  printf '  parallax doctor\n'
  printf '\n'
  printf 'Optional user service:\n'
  printf '  systemctl --user enable --now prlx-hostd.service\n'
  printf '  systemctl --user status prlx-hostd.service\n'
  printf '\n'

  case ":$PATH:" in
    *":$LOCAL_BIN:"*) ;;
    *)
      printf 'WARNING: %s is not in the current PATH.\n' "$LOCAL_BIN"
      printf 'Add this to ~/.bashrc or ~/.zshrc:\n'
      printf '  export PATH="%s:$PATH"\n' "$LOCAL_BIN"
      ;;
  esac
}

main() {
  for arg in "$@"; do
    case "$arg" in
      --skip-deps) SKIP_DEPS=1 ;;
      *)
        fail "Invalid argument: $arg. Use --skip-deps if dependencies are already installed."
        ;;
    esac
  done

  check_debian
  if [[ "$SKIP_DEPS" -eq 0 ]]; then
    install_apt_packages
  else
    say "Skipping dependency installation (--skip-deps)."
  fi
  ensure_rust
  load_cargo_env
  build_binaries
  install_files
  print_success
}

main "$@"
