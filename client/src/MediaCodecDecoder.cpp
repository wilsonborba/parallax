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
constexpr std::uint8_t kNalUnitTypeSps = 7;
constexpr std::uint8_t kNalUnitTypePps = 8;
} // namespace

MediaCodecDecoder::~MediaCodecDecoder() {
    Shutdown();
}

bool MediaCodecDecoder::Initialize(const MediaCodecInitParams& params) {
    width_ = params.width;
    height_ = params.height;
    output_texture_handle_ = params.output_texture_handle;
    codec_profile_ = params.codec_profile;
    sps_ = params.sps;
    pps_ = params.pps;
    frame_counter_ = 0;
    pending_packets_.clear();
    latest_frame_.reset();
    use_test_pattern_ = true;

#if defined(__ANDROID__)
    if (params.output_surface != nullptr) {
        std::cout << "MediaCodec decoder: output surface provided but CPU YUV output is selected.\n";
    }
    output_surface_ = nullptr;
    use_surface_output_ = false;
    stride_ = width_;
    slice_height_ = height_;
    if (width_ <= 0 || height_ <= 0) {
        std::cerr << "MediaCodec decoder requires a valid frame size.\n";
        return true;
    }

    codec_ = AMediaCodec_createDecoderByType(kMimeTypeH264);
    if (!codec_) {
        std::cerr << "Failed to create MediaCodec decoder for H.264.\n";
        return true;
    }

    AMediaFormat* format = AMediaFormat_new();
    AMediaFormat_setString(format, AMEDIAFORMAT_KEY_MIME, kMimeTypeH264);
    AMediaFormat_setInt32(format, AMEDIAFORMAT_KEY_WIDTH, width_);
    AMediaFormat_setInt32(format, AMEDIAFORMAT_KEY_HEIGHT, height_);
    if (codec_profile_) {
        AMediaFormat_setInt32(format, AMEDIAFORMAT_KEY_PROFILE, *codec_profile_);
    }
    if (sps_ && !sps_->empty()) {
        AMediaFormat_setBuffer(format, "csd-0", sps_->data(), sps_->size());
    }
    if (pps_ && !pps_->empty()) {
        AMediaFormat_setBuffer(format, "csd-1", pps_->data(), pps_->size());
    }
    AMediaFormat_setInt32(format, AMEDIAFORMAT_KEY_COLOR_FORMAT, COLOR_FormatYUV420Flexible);

    media_status_t status = AMediaCodec_configure(codec_, format, output_surface_, nullptr, 0);
    AMediaFormat_delete(format);
    if (status != AMEDIA_OK) {
        std::cerr << "Failed to configure MediaCodec decoder.\n";
        AMediaCodec_delete(codec_);
        codec_ = nullptr;
        return true;
    }

    status = AMediaCodec_start(codec_);
    if (status != AMEDIA_OK) {
        std::cerr << "Failed to start MediaCodec decoder.\n";
        AMediaCodec_delete(codec_);
        codec_ = nullptr;
        return true;
    }

    std::cout << "MediaCodec decoder initialized for H.264 (" << width_ << "x" << height_ << ").\n";
    use_test_pattern_ = false;
    return true;
#else
    std::cout << "MediaCodec decoder placeholder initialized.\n";
    return true;
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
    latest_frame_.reset();
    use_test_pattern_ = true;
}

void MediaCodecDecoder::SubmitAnnexBFrame(const std::vector<std::uint8_t>& packet, std::int64_t presentation_time_us,
                                         bool end_of_stream) {
    if (packet.empty() && !end_of_stream) {
        return;
    }

    CacheParameterSets(packet);
    pending_packets_.push_back(PendingPacket{packet, presentation_time_us, end_of_stream});
    FeedInputBuffers();
}

bool MediaCodecDecoder::DrainDecodedFrames() {
#if defined(__ANDROID__)
    if (codec_) {
        return DrainOutputBuffers();
    }
#endif
    if (use_test_pattern_) {
        return GenerateTestPatternFrame();
    }
    return false;
}

bool MediaCodecDecoder::AcquireFrame(DecodedFrame& out_frame) {
    if (!latest_frame_) {
        return false;
    }

    out_frame = std::move(*latest_frame_);
    latest_frame_.reset();
    return true;
}

