#pragma once

#include "ParallaxOpenXR.h"

#include <cstdint>
#include <vector>

struct DecodedFrame;

class QuadRenderer {
public:
    bool Initialize(XrInstance instance, XrSystemId system_id);
    void RenderFrame(const DecodedFrame& frame);
    void Shutdown();

private:
    enum class ShaderPath {
        kYuv,
        kRgb
    };

    struct TextureSet {
        int width = 0;
        int height = 0;
        std::vector<std::uint8_t> y_plane;
        std::vector<std::uint8_t> u_plane;
        std::vector<std::uint8_t> v_plane;
        std::vector<std::uint8_t> rgb;
        std::uint64_t frame_index = 0;
        bool in_use = false;
    };

    TextureSet& AcquireTextureSet(const DecodedFrame& frame);
    void UploadFrame(const DecodedFrame& frame, TextureSet& textures);
    void ConvertYuvToRgb(const DecodedFrame& frame, TextureSet& textures);

    ShaderPath shader_path_ = ShaderPath::kRgb;
    std::vector<TextureSet> texture_pool_;
    std::size_t texture_pool_index_ = 0;

#if PARALLAX_HAVE_OPENXR && PARALLAX_HAS_OPENXR_HEADERS
    bool InitializeOpenXRResources();
    void DestroyOpenXRResources();

    XrInstance instance_ = XR_NULL_HANDLE;
    XrSystemId system_id_ = XR_NULL_SYSTEM_ID;
    XrSession session_ = XR_NULL_HANDLE;
    XrSwapchain swapchain_ = XR_NULL_HANDLE;
    XrSpace local_space_ = XR_NULL_HANDLE;
    XrExtent2Df quad_size_{0.8f, 0.45f};
    XrPosef quad_pose_{ {0.0f, 0.0f, 0.0f, 1.0f}, {0.0f, 0.0f, -2.0f} };
#else
    XrInstance instance_ = XR_NULL_HANDLE;
    XrSystemId system_id_ = XR_NULL_SYSTEM_ID;
#endif
};
