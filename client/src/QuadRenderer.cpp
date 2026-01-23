#include "QuadRenderer.h"

#include "MediaCodecDecoder.h"

#include <algorithm>
#include <iostream>

#if defined(__has_include)
#if __has_include(<EGL/egl.h>)
#include <EGL/egl.h>
#define PARALLAX_HAVE_EGL 1
#else
#define PARALLAX_HAVE_EGL 0
#endif
#if __has_include(<GLES3/gl3.h>)
#include <GLES3/gl3.h>
#define PARALLAX_HAVE_GLES 1
#else
#define PARALLAX_HAVE_GLES 0
#endif
#else
#define PARALLAX_HAVE_EGL 0
#define PARALLAX_HAVE_GLES 0
#endif

namespace {
constexpr std::size_t kTexturePoolSize = 3;
constexpr bool kShaderPrefersYuv = true;
#if PARALLAX_HAVE_OPENXR && PARALLAX_HAS_OPENXR_HEADERS && PARALLAX_HAVE_GLES
constexpr std::uint32_t kQuadStride = sizeof(float) * 4;
constexpr int kTextureUnitY = 0;
constexpr int kTextureUnitU = 1;
constexpr int kTextureUnitV = 2;
constexpr int kTextureUnitRgb = 0;

GLuint CompileShader(GLenum type, const char* source) {
    GLuint shader = glCreateShader(type);
    if (shader == 0) {
        return 0;
    }
    glShaderSource(shader, 1, &source, nullptr);
    glCompileShader(shader);
    GLint compiled = GL_FALSE;
    glGetShaderiv(shader, GL_COMPILE_STATUS, &compiled);
    if (compiled != GL_TRUE) {
        glDeleteShader(shader);
        return 0;
    }
    return shader;
}

GLuint CreateProgram(const char* vertex_source, const char* fragment_source) {
    GLuint vertex_shader = CompileShader(GL_VERTEX_SHADER, vertex_source);
    if (vertex_shader == 0) {
        return 0;
    }
    GLuint fragment_shader = CompileShader(GL_FRAGMENT_SHADER, fragment_source);
    if (fragment_shader == 0) {
        glDeleteShader(vertex_shader);
        return 0;
    }
    GLuint program = glCreateProgram();
    if (program == 0) {
        glDeleteShader(vertex_shader);
        glDeleteShader(fragment_shader);
        return 0;
    }
    glAttachShader(program, vertex_shader);
    glAttachShader(program, fragment_shader);
    glLinkProgram(program);
    glDeleteShader(vertex_shader);
    glDeleteShader(fragment_shader);
    GLint linked = GL_FALSE;
    glGetProgramiv(program, GL_LINK_STATUS, &linked);
    if (linked != GL_TRUE) {
        glDeleteProgram(program);
        return 0;
    }
    return program;
}

void ConfigureTexture(GLenum target) {
    glTexParameteri(target, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(target, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    glTexParameteri(target, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
    glTexParameteri(target, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
}
#endif
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
    if (session_ != XR_NULL_HANDLE && local_space_ != XR_NULL_HANDLE) {
        if (EnsureSwapchain(textures.width, textures.height)) {
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

            if (RenderSwapchainImage(image_index, textures)) {
                XrCompositionLayerQuad quad_layer{XR_TYPE_COMPOSITION_LAYER_QUAD};
                quad_layer.space = local_space_;
                quad_layer.pose = quad_pose_;
                quad_layer.size = quad_size_;
                quad_layer.subImage.swapchain = swapchain_;
                quad_layer.subImage.imageRect.offset = {0, 0};
                quad_layer.subImage.imageRect.extent = {swapchain_width_, swapchain_height_};

                const XrCompositionLayerBaseHeader* layers[] = {
                    reinterpret_cast<const XrCompositionLayerBaseHeader*>(&quad_layer)
                };

                XrFrameEndInfo end_info{XR_TYPE_FRAME_END_INFO};
                end_info.displayTime = frame_state.predictedDisplayTime;
                end_info.environmentBlendMode = XR_ENVIRONMENT_BLEND_MODE_OPAQUE;
                end_info.layerCount = 1;
                end_info.layers = layers;
                xrEndFrame(session_, &end_info);
            }

            XrSwapchainImageReleaseInfo release_info{XR_TYPE_SWAPCHAIN_IMAGE_RELEASE_INFO};
            xrReleaseSwapchainImage(swapchain_, &release_info);
        }
    }
#endif

    textures.in_use = false;
}

void QuadRenderer::Shutdown() {
    for (auto& textures : texture_pool_) {
        DestroyTextureSet(textures);
    }
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
    textures.y_plane_height = frame.y_plane_height;
    textures.uv_plane_height = frame.uv_plane_height;
    textures.y_stride = frame.y_stride;
    textures.uv_stride = frame.uv_stride;
    textures.y_plane = frame.y_plane;
    textures.u_plane = frame.u_plane;
    textures.v_plane = frame.v_plane;

    if (shader_path_ == ShaderPath::kRgb) {
        ConvertYuvToRgb(frame, textures);
    }
#if PARALLAX_HAVE_OPENXR && PARALLAX_HAS_OPENXR_HEADERS && PARALLAX_HAVE_GLES
    if (!graphics_ready_) {
        return;
    }

    const int width = frame.width;
    const int height = frame.height;
    const int y_stride = frame.y_stride > 0 ? frame.y_stride : width;
    const int uv_stride = frame.uv_stride > 0 ? frame.uv_stride : (width / 2);
    const int y_height = frame.y_plane_height > 0 ? frame.y_plane_height : height;
    const int uv_height = frame.uv_plane_height > 0 ? frame.uv_plane_height : (height / 2);
    const int uv_width = width / 2;

    if (shader_path_ == ShaderPath::kYuv) {
        if (textures.y_texture == 0) {
            glGenTextures(1, &textures.y_texture);
            glBindTexture(GL_TEXTURE_2D, textures.y_texture);
            ConfigureTexture(GL_TEXTURE_2D);
        }
        if (textures.u_texture == 0) {
            glGenTextures(1, &textures.u_texture);
            glBindTexture(GL_TEXTURE_2D, textures.u_texture);
            ConfigureTexture(GL_TEXTURE_2D);
        }
        if (textures.v_texture == 0) {
            glGenTextures(1, &textures.v_texture);
            glBindTexture(GL_TEXTURE_2D, textures.v_texture);
            ConfigureTexture(GL_TEXTURE_2D);
        }

        if (textures.gpu_width != width || textures.gpu_height != height) {
            glBindTexture(GL_TEXTURE_2D, textures.y_texture);
            glTexImage2D(GL_TEXTURE_2D, 0, GL_R8, width, height, 0, GL_RED, GL_UNSIGNED_BYTE, nullptr);
            glBindTexture(GL_TEXTURE_2D, textures.u_texture);
            glTexImage2D(GL_TEXTURE_2D, 0, GL_R8, uv_width, uv_height, 0, GL_RED, GL_UNSIGNED_BYTE, nullptr);
            glBindTexture(GL_TEXTURE_2D, textures.v_texture);
            glTexImage2D(GL_TEXTURE_2D, 0, GL_R8, uv_width, uv_height, 0, GL_RED, GL_UNSIGNED_BYTE, nullptr);
            textures.gpu_width = width;
            textures.gpu_height = height;
            textures.gpu_uv_width = uv_width;
            textures.gpu_uv_height = uv_height;
        }

        glPixelStorei(GL_UNPACK_ALIGNMENT, 1);
        glPixelStorei(GL_UNPACK_ROW_LENGTH, y_stride);
        glBindTexture(GL_TEXTURE_2D, textures.y_texture);
        glTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, width, y_height, GL_RED, GL_UNSIGNED_BYTE,
                        textures.y_plane.data());
        glPixelStorei(GL_UNPACK_ROW_LENGTH, uv_stride);
        glBindTexture(GL_TEXTURE_2D, textures.u_texture);
        glTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, uv_width, uv_height, GL_RED, GL_UNSIGNED_BYTE,
                        textures.u_plane.data());
        glBindTexture(GL_TEXTURE_2D, textures.v_texture);
        glTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, uv_width, uv_height, GL_RED, GL_UNSIGNED_BYTE,
                        textures.v_plane.data());
        glPixelStorei(GL_UNPACK_ROW_LENGTH, 0);
    } else {
        if (textures.rgb_texture == 0) {
            glGenTextures(1, &textures.rgb_texture);
            glBindTexture(GL_TEXTURE_2D, textures.rgb_texture);
            ConfigureTexture(GL_TEXTURE_2D);
        }
        if (textures.gpu_width != width || textures.gpu_height != height) {
            glBindTexture(GL_TEXTURE_2D, textures.rgb_texture);
            glTexImage2D(GL_TEXTURE_2D, 0, GL_RGB8, width, height, 0, GL_RGB, GL_UNSIGNED_BYTE, nullptr);
            textures.gpu_width = width;
            textures.gpu_height = height;
        }
        glPixelStorei(GL_UNPACK_ALIGNMENT, 1);
        glBindTexture(GL_TEXTURE_2D, textures.rgb_texture);
        glTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, width, height, GL_RGB, GL_UNSIGNED_BYTE, textures.rgb.data());
    }
#endif
}

