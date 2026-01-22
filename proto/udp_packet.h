#pragma once

#include <cstdint>

namespace parallax::proto {

constexpr std::uint32_t kMagic = 0x50524c58; // "PRLX"
constexpr std::uint8_t kVersion = 1;
constexpr std::uint8_t kHeaderSizeBytes = 24;

constexpr std::uint16_t kFlagKeyFrame = 1 << 0;
constexpr std::uint16_t kFlagConfig = 1 << 1;
constexpr std::uint16_t kFlagEndOfFrame = 1 << 2;
constexpr std::uint16_t kFlagDiscontinuity = 1 << 3;

constexpr std::uint8_t kPayloadVideo = 0x01;
constexpr std::uint8_t kPayloadAudio = 0x02;
constexpr std::uint8_t kPayloadControl = 0x03;

constexpr std::uint16_t kMtuBytes = 1200;
constexpr std::uint16_t kMaxPayloadSize = kMtuBytes - kHeaderSizeBytes;

#pragma pack(push, 1)
struct UdpPacketHeader {
    std::uint32_t magic;
    std::uint8_t version;
    std::uint8_t header_length;
    std::uint16_t flags;
    std::uint32_t stream_id;
    std::uint32_t frame_id;
    std::uint16_t packet_id;
    std::uint16_t packet_count;
    std::uint8_t payload_type;
    std::uint8_t reserved;
    std::uint16_t payload_length;
};
#pragma pack(pop)

static_assert(sizeof(UdpPacketHeader) == kHeaderSizeBytes, "Header size mismatch");

} // namespace parallax::proto
