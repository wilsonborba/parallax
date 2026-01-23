#include "MediaCodecDecoder.h"

#include <iostream>

bool MediaCodecDecoder::Initialize() {
    std::cout << "MediaCodec decoder placeholder initialized.\n";
    return true;
}

void MediaCodecDecoder::SubmitPacket(const std::vector<std::uint8_t>& packet) {
    (void)packet;
    // Placeholder: feed packet to MediaCodec input buffers.
}

bool MediaCodecDecoder::DecodeNextFrame() {
    // Placeholder: fabricate a simple YUV frame to exercise the renderer.
    constexpr int kFrameWidth = 320;
    constexpr int kFrameHeight = 180;

    DecodedFrame frame;
    frame.width = kFrameWidth;
    frame.height = kFrameHeight;
    frame.frame_index = frame_counter_++;

    const std::size_t luma_size = static_cast<std::size_t>(kFrameWidth * kFrameHeight);
    const std::size_t chroma_width = kFrameWidth / 2;
    const std::size_t chroma_height = kFrameHeight / 2;
    const std::size_t chroma_size = static_cast<std::size_t>(chroma_width * chroma_height);

    frame.y_plane.resize(luma_size);
    frame.u_plane.resize(chroma_size);
    frame.v_plane.resize(chroma_size);

    for (int y = 0; y < kFrameHeight; ++y) {
        for (int x = 0; x < kFrameWidth; ++x) {
            frame.y_plane[static_cast<std::size_t>(y * kFrameWidth + x)] =
                static_cast<std::uint8_t>((x + frame.frame_index) % 255);
        }
    }

    const std::uint8_t u_value = static_cast<std::uint8_t>(96 + (frame.frame_index % 64));
    const std::uint8_t v_value = static_cast<std::uint8_t>(160 - (frame.frame_index % 64));
    std::fill(frame.u_plane.begin(), frame.u_plane.end(), u_value);
    std::fill(frame.v_plane.begin(), frame.v_plane.end(), v_value);

    latest_frame_ = std::move(frame);
    return true;
}

bool MediaCodecDecoder::AcquireFrame(DecodedFrame& out_frame) {
    if (!latest_frame_) {
        return false;
    }

    out_frame = std::move(*latest_frame_);
    latest_frame_.reset();
    return true;
}