void QuadRenderer::ConvertYuvToRgb(const DecodedFrame& frame, TextureSet& textures) {
    const int width = frame.width;
    const int height = frame.height;
    const int y_stride = frame.y_stride > 0 ? frame.y_stride : width;
    const int uv_stride = frame.uv_stride > 0 ? frame.uv_stride : (width / 2);
    textures.rgb.resize(static_cast<std::size_t>(width * height * 3));

    for (int y = 0; y < height; ++y) {
        for (int x = 0; x < width; ++x) {
            const int y_index = y * y_stride + x;
            const int chroma_x = x / 2;
            const int chroma_y = y / 2;
            const int chroma_index = chroma_y * uv_stride + chroma_x;

            const int luma = static_cast<int>(frame.y_plane[static_cast<std::size_t>(y_index)]);
            const int u = static_cast<int>(frame.u_plane[static_cast<std::size_t>(chroma_index)]) - 128;
            const int v = static_cast<int>(frame.v_plane[static_cast<std::size_t>(chroma_index)]) - 128;

            int r = static_cast<int>(luma + 1.402f * v);
            int g = static_cast<int>(luma - 0.344f * u - 0.714f * v);
            int b = static_cast<int>(luma + 1.772f * u);

            r = std::clamp(r, 0, 255);
            g = std::clamp(g, 0, 255);
            b = std::clamp(b, 0, 255);

            const std::size_t out_index = static_cast<std::size_t>((y * width + x) * 3);
            textures.rgb[out_index + 0] = static_cast<std::uint8_t>(r);
            textures.rgb[out_index + 1] = static_cast<std::uint8_t>(g);
            textures.rgb[out_index + 2] = static_cast<std::uint8_t>(b);
        }
    }
}

