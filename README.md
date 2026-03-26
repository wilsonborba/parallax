# Parallax

Parallax is an experimental remote-rendering pipeline for streaming a Linux desktop into a
client device (with a focus on VR/AR use cases). It captures an X11 display, encodes it as
H.264, transmits it over UDP, and coordinates sessions over a lightweight TCP control
channel.

## Why Parallax

- **Low-latency streaming:** UDP transport with a minimal framing protocol for fast reassembly.
- **Secure pairing:** A TCP control channel with a pairing token workflow.
- **Cross-device clients:** Current client work targets Android, with QR-based pairing.

## Architecture overview

```
+------------------+        TCP control        +-------------------+
| Linux host (Rust)| <-----------------------> | Android client     |
| prlx-hostd       |                           | Parallax Receiver  |
| - X11 capture    |        UDP video          | - H.264 decoder    |
| - H.264 encode   | ========================> | - Stream UI        |
+------------------+                           +-------------------+
```

- **Host daemon (`prlx-hostd`)**: Captures the X11 display, encodes frames, and serves the
  control channel that brokers client sessions.
- **Host UI (`prlx-host-ui`)**: A desktop UI to visualize status and show the pairing QR code.
- **Android client**: Scans the QR code, connects to the control channel, and receives the
  H.264 stream.

## Repository layout

- `host/`: Rust-based host daemon and desktop UI.
- `client/`: Android client (Kotlin + Jetpack Compose).
- `proto/`: UDP packet format and wire protocol documentation.
- `packaging/`: Systemd service unit(s) for the host daemon.

## Debian install options (3 alternatives)

### 1) One command (`curl | bash`) for Debian/Ubuntu

From any directory:

```bash
curl -fsSL https://raw.githubusercontent.com/asodya/parallax/main/install.sh | bash
```

When you publish the website on Cloudflare Pages with Flutter's default build output, the installer is available at:

```bash
curl -fsSL https://parallax.asodya.com/assets/assets/install.sh | bash
```

Modes:

```bash
# default: source install (user-local launcher + desktop icon)
curl -fsSL https://raw.githubusercontent.com/asodya/parallax/main/install.sh | bash

# build and install a .deb package
curl -fsSL https://raw.githubusercontent.com/asodya/parallax/main/install.sh | bash -s -- --mode deb

# cargo-based install flow
curl -fsSL https://raw.githubusercontent.com/asodya/parallax/main/install.sh | bash -s -- --mode cargo
```

### 2) Cargo-focused flow (from repo)

```bash
cargo install --path host
./packaging/install-debian.sh
```

### 3) Clone repo and install

```bash
git clone https://github.com/asodya/parallax.git
cd parallax
./packaging/install-debian.sh
```

## Build a `.deb` package (Debian)

From the repo root:

```bash
./packaging/build-deb.sh
```

This creates a package in `dist/`, for example:

```bash
dist/parallax-host_0.1.0_amd64.deb
```

Install it with:

```bash
sudo apt install ./dist/parallax-host_0.1.0_amd64.deb
```

If dependencies are already installed, you can skip `apt` in the build step:

```bash
./packaging/build-deb.sh --skip-deps
```

To run package quality checks with `lintian` after building:

```bash
./packaging/build-deb.sh --lint
```

After install:

- `parallax` is available in `/usr/bin/parallax`.
- The launcher appears in the desktop app menu as **Parallax Host**.
- UI start/stop controls manage the host daemon lifecycle.

## Quick start

### 1) Build the host daemon

From the repo root:

```bash
cd host
cargo build --release
```

Run the daemon (defaults shown in `--help`):

```bash
./target/release/prlx-hostd \
  --display :0 \
  --bind 0.0.0.0:5000 \
  --target 127.0.0.1:5000 \
  --control-bind 0.0.0.0:0
```

The control bind defaults to `0.0.0.0:0`, which lets the OS select an available port. The
selected port is printed by the host UI and included in the pairing QR code.

### 2) Launch the host UI (optional but recommended)

The UI will attempt to connect to the running daemon and show a pairing QR code.

```bash
cd host
cargo run --release --bin prlx-host-ui
```

### 3) Build and run the Android client

Open `client/` in Android Studio **or** build from the command line:

```bash
cd client
./gradlew :app:assembleDebug
```

Install the APK on a device running Android 10+ (minSdk 29). Open the app and scan the QR
code from the host UI to connect.

## Host configuration

`prlx-hostd` accepts CLI flags (run `--help` for the full list):

- `--display <DISPLAY>`: X11 display to capture (default `:0`).
- `--bind <ADDR>`: UDP bind address for the host (default `0.0.0.0:5000`).
- `--target <ADDR>`: UDP target address for the client (default `127.0.0.1:5000`).
- `--control-bind <ADDR>`: TCP control bind address (default `0.0.0.0:0`).
- `--pairing-token <KEY>`: Token required for control sessions (default `auto`).
- `--prefer-vaapi` / `--software`: Choose hardware vs. software H.264 encoding.

## Systemd user unit

A sample systemd user unit is available at `packaging/prlx-hostd.service`. Customize the
arguments as needed for your display and addresses.

Enable the user unit:

```bash
mkdir -p ~/.config/systemd/user
cp packaging/prlx-hostd.service ~/.config/systemd/user/prlx-hostd.service
systemctl --user daemon-reload
systemctl --user enable --now prlx-hostd.service
```

Check status/logs:

```bash
systemctl --user status prlx-hostd.service
journalctl --user -u prlx-hostd.service
```

## Protocol documentation

- UDP packet framing: [`proto/README.md`](proto/README.md)
- TCP control channel: [`proto/CONTROL.md`](proto/CONTROL.md)

## Development notes

- The host currently targets Linux with X11 capture and FFmpeg-based H.264 encoding.
- The Android client uses Jetpack Compose and includes QR-based pairing for ease of setup.

## License

This project is licensed under the terms of the MIT license. See [`LICENSE`](LICENSE).
