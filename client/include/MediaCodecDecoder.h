#pragma once

#include <cstdint>
#include <vector>

class MediaCodecDecoder {
public:
    bool Initialize();
    void SubmitPacket(const std::vector<std::uint8_t>& packet);
    bool DecodeNextFrame();
};
