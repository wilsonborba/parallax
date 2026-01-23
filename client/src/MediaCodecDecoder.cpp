#include "MediaCodecDecoder.h"

#include <algorithm>
#include <cstring>
#include <iostream>

#if defined(__ANDROID__)
#include <android/native_window.h>
#include <media/NdkMediaCodec.h>
#include <media/NdkMediaFormat.h>
#endif

namespace {
constexpr const char* kMimeTypeH264 = "video/avc";
} // namespace

MediaCodecDecoder::~MediaCodecDecoder() {
    Shutdown();
}

bool MediaCodecDecoder::Initialize(int width, int height, ANativeWindow* output_surface) {
    width_ = width;
    height_ = height;
    output_surface_ = output_surface;
    use_surface_output_ = output_surface_ != nullptr;
    stride_ = width_;
    slice_height_ = height_;

#if defined(__ANDROID__)
    if (width_ <= 0 || height_ <= 0) {
        std::cerr << "MediaCodec decoder requires a valid frame size.\n";
        return false;
    }

    codec_ = AMediaCodec_createDecoderByType(kMimeTypeH264);
    if (!codec_) {
        std::cerr << "Failed to create MediaCodec decoder for H.264.\n";
        return false;
    }

    AMediaFormat* format = AMediaFormat_new();
    AMediaFormat_setString(format, AMEDIAFORMAT_KEY_MIME, kMimeTypeH264);
    AMediaFormat_setInt32(format, AMEDIAFORMAT_KEY_WIDTH, width_);
    AMediaFormat_setInt32(format, AMEDIAFORMAT_KEY_HEIGHT, height_);
    if (!use_surface_output_) {
        AMediaFormat_setInt32(format, AMEDIAFORMAT_KEY_COLOR_FORMAT, COLOR_FormatYUV420Flexible);
    }

    media_status_t status = AMediaCodec_configure(codec_, format, output_surface_, nullptr, 0);
    AMediaFormat_delete(format);
    if (status != AMEDIA_OK) {
        std::cerr << "Failed to configure MediaCodec decoder.\n";
        AMediaCodec_delete(codec_);
        codec_ = nullptr;
        return false;
    }

    status = AMediaCodec_start(codec_);
    if (status != AMEDIA_OK) {
        std::cerr << "Failed to start MediaCodec decoder.\n";
        AMediaCodec_delete(codec_);
        codec_ = nullptr;
        return false;
    }

    std::cout << "MediaCodec decoder initialized for H.264 (" << width_ << "x" << height_ << ").\n";
    return true;
#else
    std::cout << "MediaCodec decoder placeholder initialized.\n";
    return true;
#endif
}

void MediaCodecDecoder::SubmitPacket(const std::vector<std::uint8_t>& packet, int64_t presentation_time_us,
                                     bool end_of_stream) {
    if (packet.empty() && !end_of_stream) {
        return;
    }

    pending_packets_.push_back(PendingPacket{packet, presentation_time_us, end_of_stream});
    FeedInputBuffers();
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
