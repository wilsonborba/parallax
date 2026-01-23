#include "OpenXRApp.h"

#include <chrono>
#include <cstring>
#include <iostream>
#include <thread>

namespace {
constexpr std::uint16_t kDefaultUdpPort = 7777;
constexpr int kDefaultVideoWidth = 1920;
constexpr int kDefaultVideoHeight = 1080;
} // namespace

bool OpenXRApp::Initialize() {
    if (!InitializeOpenXR()) {
        return false;
    }

    if (!udp_receiver_.Initialize(kDefaultUdpPort)) {
        return false;
    }

    MediaCodecInitParams decoder_params;
    decoder_params.width = kDefaultVideoWidth;
    decoder_params.height = kDefaultVideoHeight;
    if (!decoder_.Initialize(decoder_params)) {
        return false;
    }

    if (!renderer_.Initialize(instance_, system_id_)) {
        return false;
    }

    running_ = true;
    return true;
}

void OpenXRApp::Run() {
    if (!running_) {
        return;
    }

    FrameLoop();
}

void OpenXRApp::Shutdown() {
    renderer_.Shutdown();
#if PARALLAX_HAVE_OPENXR && PARALLAX_HAS_OPENXR_HEADERS
    if (instance_ != XR_NULL_HANDLE) {
        xrDestroyInstance(instance_);
        instance_ = XR_NULL_HANDLE;
    }
#endif
    decoder_.Shutdown();
    running_ = false;
}

bool OpenXRApp::InitializeOpenXR() {
#if PARALLAX_HAVE_OPENXR && PARALLAX_HAS_OPENXR_HEADERS
    XrInstanceCreateInfo create_info{};
    create_info.type = XR_TYPE_INSTANCE_CREATE_INFO;
    create_info.next = nullptr;
    create_info.applicationVersion = 1;
    create_info.engineVersion = 1;
    create_info.apiVersion = XR_CURRENT_API_VERSION;
    std::snprintf(create_info.applicationName, sizeof(create_info.applicationName), "ParallaxClient");
    std::snprintf(create_info.engineName, sizeof(create_info.engineName), "Parallax");

    XrResult result = xrCreateInstance(&create_info, &instance_);
    if (result != XR_SUCCESS) {
        std::cerr << "Failed to create OpenXR instance." << std::endl;
        return false;
    }

    XrSystemGetInfo system_info{};
    system_info.type = XR_TYPE_SYSTEM_GET_INFO;
    system_info.formFactor = XR_FORM_FACTOR_HEAD_MOUNTED_DISPLAY;

    result = xrGetSystem(instance_, &system_info, &system_id_);
    if (result != XR_SUCCESS) {
        std::cerr << "Failed to get OpenXR system." << std::endl;
        return false;
    }

    std::cout << "OpenXR initialized with system id " << system_id_ << ".\n";
    return true;
#else
    std::cout << "OpenXR headers/loader not available; running in placeholder mode." << std::endl;
    return true;
#endif
}

void OpenXRApp::FrameLoop() {
    while (running_) {
        auto packet = udp_receiver_.ReceivePacket();
        if (!packet.empty()) {
            decoder_.SubmitPacket(packet);
        }

        while (decoder_.DecodeNextFrame()) {
            DecodedFrame decoded_frame;
            if (decoder_.AcquireFrame(decoded_frame)) {
                renderer_.RenderFrame(decoded_frame);
            }
        }

        std::this_thread::sleep_for(std::chrono::milliseconds(16));
    }
}
