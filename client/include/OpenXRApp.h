#pragma once

#include "MediaCodecDecoder.h"
#include "ParallaxOpenXR.h"
#include "QuadRenderer.h"
#include "UdpReceiver.h"

class OpenXRApp {
public:
    bool Initialize();
    void Run();
    void Shutdown();

private:
    bool InitializeOpenXR();
    void FrameLoop();

    XrInstance instance_ = XR_NULL_HANDLE;
    XrSystemId system_id_ = XR_NULL_SYSTEM_ID;
    bool running_ = false;

    UdpReceiver udp_receiver_;
    MediaCodecDecoder decoder_;
    QuadRenderer renderer_;
};