void QuadRenderer::DestroyTextureSet(TextureSet& textures) {
#if PARALLAX_HAVE_OPENXR && PARALLAX_HAS_OPENXR_HEADERS && PARALLAX_HAVE_GLES
    if (textures.y_texture != 0) {
        glDeleteTextures(1, &textures.y_texture);
        textures.y_texture = 0;
    }
    if (textures.u_texture != 0) {
        glDeleteTextures(1, &textures.u_texture);
        textures.u_texture = 0;
    }
    if (textures.v_texture != 0) {
        glDeleteTextures(1, &textures.v_texture);
        textures.v_texture = 0;
    }
    if (textures.rgb_texture != 0) {
        glDeleteTextures(1, &textures.rgb_texture);
        textures.rgb_texture = 0;
    }
#endif
    textures.rgb.clear();
    textures.y_plane.clear();
    textures.u_plane.clear();
    textures.v_plane.clear();
    textures.gpu_width = 0;
    textures.gpu_height = 0;
    textures.gpu_uv_width = 0;
    textures.gpu_uv_height = 0;
}

#if PARALLAX_HAVE_OPENXR && PARALLAX_HAS_OPENXR_HEADERS
bool QuadRenderer::InitializeOpenXRResources() {
    if (instance_ == XR_NULL_HANDLE || system_id_ == XR_NULL_SYSTEM_ID) {
        std::cout << "OpenXR handles not set; skipping OpenXR renderer setup." << std::endl;
        return true;
    }

#if PARALLAX_HAVE_GLES && PARALLAX_HAVE_EGL && defined(XR_TYPE_GRAPHICS_BINDING_OPENGL_ES_ANDROID_KHR)
    EGLDisplay display = eglGetDisplay(EGL_DEFAULT_DISPLAY);
    if (display == EGL_NO_DISPLAY) {
        std::cout << "Failed to get EGL display; running without swapchain." << std::endl;
        return true;
    }
    if (eglInitialize(display, nullptr, nullptr) == EGL_FALSE) {
        std::cout << "Failed to initialize EGL; running without swapchain." << std::endl;
        return true;
    }

    const EGLint config_attributes[] = {
        EGL_RENDERABLE_TYPE, EGL_OPENGL_ES3_BIT,
        EGL_SURFACE_TYPE, EGL_PBUFFER_BIT,
        EGL_RED_SIZE, 8,
        EGL_GREEN_SIZE, 8,
        EGL_BLUE_SIZE, 8,
        EGL_ALPHA_SIZE, 8,
        EGL_NONE
    };
    EGLConfig config = nullptr;
    EGLint config_count = 0;
    if (eglChooseConfig(display, config_attributes, &config, 1, &config_count) == EGL_FALSE || config_count == 0) {
        std::cout << "Failed to choose EGL config; running without swapchain." << std::endl;
        return true;
    }

    const EGLint pbuffer_attributes[] = {
        EGL_WIDTH, 16,
        EGL_HEIGHT, 16,
        EGL_NONE
    };
    EGLSurface surface = eglCreatePbufferSurface(display, config, pbuffer_attributes);
    if (surface == EGL_NO_SURFACE) {
        std::cout << "Failed to create EGL surface; running without swapchain." << std::endl;
        return true;
    }

    const EGLint context_attributes[] = {
        EGL_CONTEXT_CLIENT_VERSION, 3,
        EGL_NONE
    };
    EGLContext context = eglCreateContext(display, config, EGL_NO_CONTEXT, context_attributes);
    if (context == EGL_NO_CONTEXT) {
        std::cout << "Failed to create EGL context; running without swapchain." << std::endl;
        return true;
    }
    if (eglMakeCurrent(display, surface, surface, context) == EGL_FALSE) {
        std::cout << "Failed to make EGL context current; running without swapchain." << std::endl;
        return true;
    }

    egl_display_ = display;
    egl_context_ = context;
    egl_surface_ = surface;

    XrGraphicsBindingOpenGLESAndroidKHR graphics_binding{XR_TYPE_GRAPHICS_BINDING_OPENGL_ES_ANDROID_KHR};
    graphics_binding.display = display;
    graphics_binding.config = config;
    graphics_binding.context = context;

    XrSessionCreateInfo session_info{XR_TYPE_SESSION_CREATE_INFO};
    session_info.next = &graphics_binding;
    session_info.systemId = system_id_;
    XrResult result = xrCreateSession(instance_, &session_info, &session_);
    if (result != XR_SUCCESS) {
        std::cout << "Failed to create OpenXR session; running without swapchain." << std::endl;
        session_ = XR_NULL_HANDLE;
        return true;
    }

    if (!InitializeGraphicsResources()) {
        std::cout << "Failed to initialize graphics resources; running without swapchain." << std::endl;
        xrDestroySession(session_);
        session_ = XR_NULL_HANDLE;
        return true;
    }
#else
    std::cout << "OpenGL ES graphics binding not available; running without swapchain." << std::endl;
    return true;
#endif

    XrReferenceSpaceCreateInfo space_info{XR_TYPE_REFERENCE_SPACE_CREATE_INFO};
    space_info.referenceSpaceType = XR_REFERENCE_SPACE_TYPE_LOCAL;
    space_info.poseInReferenceSpace = quad_pose_;
    XrResult result = xrCreateReferenceSpace(session_, &space_info, &local_space_);
    if (result != XR_SUCCESS) {
        std::cout << "Failed to create reference space." << std::endl;
        local_space_ = XR_NULL_HANDLE;
    }

    return true;
}

