#pragma once

#include "MediaCodecDecoder.h"

class QuadRenderer {
public:
    bool Initialize();
    void RenderFrame(const MediaCodecDecoder::DecodedFrame& frame);
};
