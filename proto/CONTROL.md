# Parallax Control Protocol (TCP)

This document describes the TCP control channel message types used between client and host daemon (`prlx-hostd`).

## Frame format

Each frame on the TCP stream uses this header:

- `version` (1 byte)
- `message_type` (1 byte)
- `payload_len` (2 bytes, big-endian)
- `payload` (`payload_len` bytes)

Current version: `1`

## Message types

### Handshake/Auth

- `0x01` `Hello`
- `0x02` `HelloAck`
- `0x03` `PairRequest`
- `0x04` `PairAccept`
- `0x05` `PairReject`
- `0x06` `AuthChallenge`
- `0x07` `AuthResponse`

### Stream lifecycle/config

- `0x10` `StartStream`
- `0x11` `StopStream`
- `0x12` `StreamStarted`
- `0x13` `StreamStopped`
- `0x14` `ListStreams`
- `0x15` `Streams`
- `0x16` `SetStreamConfig`
- `0x17` `StreamConfigAck`

### Keepalive

- `0x20` `Ping`
- `0x21` `Pong`

### Display management (client-driven)

- `0x30` `ListDisplays`
- `0x31` `Displays`
- `0x32` `AddVirtualDisplay`
- `0x33` `RemoveVirtualDisplay`
- `0x34` `DisplayOpAck`

### Generic

- `0x7f` `Error`

## Payload contracts (current)

### `StartStream` / `StopStream`

Payload optional:

- empty -> defaults to `stream_id=1`
- `stream_id=<id>`
- `<id>` (plain integer)

Current host supports `stream_id=1..3`.

### `ListStreams`

Payload: empty.

`Streams` response payload (UTF-8 text):

```text
protocol=2
streams:
<stream_id>,<display>,<bind_addr>,<target_addr>,<prefer_vaapi>,<running>,<width>,<height>,<fps>,<bitrate_kbps>
```

Notes:

- `fps` is emitted as decimal text (`NN.NN`).
- `bitrate_kbps` is integer kbps.
- For backward compatibility, older clients may ignore fields after `running`.

### `SetStreamConfig`

Payload is UTF-8 `key=value` lines.

Required:

```text
stream_id=1
```

Optional keys:

- `display`
- `bind_addr`
- `target_addr`
- `prefer_vaapi` (`true|false`)

Success response: `StreamConfigAck`.

### `ListDisplays`

Payload: empty.

`Displays` response payload (UTF-8 text):

```text
protocol=2
physical:
<id>,<name>,<primary>,<connected>,<width>,<height>,<x>,<y>
virtual:
<id>,<enabled>,<width>,<height>,<x>,<y>
```

### `AddVirtualDisplay`

Payload UTF-8 CSV:

```text
id,width,height,x,y
```

Success response: `DisplayOpAck`.

### `RemoveVirtualDisplay`

Payload UTF-8 text:

```text
id
```

Success response: `DisplayOpAck`.

## Authorization

After handshake starts, control operations that mutate state require the session to be paired/authenticated.
When not paired, host responds with `Error` payload: `pairing required`.

## Host Status Socket (Unix domain)

The host also exposes a local status socket (default):

- `~/.local/share/prlx/prlx.sock`

This is used by host-side tools/UI for quick observability and lightweight control.

### Commands

- `status`
  - Returns daemon state line (`idle|waiting|connected|streaming`) plus optional pin/qr fields.
- `start`
  - Starts stream `1`.
- `stop`
  - Stops stream `1`.
- `start <id>`
  - Starts stream `<id>` (`1..3`).
- `stop <id>`
  - Stops stream `<id>` (`1..3`).
- `streams`
  - Returns the same `ListStreams` payload format shown above.

### Example

```bash
SOCK="$HOME/.local/share/prlx/prlx.sock"
printf 'streams\n' | socat - UNIX-CONNECT:"$SOCK"
printf 'start 2\n' | socat - UNIX-CONNECT:"$SOCK"
printf 'stop 2\n'  | socat - UNIX-CONNECT:"$SOCK"
```