void QuadRenderer::DestroyOpenXRResources() {
    DestroyGraphicsResources();
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
#if PARALLAX_HAVE_EGL
    if (egl_display_ != nullptr) {
        EGLDisplay display = static_cast<EGLDisplay>(egl_display_);
        EGLSurface surface = static_cast<EGLSurface>(egl_surface_);
        EGLContext context = static_cast<EGLContext>(egl_context_);
        if (context != EGL_NO_CONTEXT) {
            eglDestroyContext(display, context);
        }
        if (surface != EGL_NO_SURFACE) {
            eglDestroySurface(display, surface);
        }
        eglTerminate(display);
    }
    egl_display_ = nullptr;
    egl_surface_ = nullptr;
    egl_context_ = nullptr;
#endif
}

bool QuadRenderer::InitializeGraphicsResources() {
#if PARALLAX_HAVE_GLES
    if (graphics_ready_) {
        return true;
    }
    const char* vertex_source = R"(
        #version 300 es
        layout(location = 0) in vec2 in_pos;
        layout(location = 1) in vec2 in_uv;
        out vec2 v_uv;
        void main() {
            v_uv = in_uv;
            gl_Position = vec4(in_pos, 0.0, 1.0);
        }
    )";

    const char* fragment_source_yuv = R"(
        #version 300 es
        precision mediump float;
        in vec2 v_uv;
        uniform sampler2D u_y;
        uniform sampler2D u_u;
        uniform sampler2D u_v;
        out vec4 outColor;
        void main() {
            float y = texture(u_y, v_uv).r;
            float u = texture(u_u, v_uv).r - 0.5;
            float v = texture(u_v, v_uv).r - 0.5;
            float r = y + 1.402 * v;
            float g = y - 0.344 * u - 0.714 * v;
            float b = y + 1.772 * u;
            outColor = vec4(r, g, b, 1.0);
        }
    )";

    const char* fragment_source_rgb = R"(
        #version 300 es
        precision mediump float;
        in vec2 v_uv;
        uniform sampler2D u_rgb;
        out vec4 outColor;
        void main() {
            outColor = texture(u_rgb, v_uv);
        }
    )";

    shader_program_ = CreateProgram(vertex_source,
                                    shader_path_ == ShaderPath::kYuv ? fragment_source_yuv : fragment_source_rgb);
    if (shader_program_ == 0) {
        return false;
    }

    glGenVertexArrays(1, &vertex_array_);
    glGenBuffers(1, &vertex_buffer_);
    glBindVertexArray(vertex_array_);
    glBindBuffer(GL_ARRAY_BUFFER, vertex_buffer_);
    const float quad_vertices[] = {
        -1.0f, -1.0f, 0.0f, 0.0f,
        1.0f, -1.0f, 1.0f, 0.0f,
        -1.0f, 1.0f, 0.0f, 1.0f,
        1.0f, 1.0f, 1.0f, 1.0f
    };
    glBufferData(GL_ARRAY_BUFFER, sizeof(quad_vertices), quad_vertices, GL_STATIC_DRAW);
    glEnableVertexAttribArray(0);
    glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE, kQuadStride, reinterpret_cast<void*>(0));
    glEnableVertexAttribArray(1);
    glVertexAttribPointer(1, 2, GL_FLOAT, GL_FALSE, kQuadStride, reinterpret_cast<void*>(sizeof(float) * 2));
    glBindBuffer(GL_ARRAY_BUFFER, 0);
    glBindVertexArray(0);

    glGenFramebuffers(1, &frame_buffer_);

    yuv_sampler_location_ = glGetUniformLocation(shader_program_, "u_y");
    u_sampler_location_ = glGetUniformLocation(shader_program_, "u_u");
    v_sampler_location_ = glGetUniformLocation(shader_program_, "u_v");
    rgb_sampler_location_ = glGetUniformLocation(shader_program_, "u_rgb");
    graphics_ready_ = true;
    return true;
