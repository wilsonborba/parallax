#pragma once

#include <array>
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

struct MediaCodecInitParams {
    int width = 0;
    int height = 0;
    void* output_surface = nullptr;
    std::optional<std::uintptr_t> output_texture_handle;
    std::optional<int> codec_profile;
    std::optional<std::vector<std::uint8_t>> sps;
    std::optional<std::vector<std::uint8_t>> pps;
};

class MediaCodecDecoder {
public:
    ~MediaCodecDecoder();

    bool Initialize(const MediaCodecInitParams& params);
    void Shutdown();
    void SubmitAnnexBFrame(const std::vector<std::uint8_t>& packet, std::int64_t presentation_time_us = 0,
                           bool end_of_stream = false);
    bool DrainDecodedFrames();
    bool AcquireFrame(DecodedFrame& out_frame);

private:
    struct PendingPacket {
        std::vector<std::uint8_t> data;
        std::int64_t presentation_time_us = 0;
        bool end_of_stream = false;
    };

    bool GenerateTestPatternFrame();
    void CacheParameterSets(const std::vector<std::uint8_t>& packet);
    void FeedInputBuffers();
    bool DrainOutputBuffers();

    std::optional<DecodedFrame> latest_frame_;
    std::vector<PendingPacket> pending_packets_;
    std::uint64_t frame_counter_ = 0;
    bool use_test_pattern_ = true;
    int width_ = 0;
    int height_ = 0;
    std::optional<std::uintptr_t> output_texture_handle_;
    std::optional<int> codec_profile_;
    std::optional<std::vector<std::uint8_t>> sps_;
    std::optional<std::vector<std::uint8_t>> pps_;
#if defined(__ANDROID__)
    struct AMediaCodec* codec_ = nullptr;
    struct ANativeWindow* output_surface_ = nullptr;
    bool use_surface_output_ = false;
    int32_t stride_ = 0;
    int32_t slice_height_ = 0;
#endif
};
