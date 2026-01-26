# parallax
Can we reliably capture a Linux desktop, encode it, transmit it, decode it, and render it in VR with acceptable latency and stability?

## Repository layout
- `host/`: Rust-based streaming backend.
- `proto/`: Packet framing definitions shared with client implementations.

Legacy C++ client code has been removed from this repository as the project shifts to an
Android-based client.
