#pragma once

#include <array>
#include <cstdint>
#include <deque>
#include <vector>

struct ANativeWindow;

class MediaCodecDecoder {
public:
    struct DecodedPlane {
        const std::uint8_t* data = nullptr;
        int stride = 0;
        int size = 0;
    };

    struct DecodedFrame {
        int width = 0;
        int height = 0;
        int64_t presentation_time_us = 0;
        int buffer_index = -1;
        bool texture_output = false;
        int plane_count = 0;
        std::array<DecodedPlane, 3> planes{};
    };

    ~MediaCodecDecoder();

    bool Initialize(int width, int height, ANativeWindow* output_surface = nullptr);
    void SubmitPacket(const std::vector<std::uint8_t>& packet, int64_t presentation_time_us = 0,
                      bool end_of_stream = false);
    bool DecodeNextFrame(DecodedFrame* frame);
    void ReleaseFrame(const DecodedFrame& frame);
    void Shutdown();

private:
    struct PendingPacket {
        std::vector<std::uint8_t> data;
        int64_t presentation_time_us = 0;
        bool end_of_stream = false;
    };

    void FeedInputBuffers();

    int width_ = 0;
    int height_ = 0;
    int stride_ = 0;
    int slice_height_ = 0;
    int color_format_ = 0;
    ANativeWindow* output_surface_ = nullptr;
    bool use_surface_output_ = false;
    std::deque<PendingPacket> pending_packets_;
#if defined(__ANDROID__)
    struct AMediaCodec* codec_ = nullptr;
#endif
};
