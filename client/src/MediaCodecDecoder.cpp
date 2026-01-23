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

bool MediaCodecDecoder::DecodeNextFrame(DecodedFrame* frame) {
    if (!frame) {
        return false;
    }

#if defined(__ANDROID__)
    if (!codec_) {
        return false;
    }

    FeedInputBuffers();

    AMediaCodecBufferInfo buffer_info{};
    ssize_t output_index = AMediaCodec_dequeueOutputBuffer(codec_, &buffer_info, 0);
    if (output_index == AMEDIACODEC_INFO_OUTPUT_FORMAT_CHANGED) {
        AMediaFormat* format = AMediaCodec_getOutputFormat(codec_);
        if (format) {
            AMediaFormat_getInt32(format, AMEDIAFORMAT_KEY_WIDTH, &width_);
            AMediaFormat_getInt32(format, AMEDIAFORMAT_KEY_HEIGHT, &height_);
            AMediaFormat_getInt32(format, AMEDIAFORMAT_KEY_STRIDE, &stride_);
            AMediaFormat_getInt32(format, AMEDIAFORMAT_KEY_SLICE_HEIGHT, &slice_height_);
            AMediaFormat_getInt32(format, AMEDIAFORMAT_KEY_COLOR_FORMAT, &color_format_);
            AMediaFormat_delete(format);
        }
        return false;
    }

    if (output_index < 0) {
        return false;
    }

    frame->width = width_;
    frame->height = height_;
    frame->presentation_time_us = buffer_info.presentationTimeUs;
    frame->buffer_index = static_cast<int>(output_index);
    frame->texture_output = use_surface_output_;

    if (use_surface_output_) {
        frame->plane_count = 0;
        return true;
    }

    size_t data_size = 0;
    std::uint8_t* data = AMediaCodec_getOutputBuffer(codec_, output_index, &data_size);
    if (!data || data_size == 0) {
        return false;
    }

    std::uint8_t* base = data + buffer_info.offset;
    int y_stride = stride_ > 0 ? stride_ : width_;
    int y_height = slice_height_ > 0 ? slice_height_ : height_;
    int uv_stride = y_stride / 2;
    int uv_height = y_height / 2;
    size_t y_size = static_cast<size_t>(y_stride) * static_cast<size_t>(y_height);
    size_t uv_size = static_cast<size_t>(uv_stride) * static_cast<size_t>(uv_height);

    size_t required_size = y_size + 2 * uv_size;
    if (buffer_info.size < static_cast<int32_t>(required_size)) {
        required_size = static_cast<size_t>(buffer_info.size);
    }

    frame->plane_count = 3;
    frame->planes[0] = {base, y_stride, static_cast<int>(std::min(y_size, required_size))};

    std::uint8_t* u_plane = base + y_size;
    std::uint8_t* v_plane = u_plane + uv_size;
    frame->planes[1] = {u_plane, uv_stride, static_cast<int>(std::min(uv_size, required_size - y_size))};
    frame->planes[2] = {v_plane, uv_stride,
                        static_cast<int>(std::min(uv_size, required_size - y_size - uv_size))};
    return true;
#else
    (void)frame;
    return false;
#endif
}

void MediaCodecDecoder::ReleaseFrame(const DecodedFrame& frame) {
#if defined(__ANDROID__)
    if (!codec_ || frame.buffer_index < 0) {
        return;
    }
    AMediaCodec_releaseOutputBuffer(codec_, frame.buffer_index, frame.texture_output);
#else
    (void)frame;
#endif
}

void MediaCodecDecoder::Shutdown() {
#if defined(__ANDROID__)
    if (codec_) {
        AMediaCodec_stop(codec_);
        AMediaCodec_delete(codec_);
        codec_ = nullptr;
    }
#endif
    pending_packets_.clear();
}

void MediaCodecDecoder::FeedInputBuffers() {
#if defined(__ANDROID__)
    if (!codec_) {
        return;
    }

    while (!pending_packets_.empty()) {
        ssize_t input_index = AMediaCodec_dequeueInputBuffer(codec_, 0);
        if (input_index < 0) {
            break;
        }

        PendingPacket packet = std::move(pending_packets_.front());
        pending_packets_.pop_front();

        size_t buffer_size = 0;
        std::uint8_t* buffer = AMediaCodec_getInputBuffer(codec_, input_index, &buffer_size);
        if (!buffer) {
            AMediaCodec_queueInputBuffer(codec_, input_index, 0, 0, packet.presentation_time_us, 0);
            continue;
        }

        size_t copy_size = std::min(buffer_size, packet.data.size());
        if (copy_size > 0) {
            std::memcpy(buffer, packet.data.data(), copy_size);
        }

        uint32_t flags = packet.end_of_stream ? AMEDIACODEC_BUFFER_FLAG_END_OF_STREAM : 0;
        AMediaCodec_queueInputBuffer(codec_, input_index, 0, copy_size, packet.presentation_time_us, flags);
    }
#else
    return;
#endif
}
