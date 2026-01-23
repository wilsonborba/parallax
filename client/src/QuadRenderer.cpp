#include "QuadRenderer.h"

#include <iostream>

bool QuadRenderer::Initialize() {
    std::cout << "Quad renderer placeholder initialized.\n";
    return true;
}

void QuadRenderer::RenderFrame(const MediaCodecDecoder::DecodedFrame& frame) {
    (void)frame;
    // Placeholder: bind YUV planes or texture output for rendering.
}
