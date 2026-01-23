#pragma once

#include <cstdint>
#include <optional>
#include <vector>

struct DecodedFrame {
    int width = 0;
    int height = 0;
    std::vector<std::uint8_t> y_plane;
    std::vector<std::uint8_t> u_plane;
    std::vector<std::uint8_t> v_plane;
    std::uint64_t frame_index = 0;
};

class MediaCodecDecoder {
public:
    bool Initialize();
    void SubmitPacket(const std::vector<std::uint8_t>& packet);
    bool DecodeNextFrame();
    bool AcquireFrame(DecodedFrame& out_frame);

private:
    std::optional<DecodedFrame> latest_frame_;
    std::uint64_t frame_counter_ = 0;
};
