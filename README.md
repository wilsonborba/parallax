# parallax
Can we reliably capture a Linux desktop, encode it, transmit it, decode it, and render it in VR with acceptable latency and stability?

## Repository layout
- `host/`: Rust-based streaming backend.
- `proto/`: Packet framing definitions shared with client implementations.

Legacy C++ client code has been removed from this repository as the project shifts to an
Android-based client.

## Host daemon (systemd user unit)
The `prlx-hostd` binary runs the TCP control loop and manages UDP streaming. A sample systemd
user unit is available at `packaging/prlx-hostd.service`. Customize the arguments as needed
for your display and addresses.

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
