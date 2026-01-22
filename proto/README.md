# Parallax UDP packet format

This directory defines the **wire format** used for UDP-based streaming between the
host and client. The goal is to keep the packet framing minimal while still providing
enough metadata for reassembly, loss detection, and codec configuration updates.

## Overview

Each UDP datagram begins with a fixed-size header followed by an opaque payload. The
payload contains the encoded media bytes (or control/config bytes) and is treated as
binary data.

```
0                   1                   2                   3
0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-------------------------------+---------------+-------------+
|            Magic              | Version | Hdr |   Flags     |
+-------------------------------+---------------+-------------+
|                          Stream ID                            |
+---------------------------------------------------------------+
|                           Frame ID                            |
+-------------------------------+---------------+---------------+
|   Packet ID   | Packet Count  | Payload Type  |   Reserved    |
+-------------------------------+---------------+---------------+
|          Payload Length       |      Payload Bytes ...        |
+-------------------------------+--------------------------------+
```

All multi-byte fields are **big-endian** (network byte order). The header is fixed at
24 bytes.

## Header fields

| Field | Size | Description |
| --- | --- | --- |
| Magic | 4 bytes | ASCII `PRLX` (`0x50 0x52 0x4c 0x58`). Used to validate packet framing. |
| Version | 1 byte | Protocol version. Start at `1`. |
| Header Length | 1 byte | Header size in bytes. Currently `24`. |
| Flags | 2 bytes | Bitfield describing payload semantics. |
| Stream ID | 4 bytes | Identifies a logical stream (e.g., video vs. audio). |
| Frame ID | 4 bytes | Monotonic frame counter for reassembly and timing. |
| Packet ID | 2 bytes | Index of this packet within the frame (0-based). |
| Packet Count | 2 bytes | Total number of packets in the frame. |
| Payload Type | 1 byte | Media or control payload classification. |
| Reserved | 1 byte | Must be zero; reserved for future use. |
| Payload Length | 2 bytes | Length of payload bytes following the header. |

## Flags

| Flag | Bit | Meaning |
| --- | --- | --- |
| `kFlagKeyFrame` | 0 | Payload includes a keyframe. |
| `kFlagConfig` | 1 | Payload includes codec configuration data (SPS/PPS/etc). |
| `kFlagEndOfFrame` | 2 | This packet is the final packet of a frame. |
| `kFlagDiscontinuity` | 3 | Indicates a reset in sequence or a dropped frame. |

## Payload types

| Type | Value | Description |
| --- | --- | --- |
| `kPayloadVideo` | `0x01` | Encoded video data. |
| `kPayloadAudio` | `0x02` | Encoded audio data. |
| `kPayloadControl` | `0x03` | Control or session metadata. |

## MTU guidance

We reserve space for the 24-byte header and target a safe UDP payload size that avoids
IP fragmentation on typical networks.

- **MTU (bytes):** 1200
- **Max payload size:** `MTU - header` = 1176 bytes

If the payload exceeds the maximum size, split it across multiple packets and populate
`Packet ID` / `Packet Count` accordingly.

## Versioning

Increment the version when header fields or semantics change. For forward compatibility,
receivers should ignore unknown flag bits and payload types, and should drop packets with
an unexpected `Magic` value.