#else
    return false;
#endif
}

void QuadRenderer::DestroyGraphicsResources() {
#if PARALLAX_HAVE_GLES
    if (frame_buffer_ != 0) {
        glDeleteFramebuffers(1, &frame_buffer_);
        frame_buffer_ = 0;
    }
    if (vertex_buffer_ != 0) {
        glDeleteBuffers(1, &vertex_buffer_);
        vertex_buffer_ = 0;
    }
    if (vertex_array_ != 0) {
        glDeleteVertexArrays(1, &vertex_array_);
        vertex_array_ = 0;
    }
    if (shader_program_ != 0) {
        glDeleteProgram(shader_program_);
        shader_program_ = 0;
    }
#endif
    graphics_ready_ = false;
    swapchain_images_.clear();
}

bool QuadRenderer::EnsureSwapchain(int width, int height) {
    if (!graphics_ready_) {
        return false;
    }
    if (swapchain_ == XR_NULL_HANDLE || swapchain_width_ != width || swapchain_height_ != height) {
        return CreateSwapchain(width, height);
    }
    return true;
}

bool QuadRenderer::CreateSwapchain(int width, int height) {
    if (session_ == XR_NULL_HANDLE) {
        return false;
    }
    if (swapchain_ != XR_NULL_HANDLE) {
        xrDestroySwapchain(swapchain_);
        swapchain_ = XR_NULL_HANDLE;
    }

    std::uint32_t format_count = 0;
    xrEnumerateSwapchainFormats(session_, 0, &format_count, nullptr);
    std::vector<std::int64_t> formats(format_count);
    if (format_count > 0) {
        xrEnumerateSwapchainFormats(session_, format_count, &format_count, formats.data());
    }
    std::int64_t chosen_format = 0;
#if PARALLAX_HAVE_GLES
    const std::int64_t preferred_formats[] = {GL_SRGB8_ALPHA8, GL_RGBA8};
    for (const auto format : preferred_formats) {
        if (std::find(formats.begin(), formats.end(), format) != formats.end()) {
            chosen_format = format;
            break;
        }
    }
#endif
    if (chosen_format == 0 && !formats.empty()) {
        chosen_format = formats.front();
    }
    if (chosen_format == 0) {
        std::cout << "No compatible swapchain formats reported." << std::endl;
        return false;
    }

    XrSwapchainCreateInfo swapchain_info{XR_TYPE_SWAPCHAIN_CREATE_INFO};
    swapchain_info.arraySize = 1;
    swapchain_info.faceCount = 1;
    swapchain_info.mipCount = 1;
    swapchain_info.sampleCount = 1;
    swapchain_info.format = chosen_format;
    swapchain_info.width = static_cast<std::uint32_t>(width);
    swapchain_info.height = static_cast<std::uint32_t>(height);
    swapchain_info.usageFlags = XR_SWAPCHAIN_USAGE_COLOR_ATTACHMENT_BIT | XR_SWAPCHAIN_USAGE_SAMPLED_BIT;
    XrResult result = xrCreateSwapchain(session_, &swapchain_info, &swapchain_);
    if (result != XR_SUCCESS) {
        std::cout << "Failed to create swapchain." << std::endl;
        swapchain_ = XR_NULL_HANDLE;
        return false;
    }

    swapchain_width_ = width;
    swapchain_height_ = height;

#if PARALLAX_HAVE_GLES && defined(XR_TYPE_SWAPCHAIN_IMAGE_OPENGL_ES_KHR)
    std::uint32_t image_count = 0;
    xrEnumerateSwapchainImages(swapchain_, 0, &image_count, nullptr);
    std::vector<XrSwapchainImageOpenGLESKHR> images(image_count);
    for (auto& image : images) {
        image.type = XR_TYPE_SWAPCHAIN_IMAGE_OPENGL_ES_KHR;
    }
    if (image_count > 0) {
        xrEnumerateSwapchainImages(swapchain_, image_count, &image_count,
                                   reinterpret_cast<XrSwapchainImageBaseHeader*>(images.data()));
    }
    swapchain_images_.clear();
    swapchain_images_.reserve(images.size());
    for (const auto& image : images) {
        swapchain_images_.push_back(image.image);
    }
#else
    swapchain_images_.clear();
#endif
    return true;
}