bool MediaCodecDecoder::GenerateTestPatternFrame() {
    constexpr int kFrameWidth = 320;
    constexpr int kFrameHeight = 180;

    DecodedFrame frame;
    frame.width = kFrameWidth;
    frame.height = kFrameHeight;
    frame.y_stride = kFrameWidth;
    frame.uv_stride = kFrameWidth / 2;
    frame.y_plane_height = kFrameHeight;
    frame.uv_plane_height = kFrameHeight / 2;
    frame.frame_index = frame_counter_++;

    const std::size_t luma_size =
        static_cast<std::size_t>(frame.y_stride) * static_cast<std::size_t>(frame.y_plane_height);
    const std::size_t chroma_size =
        static_cast<std::size_t>(frame.uv_stride) * static_cast<std::size_t>(frame.uv_plane_height);

    frame.y_plane.resize(luma_size);
    frame.u_plane.resize(chroma_size);
    frame.v_plane.resize(chroma_size);

    for (int y = 0; y < kFrameHeight; ++y) {
        for (int x = 0; x < kFrameWidth; ++x) {
            frame.y_plane[static_cast<std::size_t>(y * frame.y_stride + x)] =
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

void MediaCodecDecoder::CacheParameterSets(const std::vector<std::uint8_t>& packet) {
    auto scan_start_code = [&packet](std::size_t offset) -> std::optional<std::size_t> {
        for (std::size_t i = offset; i + 3 < packet.size(); ++i) {
            if (packet[i] == 0 && packet[i + 1] == 0) {
                if (packet[i + 2] == 1) {
                    return i;
                }
                if (packet[i + 2] == 0 && packet[i + 3] == 1) {
                    return i;
                }
            }
        }
        return std::nullopt;
    };

    std::vector<std::pair<const std::uint8_t*, std::size_t>> nal_units;
    std::size_t offset = 0;
    while (offset < packet.size()) {
        auto start_code = scan_start_code(offset);
        if (!start_code) {
            break;
        }
        std::size_t start = *start_code;
        std::size_t start_code_size = packet[start + 2] == 1 ? 3 : 4;
        std::size_t nal_start = start + start_code_size;
        auto next_start = scan_start_code(nal_start);
        std::size_t nal_end = next_start ? *next_start : packet.size();
        if (nal_start < nal_end) {
            nal_units.emplace_back(packet.data() + nal_start, nal_end - nal_start);
        }
        offset = nal_end;
    }

    for (const auto& nal : nal_units) {
        if (!nal.first || nal.second == 0) {
            continue;
        }
        std::uint8_t nal_type = nal.first[0] & 0x1F;
        if (nal_type == kNalUnitTypeSps) {
            sps_ = std::vector<std::uint8_t>(nal.first, nal.first + nal.second);
        } else if (nal_type == kNalUnitTypePps) {
            pps_ = std::vector<std::uint8_t>(nal.first, nal.first + nal.second);
        }
    }
}

void MediaCodecDecoder::FeedInputBuffers() {
#if defined(__ANDROID__)
    if (!codec_) {
        return;
    }

    while (!pending_packets_.empty()) {
        ssize_t buffer_index = AMediaCodec_dequeueInputBuffer(codec_, 0);
        if (buffer_index < 0) {
            break;
        }

        std::size_t buffer_size = 0;
        std::uint8_t* buffer = AMediaCodec_getInputBuffer(codec_, buffer_index, &buffer_size);
        PendingPacket packet = std::move(pending_packets_.front());
        pending_packets_.erase(pending_packets_.begin());

        std::size_t copy_size = 0;
        if (buffer && buffer_size > 0 && !packet.data.empty()) {
            copy_size = std::min(buffer_size, packet.data.size());
            std::memcpy(buffer, packet.data.data(), copy_size);
        }

        int flags = packet.end_of_stream ? AMEDIACODEC_BUFFER_FLAG_END_OF_STREAM : 0;
        AMediaCodec_queueInputBuffer(codec_, buffer_index, 0, copy_size, packet.presentation_time_us, flags);
    }
#else
    (void)pending_packets_;
#endif
}

bool MediaCodecDecoder::DrainOutputBuffers() {
#if defined(__ANDROID__)
    if (!codec_) {
        return false;
    }

    bool produced_frame = false;
    for (;;) {
        AMediaCodecBufferInfo info;
        ssize_t output_index = AMediaCodec_dequeueOutputBuffer(codec_, &info, 0);
        if (output_index >= 0) {
            if (info.size > 0 && !use_surface_output_) {
                std::size_t buffer_size = 0;
                std::uint8_t* buffer = AMediaCodec_getOutputBuffer(codec_, output_index, &buffer_size);
                if (buffer && buffer_size > static_cast<std::size_t>(info.offset)) {
                    std::uint8_t* data = buffer + info.offset;
                    std::size_t data_size = std::min(static_cast<std::size_t>(info.size),
                                                     buffer_size - static_cast<std::size_t>(info.offset));

                    const int32_t stride = stride_ > 0 ? stride_ : width_;
                    const int32_t slice_height = slice_height_ > 0 ? slice_height_ : height_;
                    const std::size_t y_plane_size =
                        static_cast<std::size_t>(stride) * static_cast<std::size_t>(slice_height);
                    const std::size_t chroma_stride = static_cast<std::size_t>(stride / 2);
                    const std::size_t chroma_height = static_cast<std::size_t>(slice_height / 2);
                    const std::size_t chroma_plane_size = chroma_stride * chroma_height;

                    if (data_size >= y_plane_size + chroma_plane_size * 2U) {
                        DecodedFrame frame;
                        frame.width = width_;
                        frame.height = height_;
                        frame.frame_index = frame_counter_++;
                        frame.y_stride = stride;
                        frame.uv_stride = static_cast<int>(chroma_stride);
                        frame.y_plane_height = slice_height;
                        frame.uv_plane_height = static_cast<int>(chroma_height);
                        frame.y_plane.resize(y_plane_size);
                        frame.u_plane.resize(chroma_plane_size);
                        frame.v_plane.resize(chroma_plane_size);

                        for (int y = 0; y < slice_height; ++y) {
                            const std::uint8_t* src = data + static_cast<std::size_t>(y * stride);
                            std::uint8_t* dst = frame.y_plane.data() + static_cast<std::size_t>(y * stride);
                            std::memcpy(dst, src, static_cast<std::size_t>(stride));
                        }

                        const std::uint8_t* u_plane = data + y_plane_size;
                        const std::uint8_t* v_plane = u_plane + chroma_plane_size;
                        for (int y = 0; y < static_cast<int>(chroma_height); ++y) {
                            const std::uint8_t* src_u = u_plane + static_cast<std::size_t>(y * chroma_stride);
                            const std::uint8_t* src_v = v_plane + static_cast<std::size_t>(y * chroma_stride);
                            std::uint8_t* dst_u =
                                frame.u_plane.data() + static_cast<std::size_t>(y * chroma_stride);
                            std::uint8_t* dst_v =
                                frame.v_plane.data() + static_cast<std::size_t>(y * chroma_stride);
                            std::memcpy(dst_u, src_u, chroma_stride);
                            std::memcpy(dst_v, src_v, chroma_stride);
                        }

                        latest_frame_ = std::move(frame);
                        produced_frame = true;
                    }
                }
            }

            AMediaCodec_releaseOutputBuffer(codec_, output_index, use_surface_output_);
            if ((info.flags & AMEDIACODEC_BUFFER_FLAG_END_OF_STREAM) != 0) {
                break;
            }
        } else if (output_index == AMEDIACODEC_INFO_TRY_AGAIN_LATER) {
            break;
        } else if (output_index == AMEDIACODEC_INFO_OUTPUT_FORMAT_CHANGED) {
            AMediaFormat* format = AMediaCodec_getOutputFormat(codec_);
            if (format) {
                int32_t stride = 0;
                int32_t slice_height = 0;
                AMediaFormat_getInt32(format, AMEDIAFORMAT_KEY_WIDTH, &width_);
                AMediaFormat_getInt32(format, AMEDIAFORMAT_KEY_HEIGHT, &height_);
                if (AMediaFormat_getInt32(format, AMEDIAFORMAT_KEY_STRIDE, &stride)) {
                    stride_ = stride;
                }
                if (AMediaFormat_getInt32(format, AMEDIAFORMAT_KEY_SLICE_HEIGHT, &slice_height)) {
                    slice_height_ = slice_height;
                }
                AMediaFormat_delete(format);
            }
        } else if (output_index == AMEDIACODEC_INFO_OUTPUT_BUFFERS_CHANGED) {
            continue;
        } else {
            break;
        }
    }

    return produced_frame;
#else
    return false;
#endif
}
