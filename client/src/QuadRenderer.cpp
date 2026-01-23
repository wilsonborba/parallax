#include "QuadRenderer.h"

#include "MediaCodecDecoder.h"

#include <algorithm>
#include <iostream>

namespace {
constexpr std::size_t kTexturePoolSize = 3;
constexpr bool kShaderPrefersYuv = true;
} // namespace

bool QuadRenderer::Initialize(XrInstance instance, XrSystemId system_id) {
    instance_ = instance;
    system_id_ = system_id;
    shader_path_ = kShaderPrefersYuv ? ShaderPath::kYuv : ShaderPath::kRgb;
    texture_pool_.resize(kTexturePoolSize);
    std::cout << "Quad renderer initialized with a texture pool of " << kTexturePoolSize << ".\n";

#if PARALLAX_HAVE_OPENXR && PARALLAX_HAS_OPENXR_HEADERS
    return InitializeOpenXRResources();
#else
    return true;
#endif
}

void QuadRenderer::RenderFrame(const DecodedFrame& frame) {
    TextureSet& textures = AcquireTextureSet(frame);
    UploadFrame(frame, textures);

#if PARALLAX_HAVE_OPENXR && PARALLAX_HAS_OPENXR_HEADERS
    if (session_ != XR_NULL_HANDLE && swapchain_ != XR_NULL_HANDLE && local_space_ != XR_NULL_HANDLE) {
        XrFrameWaitInfo wait_info{XR_TYPE_FRAME_WAIT_INFO};
        XrFrameState frame_state{XR_TYPE_FRAME_STATE};
        xrWaitFrame(session_, &wait_info, &frame_state);

        XrFrameBeginInfo begin_info{XR_TYPE_FRAME_BEGIN_INFO};
        xrBeginFrame(session_, &begin_info);

        XrSwapchainImageAcquireInfo acquire_info{XR_TYPE_SWAPCHAIN_IMAGE_ACQUIRE_INFO};
        std::uint32_t image_index = 0;
        xrAcquireSwapchainImage(swapchain_, &acquire_info, &image_index);

        XrSwapchainImageWaitInfo wait_image_info{XR_TYPE_SWAPCHAIN_IMAGE_WAIT_INFO};
        wait_image_info.timeout = XR_INFINITE_DURATION;
        xrWaitSwapchainImage(swapchain_, &wait_image_info);

        XrCompositionLayerQuad quad_layer{XR_TYPE_COMPOSITION_LAYER_QUAD};
        quad_layer.space = local_space_;
        quad_layer.pose = quad_pose_;
        quad_layer.size = quad_size_;
        quad_layer.subImage.swapchain = swapchain_;
        quad_layer.subImage.imageRect.offset = {0, 0};
        quad_layer.subImage.imageRect.extent = {textures.width, textures.height};

        const XrCompositionLayerBaseHeader* layers[] = {
            reinterpret_cast<const XrCompositionLayerBaseHeader*>(&quad_layer)
        };

        XrFrameEndInfo end_info{XR_TYPE_FRAME_END_INFO};
        end_info.displayTime = frame_state.predictedDisplayTime;
        end_info.environmentBlendMode = XR_ENVIRONMENT_BLEND_MODE_OPAQUE;
        end_info.layerCount = 1;
        end_info.layers = layers;
        xrEndFrame(session_, &end_info);

        XrSwapchainImageReleaseInfo release_info{XR_TYPE_SWAPCHAIN_IMAGE_RELEASE_INFO};
        xrReleaseSwapchainImage(swapchain_, &release_info);
    }
#else
    std::cout << "Rendered frame " << textures.frame_index << " in placeholder mode.\n";
#endif

    textures.in_use = false;
}

void QuadRenderer::Shutdown() {
#if PARALLAX_HAVE_OPENXR && PARALLAX_HAS_OPENXR_HEADERS
    DestroyOpenXRResources();
#endif
    texture_pool_.clear();
}

QuadRenderer::TextureSet& QuadRenderer::AcquireTextureSet(const DecodedFrame& frame) {
    TextureSet& candidate = texture_pool_[texture_pool_index_];
    if (candidate.in_use) {
        std::cout << "Texture pool exhausted; reusing in-flight textures." << std::endl;
    }
    candidate.in_use = true;
    candidate.frame_index = frame.frame_index;
    texture_pool_index_ = (texture_pool_index_ + 1) % texture_pool_.size();
    return candidate;
}