bool QuadRenderer::RenderSwapchainImage(std::uint32_t image_index, const TextureSet& textures) {
#if PARALLAX_HAVE_GLES
    if (!graphics_ready_ || swapchain_images_.empty() || image_index >= swapchain_images_.size()) {
        return false;
    }
    const GLuint color_texture = swapchain_images_[image_index];
    glBindFramebuffer(GL_FRAMEBUFFER, frame_buffer_);
    glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, color_texture, 0);
    glViewport(0, 0, swapchain_width_, swapchain_height_);
    glDisable(GL_DEPTH_TEST);
    glDisable(GL_CULL_FACE);

    glUseProgram(shader_program_);
    glBindVertexArray(vertex_array_);

    if (shader_path_ == ShaderPath::kYuv) {
        glActiveTexture(GL_TEXTURE0 + kTextureUnitY);
        glBindTexture(GL_TEXTURE_2D, textures.y_texture);
        glUniform1i(yuv_sampler_location_, kTextureUnitY);

        glActiveTexture(GL_TEXTURE0 + kTextureUnitU);
        glBindTexture(GL_TEXTURE_2D, textures.u_texture);
        glUniform1i(u_sampler_location_, kTextureUnitU);

        glActiveTexture(GL_TEXTURE0 + kTextureUnitV);
        glBindTexture(GL_TEXTURE_2D, textures.v_texture);
        glUniform1i(v_sampler_location_, kTextureUnitV);
    } else {
        glActiveTexture(GL_TEXTURE0 + kTextureUnitRgb);
        glBindTexture(GL_TEXTURE_2D, textures.rgb_texture);
        glUniform1i(rgb_sampler_location_, kTextureUnitRgb);
    }

    glDrawArrays(GL_TRIANGLE_STRIP, 0, 4);
    glBindVertexArray(0);
    glUseProgram(0);
    glBindFramebuffer(GL_FRAMEBUFFER, 0);
    return true;
#else
    (void)image_index;
    (void)textures;
    return false;
#endif
}
#endif
