#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST_DIR="$REPO_ROOT/dist"
SKIP_DEPS=0
RUN_LINTIAN=0

need_cmd() {
  command -v "$1" >/dev/null 2>&1
}

say() {
  printf '[parallax-deb] %s\n' "$*"
}

fail() {
  printf '[parallax-deb] ERROR: %s\n' "$*" >&2
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
    dpkg-dev
    lintian
  )

  local sudo_cmd=""
  if [[ "${EUID:-$(id -u)}" -ne 0 ]]; then
    if need_cmd sudo; then
      sudo_cmd="sudo"
    else
      fail "sudo not found. Run as root or install sudo."
    fi
  fi

  say "Installing build dependencies (apt)..."
  $sudo_cmd apt-get update
  $sudo_cmd apt-get install -y "${pkgs[@]}"
}

ensure_rust() {
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
  fail "Unsupported distribution for this script (Debian-like only)."
}

read_version() {
  awk -F '"' '/^version = "/ {print $2; exit}' "$REPO_ROOT/host/Cargo.toml"
}

read_arch() {
  dpkg --print-architecture
}

build_binaries() {
  say "Compilando binários release..."
  cargo build \
    --manifest-path "$REPO_ROOT/host/Cargo.toml" \
    --release \
    --bin prlx-hostd \
    --bin prlx-host-ui
}

write_control() {
  local control_path="$1"
  local version="$2"
  local arch="$3"
  local maintainer_name maintainer_email

  maintainer_name="$(git -C "$REPO_ROOT" config user.name || true)"
  maintainer_email="$(git -C "$REPO_ROOT" config user.email || true)"
  if [[ -z "$maintainer_name" ]]; then
    maintainer_name="Parallax Maintainers"
  fi
  if [[ -z "$maintainer_email" ]]; then
    maintainer_email="maintainers@localhost"
  fi

  cat > "$control_path" <<CONTROL
Package: parallax-host
Version: $version
Section: net
Priority: optional
Architecture: $arch
Maintainer: $maintainer_name <$maintainer_email>
Depends: libc6, libx11-6, libxext6, libxfixes3, libxrender1, libavcodec61 | libavcodec60 | libavcodec59, libavformat61 | libavformat60 | libavformat59, libavutil59 | libavutil58 | libavutil57, libswscale8 | libswscale7 | libswscale6
Description: Parallax Linux host daemon and UI
 Experimental Linux host package for Parallax streaming.
 Provides:
  - /usr/bin/parallax launcher
  - prlx-host-ui desktop UI
  - prlx-hostd daemon binary
CONTROL
}

write_maintainer_scripts() {
  local postinst="$1"
  local postrm="$2"

  cat > "$postinst" <<'POSTINST'
#!/usr/bin/env bash
set -e

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database /usr/share/applications >/dev/null 2>&1 || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -q /usr/share/icons/hicolor >/dev/null 2>&1 || true
fi
POSTINST

  cat > "$postrm" <<'POSTRM'
#!/usr/bin/env bash
set -e

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database /usr/share/applications >/dev/null 2>&1 || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -q /usr/share/icons/hicolor >/dev/null 2>&1 || true
fi
POSTRM

  chmod 0755 "$postinst" "$postrm"
}

stage_package_tree() {
  local pkg_root="$1"

  mkdir -p "$pkg_root/DEBIAN"
  mkdir -p "$pkg_root/usr/bin"
  mkdir -p "$pkg_root/usr/lib/parallax"
  mkdir -p "$pkg_root/usr/share/applications"
  mkdir -p "$pkg_root/usr/share/icons/hicolor/scalable/apps"
  mkdir -p "$pkg_root/usr/share/doc/parallax-host"

  install -m 0755 "$REPO_ROOT/packaging/templates/parallax-wrapper-system.sh" "$pkg_root/usr/bin/parallax"
  install -m 0755 "$REPO_ROOT/host/target/release/prlx-hostd" "$pkg_root/usr/lib/parallax/prlx-hostd"
  install -m 0755 "$REPO_ROOT/host/target/release/prlx-host-ui" "$pkg_root/usr/lib/parallax/prlx-host-ui"
  install -m 0644 "$REPO_ROOT/packaging/templates/parallax.desktop" "$pkg_root/usr/share/applications/parallax.desktop"
  install -m 0644 "$REPO_ROOT/packaging/assets/parallax.svg" "$pkg_root/usr/share/icons/hicolor/scalable/apps/parallax.svg"
  install -m 0644 "$REPO_ROOT/README.md" "$pkg_root/usr/share/doc/parallax-host/README.md"
}

build_deb() {
  local version arch out_name tmpdir pkg_root
  version="$(read_version)"
  [[ -n "$version" ]] || fail "Could not read version from host/Cargo.toml"

  arch="$(read_arch)"
  [[ -n "$arch" ]] || fail "Could not detect architecture with dpkg"

  out_name="parallax-host_${version}_${arch}.deb"
  mkdir -p "$DIST_DIR"

  tmpdir="$(mktemp -d)"
  trap 'rm -rf "${tmpdir:-}"' EXIT
  pkg_root="$tmpdir/parallax-host"

  stage_package_tree "$pkg_root"
  write_control "$pkg_root/DEBIAN/control" "$version" "$arch"
  write_maintainer_scripts "$pkg_root/DEBIAN/postinst" "$pkg_root/DEBIAN/postrm"

  say "Building .deb package..."
  dpkg-deb --build --root-owner-group "$pkg_root" "$DIST_DIR/$out_name"

  say "Package created: $DIST_DIR/$out_name"
  printf '\nInstall with:\n'
  printf '  sudo apt install "%s/%s"\n' "$DIST_DIR" "$out_name"
  printf '\n'
  if [[ "$RUN_LINTIAN" -eq 1 ]]; then
    run_lintian "$DIST_DIR/$out_name"
  fi
}

run_lintian() {
  local deb_path="$1"
  if ! need_cmd lintian; then
    fail "lintian not found. Install it or run without --lint."
  fi
  say "Running lintian on the package..."
  lintian "$deb_path"
}

main() {
  for arg in "$@"; do
    case "$arg" in
      --skip-deps) SKIP_DEPS=1 ;;
      --lint) RUN_LINTIAN=1 ;;
      *)
        fail "Invalid argument: $arg. Use --skip-deps and/or --lint."
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
  build_deb
}

main "$@"