void QuadRenderer::UploadFrame(const DecodedFrame& frame, TextureSet& textures) {
    textures.width = frame.width;
    textures.height = frame.height;
    textures.y_plane = frame.y_plane;
    textures.u_plane = frame.u_plane;
    textures.v_plane = frame.v_plane;

    if (shader_path_ == ShaderPath::kRgb) {
        ConvertYuvToRgb(frame, textures);
    }
}

void QuadRenderer::ConvertYuvToRgb(const DecodedFrame& frame, TextureSet& textures) {
    const int width = frame.width;
    const int height = frame.height;
    textures.rgb.resize(static_cast<std::size_t>(width * height * 3));

    for (int y = 0; y < height; ++y) {
        for (int x = 0; x < width; ++x) {
            const int y_index = y * width + x;
            const int chroma_x = x / 2;
            const int chroma_y = y / 2;
            const int chroma_index = chroma_y * (width / 2) + chroma_x;

            const int luma = static_cast<int>(frame.y_plane[static_cast<std::size_t>(y_index)]);
            const int u = static_cast<int>(frame.u_plane[static_cast<std::size_t>(chroma_index)]) - 128;
            const int v = static_cast<int>(frame.v_plane[static_cast<std::size_t>(chroma_index)]) - 128;

            int r = static_cast<int>(luma + 1.402f * v);
            int g = static_cast<int>(luma - 0.344f * u - 0.714f * v);
            int b = static_cast<int>(luma + 1.772f * u);

            r = std::clamp(r, 0, 255);
            g = std::clamp(g, 0, 255);
            b = std::clamp(b, 0, 255);

            const std::size_t out_index = static_cast<std::size_t>(y_index * 3);
            textures.rgb[out_index + 0] = static_cast<std::uint8_t>(r);
            textures.rgb[out_index + 1] = static_cast<std::uint8_t>(g);
            textures.rgb[out_index + 2] = static_cast<std::uint8_t>(b);
        }
    }
}

#if PARALLAX_HAVE_OPENXR && PARALLAX_HAS_OPENXR_HEADERS
bool QuadRenderer::InitializeOpenXRResources() {
    if (instance_ == XR_NULL_HANDLE || system_id_ == XR_NULL_SYSTEM_ID) {
        std::cout << "OpenXR handles not set; skipping OpenXR renderer setup." << std::endl;
        return true;
    }

    XrSessionCreateInfo session_info{XR_TYPE_SESSION_CREATE_INFO};
    session_info.systemId = system_id_;
    XrResult result = xrCreateSession(instance_, &session_info, &session_);
    if (result != XR_SUCCESS) {
        std::cout << "Failed to create OpenXR session; running without swapchain." << std::endl;
        session_ = XR_NULL_HANDLE;
        return true;
    }

    XrReferenceSpaceCreateInfo space_info{XR_TYPE_REFERENCE_SPACE_CREATE_INFO};
    space_info.referenceSpaceType = XR_REFERENCE_SPACE_TYPE_LOCAL;
    space_info.poseInReferenceSpace = quad_pose_;
    result = xrCreateReferenceSpace(session_, &space_info, &local_space_);
    if (result != XR_SUCCESS) {
        std::cout << "Failed to create reference space." << std::endl;
        local_space_ = XR_NULL_HANDLE;
    }

    XrSwapchainCreateInfo swapchain_info{XR_TYPE_SWAPCHAIN_CREATE_INFO};
    swapchain_info.arraySize = 1;
    swapchain_info.faceCount = 1;
    swapchain_info.mipCount = 1;
    swapchain_info.sampleCount = 1;
    swapchain_info.format = 0;
    swapchain_info.width = 1024;
    swapchain_info.height = 1024;
    swapchain_info.usageFlags = XR_SWAPCHAIN_USAGE_SAMPLED_BIT;
    result = xrCreateSwapchain(session_, &swapchain_info, &swapchain_);
    if (result != XR_SUCCESS) {
        std::cout << "Failed to create swapchain." << std::endl;
        swapchain_ = XR_NULL_HANDLE;
    }

    return true;
}

void QuadRenderer::DestroyOpenXRResources() {
    if (swapchain_ != XR_NULL_HANDLE) {
        xrDestroySwapchain(swapchain_);
        swapchain_ = XR_NULL_HANDLE;
    }

    if (local_space_ != XR_NULL_HANDLE) {
        xrDestroySpace(local_space_);
        local_space_ = XR_NULL_HANDLE;
    }

    if (session_ != XR_NULL_HANDLE) {
        xrDestroySession(session_);
        session_ = XR_NULL_HANDLE;
    }
}
#endif
