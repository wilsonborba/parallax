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
        int y_plane_height = 0;
        int uv_plane_height = 0;
        int y_stride = 0;
        int uv_stride = 0;
        std::vector<std::uint8_t> y_plane;
        std::vector<std::uint8_t> u_plane;
        std::vector<std::uint8_t> v_plane;
        std::vector<std::uint8_t> rgb;
        std::uint32_t y_texture = 0;
        std::uint32_t u_texture = 0;
        std::uint32_t v_texture = 0;
        std::uint32_t rgb_texture = 0;
        int gpu_width = 0;
        int gpu_height = 0;
        int gpu_uv_width = 0;
        int gpu_uv_height = 0;
        std::uint64_t frame_index = 0;
        bool in_use = false;
    };

    TextureSet& AcquireTextureSet(const DecodedFrame& frame);
    void UploadFrame(const DecodedFrame& frame, TextureSet& textures);
    void ConvertYuvToRgb(const DecodedFrame& frame, TextureSet& textures);
    void DestroyTextureSet(TextureSet& textures);

    ShaderPath shader_path_ = ShaderPath::kRgb;
    std::vector<TextureSet> texture_pool_;
    std::size_t texture_pool_index_ = 0;
#if PARALLAX_HAVE_OPENXR && PARALLAX_HAS_OPENXR_HEADERS
    std::uint32_t shader_program_ = 0;
    std::uint32_t vertex_array_ = 0;
    std::uint32_t vertex_buffer_ = 0;
    std::uint32_t frame_buffer_ = 0;
    int yuv_sampler_location_ = -1;
    int u_sampler_location_ = -1;
    int v_sampler_location_ = -1;
    int rgb_sampler_location_ = -1;
    bool graphics_ready_ = false;
    int swapchain_width_ = 0;
    int swapchain_height_ = 0;
    std::vector<std::uint32_t> swapchain_images_;
    void* egl_display_ = nullptr;
    void* egl_context_ = nullptr;
    void* egl_surface_ = nullptr;
#endif

#if PARALLAX_HAVE_OPENXR && PARALLAX_HAS_OPENXR_HEADERS
    bool InitializeOpenXRResources();
    void DestroyOpenXRResources();
    bool InitializeGraphicsResources();
    void DestroyGraphicsResources();
    bool EnsureSwapchain(int width, int height);
    bool CreateSwapchain(int width, int height);
    bool RenderSwapchainImage(std::uint32_t image_index, const TextureSet& textures);

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
