#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

APP_HOME="${XDG_DATA_HOME:-$HOME/.local/share}/parallax"
BIN_DIR="$APP_HOME/bin"
LOCAL_BIN="${XDG_BIN_HOME:-$HOME/.local/bin}"
DESKTOP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
ICON_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor/scalable/apps"

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
    fail "Este instalador foi feito para Debian/Ubuntu (apt-get não encontrado)."
  fi

  local sudo_cmd=""
  if [[ "${EUID:-$(id -u)}" -ne 0 ]]; then
    if need_cmd sudo; then
      sudo_cmd="sudo"
    else
      fail "sudo não encontrado. Rode como root ou instale sudo."
    fi
  fi

  say "Instalando dependências do sistema..."
  $sudo_cmd apt-get update
  $sudo_cmd apt-get install -y "${pkgs[@]}"
}

ensure_rust() {
  if need_cmd cargo && need_cmd rustc; then
    return
  fi

  say "Rust não encontrado. Instalando rustup + toolchain estável..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
}

load_cargo_env() {
  if [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck disable=SC1090
    source "$HOME/.cargo/env"
  fi
  need_cmd cargo || fail "cargo não encontrado após instalação do Rust."
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
  say "Instalando arquivos do usuário..."
  mkdir -p "$BIN_DIR" "$LOCAL_BIN" "$DESKTOP_DIR" "$ICON_DIR"

  install -m 0755 "$REPO_ROOT/host/target/release/prlx-hostd" "$BIN_DIR/prlx-hostd"
  install -m 0755 "$REPO_ROOT/host/target/release/prlx-host-ui" "$BIN_DIR/prlx-host-ui"
  install -m 0755 "$REPO_ROOT/packaging/templates/parallax-wrapper.sh" "$LOCAL_BIN/parallax"
  install -m 0644 "$REPO_ROOT/packaging/templates/parallax.desktop" "$DESKTOP_DIR/parallax.desktop"
  install -m 0644 "$REPO_ROOT/packaging/assets/parallax.svg" "$ICON_DIR/parallax.svg"

  if need_cmd update-desktop-database; then
    update-desktop-database "$DESKTOP_DIR" >/dev/null 2>&1 || true
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
  fail "Distribuição não suportada por este instalador (somente Debian-like)."
}

print_success() {
  say "Instalação concluída."
  printf '\n'
  printf 'Comando instalado: %s\n' "$LOCAL_BIN/parallax"
  printf 'Desktop entry:    %s\n' "$DESKTOP_DIR/parallax.desktop"
  printf 'Icone:            %s\n' "$ICON_DIR/parallax.svg"
  printf '\n'
  printf 'Uso:\n'
  printf '  parallax            # abre a UI\n'
  printf '  parallax host --help\n'
  printf '  parallax doctor\n'
  printf '\n'

  case ":$PATH:" in
    *":$LOCAL_BIN:"*) ;;
    *)
      printf 'ATENCAO: %s nao esta no PATH atual.\n' "$LOCAL_BIN"
      printf 'Adicione no ~/.bashrc ou ~/.zshrc:\n'
      printf '  export PATH="$LOCAL_BIN:$PATH"\n'
      ;;
  esac
}

main() {
  check_debian
  install_apt_packages
  ensure_rust
  load_cargo_env
  build_binaries
  install_files
  print_success
}

main "$@"
